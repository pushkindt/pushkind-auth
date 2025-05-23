use bcrypt::{DEFAULT_COST, hash, verify};
use diesel::prelude::*;

use crate::db::DbConnection;
use crate::domain::role::Role;
use crate::domain::user::{NewUser, UpdateUser, User};
use crate::models::role::{NewUserRole as DbNewUserRole, Role as DbRole};
use crate::models::user::{NewUser as NewDbUser, UpdateUser as DbUpdateUser, User as DbUser};
use crate::repository::{RepositoryError, UserRepository};

pub struct DieselUserRepository<'a> {
    pub connection: &'a mut DbConnection,
}

impl<'a> DieselUserRepository<'a> {
    pub fn new(connection: &'a mut DbConnection) -> Self {
        Self { connection }
    }
}

impl UserRepository for DieselUserRepository<'_> {
    fn get_by_id(&mut self, id: i32) -> anyhow::Result<Option<User>> {
        use crate::schema::users;

        let result = users::table
            .filter(users::id.eq(id))
            .first::<DbUser>(self.connection)
            .optional()?;

        Ok(result.map(|db_user| db_user.into())) // Convert DbUser to DomainUser
    }

    fn get_by_email(&mut self, email: &str, hub_id: i32) -> anyhow::Result<Option<User>> {
        use crate::schema::users;

        let result = users::table
            .filter(users::email.eq(email))
            .filter(users::hub_id.eq(hub_id))
            .first::<DbUser>(self.connection)
            .optional()?;

        Ok(result.map(|db_user| db_user.into())) // Convert DbUser to DomainUser
    }

    fn create(&mut self, new_user: &NewUser) -> anyhow::Result<User> {
        use crate::schema::users;

        let new_db_user = NewDbUser::try_from(new_user)?; // Convert to DbNewUser
        diesel::insert_into(users::table)
            .values(&new_db_user)
            .get_result::<DbUser>(self.connection)
            .map(|db_user| db_user.into()) // Convert DbUser to DomainUser
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn list(&mut self, hub_id: i32) -> anyhow::Result<Vec<(User, Vec<Role>)>> {
        use crate::schema::roles;
        use crate::schema::user_roles;
        use crate::schema::users;

        let users = users::table
            .filter(users::hub_id.eq(hub_id))
            .load::<DbUser>(self.connection)?;

        let user_ids: Vec<i32> = users.iter().map(|user| user.id).collect();

        let roles = roles::table
            .inner_join(user_roles::table)
            .filter(user_roles::user_id.eq_any(user_ids))
            .select((user_roles::user_id, roles::all_columns))
            .load::<(i32, DbRole)>(self.connection)?;

        let user_with_roles = users
            .into_iter()
            .map(|user| {
                let user_roles = roles
                    .iter()
                    .filter(|(user_id, _)| *user_id == user.id)
                    .map(|(_, role)| role.clone().into())
                    .collect();
                (user.into(), user_roles)
            })
            .collect();

        Ok(user_with_roles) // Convert DbUser to DomainUser
    }

    fn verify_password(&self, password: &str, stored_hash: &str) -> bool {
        verify(password, stored_hash).unwrap_or(false)
    }

    fn get_roles(&mut self, user_id: i32) -> anyhow::Result<Vec<Role>> {
        use crate::schema::roles;
        use crate::schema::user_roles;

        let results = roles::table
            .inner_join(user_roles::table)
            .filter(user_roles::user_id.eq(user_id))
            .select(roles::all_columns)
            .load::<DbRole>(self.connection)?;
        Ok(results.into_iter().map(|db_role| db_role.into()).collect())
    }

    fn update(&mut self, user_id: i32, updates: &UpdateUser) -> anyhow::Result<User> {
        use crate::schema::users;

        let user = self.get_by_id(user_id)?.ok_or(RepositoryError::NotFound)?;

        let password_hash = match updates.password.as_ref() {
            Some(password) if !password.is_empty() => hash(password, DEFAULT_COST)?,
            _ => user.password_hash,
        };

        let db_updates = DbUpdateUser {
            name: updates.name.as_deref(),
            password_hash,
        };

        diesel::update(users::table)
            .filter(users::id.eq(user_id))
            .set(&db_updates)
            .get_result::<DbUser>(self.connection)
            .map(|db_user| db_user.into()) // Convert DbUser to DomainUser
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn delete(&mut self, user_id: i32) -> anyhow::Result<()> {
        use crate::schema::user_roles;
        use crate::schema::users;

        diesel::delete(user_roles::table)
            .filter(user_roles::user_id.eq(user_id))
            .execute(self.connection)
            .map_err(|e| anyhow::anyhow!(e))?;

        diesel::delete(users::table)
            .filter(users::id.eq(user_id))
            .execute(self.connection)
            .map(|_| ()) // Convert DbUser to DomainUser
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn assign_roles(&mut self, user_id: i32, role_ids: &[i32]) -> anyhow::Result<usize> {
        use crate::schema::user_roles;

        diesel::delete(user_roles::table)
            .filter(user_roles::user_id.eq(user_id))
            .execute(self.connection)
            .map_err(|e| anyhow::anyhow!(e))?;

        let new_user_roles = role_ids
            .iter()
            .map(|role_id| DbNewUserRole {
                user_id,
                role_id: *role_id,
            })
            .collect::<Vec<DbNewUserRole>>();

        diesel::insert_into(user_roles::table)
            .values(&new_user_roles)
            .execute(self.connection)
            .map_err(|e| anyhow::anyhow!(e))
    }
}
