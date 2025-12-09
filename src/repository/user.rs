//! Diesel-backed [`UserRepository`](crate::repository::UserRepository) implementation.
//!
//! Functions in this module translate Diesel models into domain types, wrap
//! mutations in transactions, and handle shared concerns like password hashing
//! and full-text search filtering.

use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::Utc;
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Text};
use pushkind_common::repository::build_fts_match_query;
use pushkind_common::repository::errors::{RepositoryError, RepositoryResult};

use crate::domain::role::Role;
use crate::domain::types::{HubId, RoleId, UserEmail, UserId};
use crate::domain::user::{NewUser, UpdateUser, User, UserWithRoles};
use crate::models::role::{NewUserRole as DbNewUserRole, Role as DbRole};
use crate::models::user::{NewUser as NewDbUser, UpdateUser as DbUpdateUser, User as DbUser};
use crate::repository::{DieselRepository, UserListQuery, UserReader, UserRepository, UserWriter};

impl UserReader for DieselRepository {
    fn get_user_by_id(&self, id: UserId, hub_id: HubId) -> RepositoryResult<Option<UserWithRoles>> {
        use crate::schema::{roles, users};

        let mut connection = self.conn()?;

        connection.transaction::<_, RepositoryError, _>(|conn| {
            let user = users::table
                .filter(users::id.eq(id.get()))
                .filter(users::hub_id.eq(hub_id.get()))
                .first::<DbUser>(conn)
                .optional()?;

            let user = match user {
                Some(user) => user,
                None => return Ok(None),
            };

            let roles = roles::table
                .inner_join(crate::schema::user_roles::table)
                .filter(crate::schema::user_roles::user_id.eq(user.id))
                .select(crate::schema::roles::all_columns)
                .load::<DbRole>(conn)?;

            let roles = roles
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<Role>, _>>()?;

            let mut user: User = user.try_into()?;
            user.roles = roles.iter().map(|role| role.id).collect();

            Ok(Some(UserWithRoles { user, roles }))
        })
    }

    fn get_user_by_email(
        &self,
        email: &UserEmail,
        hub_id: HubId,
    ) -> RepositoryResult<Option<UserWithRoles>> {
        use crate::schema::{roles, users};

        let mut connection = self.conn()?;

        connection.transaction::<_, RepositoryError, _>(|conn| {
            let user = users::table
                .filter(users::email.eq(email.as_str()))
                .filter(users::hub_id.eq(hub_id.get()))
                .first::<DbUser>(conn)
                .optional()?;
            let user = match user {
                Some(user) => user,
                None => return Ok(None),
            };

            let roles = roles::table
                .inner_join(crate::schema::user_roles::table)
                .filter(crate::schema::user_roles::user_id.eq(user.id))
                .select(crate::schema::roles::all_columns)
                .load::<DbRole>(conn)?;

            let roles = roles
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<Role>, _>>()?;

            let mut user: User = user.try_into()?;
            user.roles = roles.iter().map(|role| role.id).collect();

            Ok(Some(UserWithRoles { user, roles }))
        })
    }

    fn list_users(&self, query: UserListQuery) -> RepositoryResult<(usize, Vec<UserWithRoles>)> {
        use crate::schema::roles;
        use crate::schema::user_fts;
        use crate::schema::user_roles;
        use crate::schema::users;

        let mut conn = self.conn()?;

        conn.transaction::<_, RepositoryError, _>(|conn| {
            // Build a boxed query with optional role and full-text filters so the
            // same definition can be reused for total counting and pagination.
            let query_builder = || {
                let mut items = users::table
                    .filter(users::hub_id.eq(query.hub_id.get()))
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
                if let Some(term) = query.search.as_ref()
                    && let Some(fts_query) = build_fts_match_query(term)
                {
                    let fts_filter = exists(
                        user_fts::table
                            .filter(user_fts::rowid.eq(users::id))
                            .filter(
                                diesel::dsl::sql::<Bool>("user_fts MATCH ")
                                    .bind::<Text, _>(fts_query),
                            ),
                    );
                    items = items.filter(fts_filter);
                }
                items
            };

            // Get the total count before applying pagination
            let total = query_builder().count().get_result::<i64>(conn)? as usize;

            let mut items = query_builder();

            // Apply pagination if requested
            if let Some(pagination) = &query.pagination {
                let offset = ((pagination.page.max(1) - 1) * pagination.per_page) as i64;
                let limit = pagination.per_page as i64;
                items = items.offset(offset).limit(limit);
            }

            // Final load
            let users = items.order(users::id.asc()).load::<DbUser>(conn)?;

            let user_ids: Vec<i32> = users.iter().map(|user| user.id).collect::<Vec<_>>();

            let roles = roles::table
                .inner_join(user_roles::table)
                .filter(user_roles::user_id.eq_any(user_ids.clone()))
                .select((user_roles::user_id, roles::all_columns))
                .load::<(i32, DbRole)>(conn)?
                .into_iter()
                .map(|(user_id, role)| {
                    let role: Role = role.try_into()?;
                    Ok((user_id, role))
                })
                .collect::<RepositoryResult<Vec<(i32, Role)>>>()?;

            let user_with_roles = users
                .into_iter()
                .map(|user| {
                    let user_roles: Vec<Role> = roles
                        .iter()
                        .filter(|(user_id, _)| *user_id == user.id)
                        .map(|(_, role)| role.clone())
                        .collect();
                    let mut user: User = user.try_into()?;
                    user.roles = user_roles.iter().map(|role| role.id).collect();

                    Ok(UserWithRoles {
                        user,
                        roles: user_roles,
                    })
                })
                .collect::<RepositoryResult<Vec<_>>>()?;

            Ok((total, user_with_roles))
        })
    }

