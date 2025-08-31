use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::Utc;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Integer, Text};
use pushkind_common::repository::errors::{RepositoryError, RepositoryResult};

use crate::domain::role::Role;
use crate::domain::user::{NewUser, UpdateUser, User, UserWithRoles};
use crate::models::role::{NewUserRole as DbNewUserRole, Role as DbRole};
use crate::models::user::{NewUser as NewDbUser, UpdateUser as DbUpdateUser, User as DbUser};
use crate::repository::{DieselRepository, UserListQuery, UserReader, UserRepository, UserWriter};

impl UserReader for DieselRepository {
    fn get_user_by_id(&self, id: i32) -> RepositoryResult<Option<UserWithRoles>> {
        use crate::schema::{roles, users};

        let mut connection = self.conn()?;

        let user = users::table
            .filter(users::id.eq(id))
            .first::<DbUser>(&mut connection)
            .optional()?;

        let user = match user {
            Some(user) => user,
            None => return Ok(None),
        };

        let roles = roles::table
            .inner_join(crate::schema::user_roles::table)
            .filter(crate::schema::user_roles::user_id.eq(user.id))
            .select(crate::schema::roles::all_columns)
            .load::<DbRole>(&mut connection)?;

        let mut user: User = user.into();
        user.roles = roles.iter().map(|role| role.id).collect();

        Ok(Some(UserWithRoles {
            user,
            roles: roles.into_iter().map(|role| role.into()).collect(),
        }))
    }

    fn get_user_by_email(
        &self,
        email: &str,
        hub_id: i32,
    ) -> RepositoryResult<Option<UserWithRoles>> {
        use crate::schema::{roles, users};

        let mut connection = self.conn()?;

        let user = users::table
            .filter(users::email.eq(email))
            .filter(users::hub_id.eq(hub_id))
            .first::<DbUser>(&mut connection)
            .optional()?;
        let user = match user {
            Some(user) => user,
            None => return Ok(None),
        };

        let roles = roles::table
            .inner_join(crate::schema::user_roles::table)
            .filter(crate::schema::user_roles::user_id.eq(user.id))
            .select(crate::schema::roles::all_columns)
            .load::<DbRole>(&mut connection)?;

        let mut user: User = user.into();
        user.roles = roles.iter().map(|role| role.id).collect();

        Ok(Some(UserWithRoles {
            user,
            roles: roles.into_iter().map(|role| role.into()).collect(),
        }))
    }

    fn list_users(&self, query: UserListQuery) -> RepositoryResult<(usize, Vec<UserWithRoles>)> {
        use crate::schema::roles;
        use crate::schema::user_roles;
        use crate::schema::users;

        let mut conn = self.conn()?;

        let query_builder = || {
            let mut items = users::table
                .filter(users::hub_id.eq(query.hub_id))
                .into_boxed::<diesel::sqlite::Sqlite>();
            if let Some(role) = &query.role {
                items = items.filter(
                    users::id.eq_any(
                        user_roles::table
                            .inner_join(roles::table)
                            .filter(roles::name.eq(role))
                            .select(user_roles::user_id),
                    ),
                );
            }
            items
        };

        // Get the total count before applying pagination
        let total = query_builder().count().get_result::<i64>(&mut conn)? as usize;

        let mut items = query_builder();

        // Apply pagination if requested
        if let Some(pagination) = &query.pagination {
            let offset = ((pagination.page.max(1) - 1) * pagination.per_page) as i64;
            let limit = pagination.per_page as i64;
            items = items.offset(offset).limit(limit);
        }

        // Final load
        let users = items.order(users::id.asc()).load::<DbUser>(&mut conn)?;

        let user_ids: Vec<i32> = users.iter().map(|user| user.id).collect();

        let roles = roles::table
            .inner_join(user_roles::table)
            .filter(user_roles::user_id.eq_any(user_ids))
            .select((user_roles::user_id, roles::all_columns))
            .load::<(i32, DbRole)>(&mut conn)?;

        let user_with_roles = users
            .into_iter()
            .map(|user| {
                let user_roles: Vec<Role> = roles
                    .iter()
                    .filter(|(user_id, _)| *user_id == user.id)
                    .map(|(_, role)| role.clone().into())
                    .collect();
                let mut user: User = user.into();
                user.roles = user_roles.iter().map(|role| role.id).collect();

                UserWithRoles {
                    user,
                    roles: user_roles,
                }
            })
            .collect();

        Ok((total, user_with_roles)) // Convert DbUser to DomainUser
    }

    fn verify_password(&self, password: &str, stored_hash: &str) -> bool {
        verify(password, stored_hash).unwrap_or(false)
    }

    fn get_roles(&self, user_id: i32) -> RepositoryResult<Vec<Role>> {
        use crate::schema::roles;
        use crate::schema::user_roles;

        let mut connection = self.conn()?;

        let results = roles::table
            .inner_join(user_roles::table)
            .filter(user_roles::user_id.eq(user_id))
            .select(roles::all_columns)
            .load::<DbRole>(&mut connection)?;
        Ok(results.into_iter().map(|db_role| db_role.into()).collect())
    }

