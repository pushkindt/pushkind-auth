//! Abstractions over data persistence.
//!
//! Traits defined in this module describe the operations that can be performed
//! on the underlying storage. Diesel based implementations live in the
//! submodules.

use pushkind_common::db::{DbConnection, DbPool};
use pushkind_common::pagination::Pagination;
use pushkind_common::repository::errors::RepositoryResult;

use crate::domain::hub::{Hub, NewHub};
use crate::domain::menu::{Menu, NewMenu};
use crate::domain::role::{NewRole, Role};
use crate::domain::types::{HubId, MenuId, RoleId, TypeConstraintError, UserEmail, UserId};
use crate::domain::user::UserWithRoles;
use crate::domain::user::{NewUser, UpdateUser, User};

pub mod hub;
pub mod menu;
#[cfg(test)]
pub mod mock;
pub mod role;
pub mod user;

#[derive(Clone)]
pub struct DieselRepository {
    pool: DbPool, // r2d2::Pool is cheap to clone
}

impl DieselRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conn(&self) -> RepositoryResult<DbConnection> {
        Ok(self.pool.get()?)
    }
}

/// Parameters used when querying for a list of users.
#[derive(Debug, Clone)]
pub struct UserListQuery {
    /// Identifier of the hub to which the users belong.
    pub hub_id: HubId,
    /// Optional role to filter the resulting users by.
    pub role: Option<String>,
    /// Text term used when performing search queries.
    pub search: Option<String>,
    /// Pagination information for limiting results.
    pub pagination: Option<Pagination>,
}

impl UserListQuery {
    pub fn new(hub_id: HubId) -> Self {
        Self {
            hub_id,
            role: None,
            search: None,
            pagination: None,
        }
    }

    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    pub fn paginate(mut self, page: usize, per_page: usize) -> Self {
        self.pagination = Some(Pagination { page, per_page });
        self
    }
}

pub trait UserReader {
    fn get_user_by_id(&self, id: UserId, hub_id: HubId) -> RepositoryResult<Option<UserWithRoles>>;
    fn get_user_by_email(
        &self,
        email: &UserEmail,
        hub_id: HubId,
    ) -> RepositoryResult<Option<UserWithRoles>>;
    fn list_users(&self, query: UserListQuery) -> RepositoryResult<(usize, Vec<UserWithRoles>)>;
    fn verify_password(&self, password: &str, stored_hash: &str) -> bool;
    /// Attempts to authenticate a user by email, password and hub.
    ///
    /// Returns the full user with roles on success, or [`None`] when
    /// authentication fails.
    fn login(
        &self,
        email: &UserEmail,
        password: &str,
        hub_id: HubId,
    ) -> RepositoryResult<Option<UserWithRoles>> {
        let user = self.get_user_by_email(email, hub_id)?;
        if let Some(ur) = user
            && self.verify_password(password, &ur.user.password_hash)
        {
            return Ok(Some(ur));
        }
        Ok(None)
    }
    fn get_roles(&self, user_id: UserId) -> RepositoryResult<Vec<Role>>;
}

pub trait UserWriter {
    fn create_user(&self, new_user: &NewUser) -> RepositoryResult<User>;
    fn assign_roles_to_user(&self, user_id: UserId, role_ids: &[RoleId])
    -> RepositoryResult<usize>;
    fn update_user(
        &self,
        user_id: UserId,
        hub_id: HubId,
        updates: &UpdateUser,
    ) -> RepositoryResult<User>;
    fn delete_user(&self, user_id: UserId) -> RepositoryResult<usize>;
}

/// Convenience trait combining [`UserReader`] and [`UserWriter`].
pub trait UserRepository: UserReader + UserWriter {}

pub trait HubReader {
    fn get_hub_by_id(&self, id: HubId) -> RepositoryResult<Option<Hub>>;
    fn get_hub_by_name(&self, name: &str) -> RepositoryResult<Option<Hub>>;
    fn list_hubs(&self) -> RepositoryResult<Vec<Hub>>;
}

pub trait HubWriter {
    fn create_hub(&self, new_hub: &NewHub) -> RepositoryResult<Hub>;
    fn delete_hub(&self, hub_id: HubId) -> RepositoryResult<usize>;
}

pub trait HubRepository: HubReader + HubWriter {}
impl<T: HubReader + HubWriter> HubRepository for T {}

pub trait RoleReader {
    fn get_role_by_id(&self, id: RoleId) -> RepositoryResult<Option<Role>>;
    fn get_role_by_name(&self, name: &str) -> RepositoryResult<Option<Role>>;
    fn list_roles(&self) -> RepositoryResult<Vec<Role>>;
}

pub trait RoleWriter {
    fn create_role(&self, new_role: &NewRole) -> RepositoryResult<Role>;
    fn delete_role(&self, role_id: RoleId) -> RepositoryResult<usize>;
}

/// Convenience trait combining [`RoleReader`] and [`RoleWriter`].
pub trait RoleRepository: RoleReader + RoleWriter {}

impl<T> RoleRepository for T where T: RoleReader + RoleWriter {}

pub trait MenuReader {
    fn get_menu_by_id(&self, menu_id: MenuId, hub_id: HubId) -> RepositoryResult<Option<Menu>>;
    fn list_menu(&self, hub_id: HubId) -> RepositoryResult<Vec<Menu>>;
}

pub trait MenuWriter {
    fn create_menu(&self, new_menu: &NewMenu) -> RepositoryResult<Menu>;
    fn delete_menu(&self, menu_id: MenuId) -> RepositoryResult<usize>;
}

/// Backwards compatibility alias combining [`MenuReader`] and [`MenuWriter`].
pub trait MenuRepository: MenuReader + MenuWriter {}

pub(crate) fn map_type_error(
    err: TypeConstraintError,
) -> pushkind_common::repository::errors::RepositoryError {
    pushkind_common::repository::errors::RepositoryError::ValidationError(err.to_string())
}