    fn verify_password(&self, password: &str, stored_hash: &str) -> bool {
        verify(password, stored_hash).unwrap_or(false)
    }

    fn get_roles(&self, user_id: UserId) -> RepositoryResult<Vec<Role>> {
        use crate::schema::roles;
        use crate::schema::user_roles;

        let mut connection = self.conn()?;

        let results = roles::table
            .inner_join(user_roles::table)
            .filter(user_roles::user_id.eq(user_id.get()))
            .select(roles::all_columns)
            .load::<DbRole>(&mut connection)?;
        let roles = results
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<Role>, _>>()?;
        Ok(roles)
    }
}

impl UserWriter for DieselRepository {
    fn create_user(&self, new_user: &NewUser) -> RepositoryResult<User> {
        use crate::schema::users;

        let mut connection = self.conn()?;

        let new_db_user = NewDbUser::try_from(new_user).map_err(|e| {
            RepositoryError::ValidationError(format!("Failed to saved User to DB: {e}"))
        })?;

        // Persist the validated input and convert the Diesel model into the
        // domain representation so callers never handle database types.
        let user = diesel::insert_into(users::table)
            .values(&new_db_user)
            .get_result::<DbUser>(&mut connection)?;
        let user = user.try_into()?;
        Ok(user)
    }

    fn update_user(
        &self,
        user_id: UserId,
        hub_id: HubId,
        updates: &UpdateUser,
    ) -> RepositoryResult<User> {
        use crate::schema::users;

        let mut connection = self.conn()?;

        connection.transaction::<_, RepositoryError, _>(|conn| {
            let user = users::table
                .filter(users::id.eq(user_id.get()))
                .filter(users::hub_id.eq(hub_id.get()))
                .first::<DbUser>(conn)
                .optional()?
                .ok_or(RepositoryError::NotFound)?;

            let password_hash = match updates.password.as_ref() {
                Some(password) if !password.is_empty() => {
                    hash(password, DEFAULT_COST).map_err(|e| {
                        RepositoryError::ValidationError(format!(
                            "Failed to update user password: {e}"
                        ))
                    })?
                }
                _ => user.password_hash,
            };

            let db_updates = DbUpdateUser {
                name: updates.name.as_str(),
                password_hash,
                updated_at: Utc::now().naive_utc(),
            };

            // Perform the mutation in a single statement so `updated_at` stays in
            // sync with the modified fields.
            let user = diesel::update(users::table)
                .filter(users::id.eq(user_id.get()))
                .set(&db_updates)
                .get_result::<DbUser>(conn)?;

            let user = user.try_into()?;
            Ok(user)
        })
    }

    fn delete_user(&self, user_id: UserId) -> RepositoryResult<usize> {
        use crate::schema::user_roles;
        use crate::schema::users;

        let mut connection = self.conn()?;

        // Delete role mappings and the user record inside a single
        // transaction so referential integrity remains consistent even when
        // cascading deletes occur.
        let result = connection.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::delete(user_roles::table)
                .filter(user_roles::user_id.eq(user_id.get()))
                .execute(conn)?;

            diesel::delete(users::table)
                .filter(users::id.eq(user_id.get()))
                .execute(conn)
        })?;

        if result == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(result)
    }

    fn assign_roles_to_user(
        &self,
        user_id: UserId,
        role_ids: &[RoleId],
    ) -> RepositoryResult<usize> {
        use crate::schema::user_roles;

        let mut connection = self.conn()?;

        connection
            .transaction::<_, diesel::result::Error, _>(|conn| {
                // Remove existing mappings first; failures during insertion
                // will roll back this deletion so the previous role set
                // remains intact.
                diesel::delete(user_roles::table)
                    .filter(user_roles::user_id.eq(user_id.get()))
                    .execute(conn)?;

                let new_user_roles = role_ids
                    .iter()
                    .map(|role_id| DbNewUserRole {
                        user_id: user_id.get(),
                        role_id: role_id.get(),
                    })
                    .collect::<Vec<DbNewUserRole>>();

                diesel::insert_into(user_roles::table)
                    .values(&new_user_roles)
                    .execute(conn)
            })
            .map_err(RepositoryError::from)
    }
}

impl UserRepository for DieselRepository {}