    fn search_users(&self, query: UserListQuery) -> RepositoryResult<(usize, Vec<UserWithRoles>)> {
        use crate::models::user::UserCount;
        use crate::schema::roles;
        use crate::schema::user_roles;

        let mut conn = self.conn()?;

        let match_query = match &query.search {
            None => return Ok((0, vec![])),
            Some(query) if query.is_empty() => {
                return Ok((0, vec![]));
            }
            Some(query) => {
                format!("{query}*")
            }
        };

        let mut sql = String::from(
            r#"
            SELECT users.*
            FROM users
            JOIN user_fts ON users.id = user_fts.rowid
            WHERE user_fts MATCH ?
            AND users.hub_id = ?
            "#,
        );

        if query.role.is_some() {
            let role_filter = r#"
                AND EXISTS (
                    SELECT 1 FROM user_roles ur
                    JOIN roles r ON ur.role_id = r.id
                    WHERE ur.user_id = users.id AND r.name = ?
                )
            "#;
            sql.push_str(role_filter);
        }

        let total_sql = format!("SELECT COUNT(*) as count FROM ({sql})");

        // Now add pagination to SQL (but not count)
        if query.pagination.is_some() {
            sql.push_str(" LIMIT ? OFFSET ? ");
        }

        // Build final data query
        let mut data_query = diesel::sql_query(&sql)
            .into_boxed()
            .bind::<Text, _>(&match_query)
            .bind::<Integer, _>(query.hub_id);

        let mut total_query = diesel::sql_query(&total_sql)
            .into_boxed()
            .bind::<Text, _>(&match_query)
            .bind::<Integer, _>(query.hub_id);

        if let Some(role) = &query.role {
            data_query = data_query.bind::<Text, _>(role);
            total_query = total_query.bind::<Text, _>(role);
        }

        if let Some(pagination) = &query.pagination {
            let limit = pagination.per_page as i64;
            let offset = ((pagination.page.max(1) - 1) * pagination.per_page) as i64;
            data_query = data_query
                .bind::<BigInt, _>(limit)
                .bind::<BigInt, _>(offset);
        }

        let users = data_query.load::<DbUser>(&mut conn)?;

        let total = total_query.get_result::<UserCount>(&mut conn)?.count as usize;

        let user_ids: Vec<i32> = users.iter().map(|user| user.id).collect();

        let roles = roles::table
            .inner_join(user_roles::table)
            .filter(user_roles::user_id.eq_any(user_ids))
            .select((user_roles::user_id, roles::all_columns))
            .load::<(i32, DbRole)>(&mut conn)?;

        let user_with_roles = users
            .into_iter()
            .map(|user| {
                let user_roles: Vec<Role> = roles
                    .iter()
                    .filter(|(user_id, _)| *user_id == user.id)
                    .map(|(_, role)| role.clone().into())
                    .collect();
                let mut user: User = user.into();
                user.roles = user_roles.iter().map(|role| role.id).collect();

                UserWithRoles {
                    user,
                    roles: user_roles,
                }
            })
            .collect();

        Ok((total, user_with_roles)) // Convert DbUser to DomainUser
    }
}

impl UserWriter for DieselRepository {
    fn create_user(&self, new_user: &NewUser) -> RepositoryResult<User> {
        use crate::schema::users;

        let mut connection = self.conn()?;

        let new_db_user = NewDbUser::try_from(new_user).map_err(|e| {
            RepositoryError::ValidationError(format!("Failed to saved User to DB: {e}"))
        })?;

        let user = diesel::insert_into(users::table)
            .values(&new_db_user)
            .get_result::<DbUser>(&mut connection)
            .map(|db_user| db_user.into())?;
        Ok(user)
    }

    fn update_user(&self, user_id: i32, updates: &UpdateUser) -> RepositoryResult<User> {
        use crate::schema::users;

        let mut connection = self.conn()?;

        let user = self
            .get_user_by_id(user_id)?
            .ok_or(RepositoryError::NotFound)?
            .user;

        let password_hash = match updates.password.as_ref() {
            Some(password) if !password.is_empty() => {
                hash(password, DEFAULT_COST).map_err(|e| {
                    RepositoryError::ValidationError(format!("Failed to update user password: {e}"))
                })?
            }
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
            .map(|db_user| db_user.into())?;
        Ok(user)
    }

    fn delete_user(&self, user_id: i32) -> RepositoryResult<usize> {
        use crate::schema::user_roles;
        use crate::schema::users;

        let mut connection = self.conn()?;

        let result = connection.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::delete(user_roles::table)
                .filter(user_roles::user_id.eq(user_id))
                .execute(conn)?;

            diesel::delete(users::table)
                .filter(users::id.eq(user_id))
                .execute(conn)
        })?;

        if result == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(result)
    }

    fn assign_roles_to_user(&self, user_id: i32, role_ids: &[i32]) -> RepositoryResult<usize> {
        use crate::schema::user_roles;

        let mut connection = self.conn()?;

        connection
            .transaction::<_, diesel::result::Error, _>(|conn| {
                diesel::delete(user_roles::table)
                    .filter(user_roles::user_id.eq(user_id))
                    .execute(conn)?;

                let new_user_roles = role_ids
                    .iter()
                    .map(|role_id| DbNewUserRole {
                        user_id,
                        role_id: *role_id,
                    })
                    .collect::<Vec<DbNewUserRole>>();

                diesel::insert_into(user_roles::table)
                    .values(&new_user_roles)
                    .execute(conn)
            })
            .map_err(Into::into)
    }
}

impl UserRepository for DieselRepository {}
