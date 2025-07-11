use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::Utc;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Integer, Text};

use crate::db::DbPool;
use crate::domain::role::Role;
use crate::domain::user::{NewUser, UpdateUser, User};
use crate::models::role::{NewUserRole as DbNewUserRole, Role as DbRole};
use crate::models::user::{NewUser as NewDbUser, UpdateUser as DbUpdateUser, User as DbUser};
use crate::repository::UserRepository;
use crate::repository::errors::{RepositoryError, RepositoryResult};

/// Diesel implementation of [`UserRepository`].
pub struct DieselUserRepository<'a> {
    pub pool: &'a DbPool,
}

impl<'a> DieselUserRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }
}

impl UserRepository for DieselUserRepository<'_> {
    fn get_by_id(&self, id: i32) -> RepositoryResult<Option<User>> {
        use crate::schema::users;

        let mut connection = self.pool.get()?;

        let result = users::table
            .filter(users::id.eq(id))
            .first::<DbUser>(&mut connection)
            .optional()?;

        Ok(result.map(|db_user| db_user.into())) // Convert DbUser to DomainUser
    }

    fn get_by_email(&self, email: &str, hub_id: i32) -> RepositoryResult<Option<User>> {
        use crate::schema::users;

        let mut connection = self.pool.get()?;

        let email = email.to_lowercase();

        let result = users::table
            .filter(users::email.eq(&email))
            .filter(users::hub_id.eq(hub_id))
            .first::<DbUser>(&mut connection)
            .optional()?;

        Ok(result.map(|db_user| db_user.into())) // Convert DbUser to DomainUser
    }

    fn create(&self, new_user: NewUser) -> RepositoryResult<User> {
        use crate::schema::users;

        let mut connection = self.pool.get()?;

        let new_db_user = NewDbUser::try_from(new_user)?; // Convert to DbNewUser

        let user = diesel::insert_into(users::table)
            .values(&new_db_user)
            .get_result::<DbUser>(&mut connection)
            .map(|db_user| db_user.into())?; // Convert DbUser to DomainUser
        Ok(user)
    }

    fn list(&self, hub_id: i32) -> RepositoryResult<Vec<(User, Vec<Role>)>> {
        use crate::schema::roles;
        use crate::schema::user_roles;
        use crate::schema::users;

        let mut connection = self.pool.get()?;

        let users = users::table
            .filter(users::hub_id.eq(hub_id))
            .load::<DbUser>(&mut connection)?;

        let user_ids: Vec<i32> = users.iter().map(|user| user.id).collect();

        let roles = roles::table
            .inner_join(user_roles::table)
            .filter(user_roles::user_id.eq_any(user_ids))
            .select((user_roles::user_id, roles::all_columns))
            .load::<(i32, DbRole)>(&mut connection)?;

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

    fn get_roles(&self, user_id: i32) -> RepositoryResult<Vec<Role>> {
        use crate::schema::roles;
        use crate::schema::user_roles;

        let mut connection = self.pool.get()?;

        let results = roles::table
            .inner_join(user_roles::table)
            .filter(user_roles::user_id.eq(user_id))
            .select(roles::all_columns)
            .load::<DbRole>(&mut connection)?;
        Ok(results.into_iter().map(|db_role| db_role.into()).collect())
    }

    fn update(&self, user_id: i32, updates: UpdateUser) -> RepositoryResult<User> {
        use crate::schema::users;

        let mut connection = self.pool.get()?;

        let user = self.get_by_id(user_id)?.ok_or(RepositoryError::NotFound)?;

        let password_hash = match updates.password.as_ref() {
            Some(password) if !password.is_empty() => hash(password, DEFAULT_COST)?,
            _ => user.password_hash,
        };

        let db_updates = DbUpdateUser {
            name: updates.name,
            password_hash,
            updated_at: Utc::now().naive_utc(),
        };

        let user = diesel::update(users::table)
            .filter(users::id.eq(user_id))
            .set(&db_updates)
            .get_result::<DbUser>(&mut connection)
            .map(|db_user| db_user.into())?; // Convert DbUser to DomainUser
        Ok(user)
    }

    fn delete(&self, user_id: i32) -> RepositoryResult<()> {
        use crate::schema::user_roles;
        use crate::schema::users;

        let mut connection = self.pool.get()?;

        diesel::delete(user_roles::table)
            .filter(user_roles::user_id.eq(user_id))
            .execute(&mut connection)?;

        let result = diesel::delete(users::table)
            .filter(users::id.eq(user_id))
            .execute(&mut connection)?;

        if result == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    fn assign_roles(&self, user_id: i32, role_ids: &[i32]) -> RepositoryResult<usize> {
        use crate::schema::user_roles;

        let mut connection = self.pool.get()?;

        diesel::delete(user_roles::table)
            .filter(user_roles::user_id.eq(user_id))
            .execute(&mut connection)?;

        let new_user_roles = role_ids
            .iter()
            .map(|role_id| DbNewUserRole {
                user_id,
                role_id: *role_id,
            })
            .collect::<Vec<DbNewUserRole>>();

        let result = diesel::insert_into(user_roles::table)
            .values(&new_user_roles)
            .execute(&mut connection)?;
        Ok(result)
    }

    fn search(&self, hub_id: i32, role: &str, query: &str) -> RepositoryResult<Vec<User>> {
        let mut connection = self.pool.get()?;

        let match_query = format!("{}*", query.to_lowercase());

        let results = sql_query(
            r#"
            SELECT users.*
            FROM users
            JOIN user_fts ON users.id = user_fts.rowid
            WHERE user_fts MATCH ?
            AND users.hub_id = ?
            AND EXISTS (
                SELECT 1 FROM user_roles ur
                JOIN roles r ON ur.role_id = r.id
                WHERE ur.user_id = users.id AND r.name = ?
            )
            "#,
        )
        .bind::<Text, _>(&match_query)
        .bind::<Integer, _>(hub_id)
        .bind::<Text, _>(role)
        .load::<DbUser>(&mut connection)?;

        Ok(results.into_iter().map(User::from).collect())
    }
}
