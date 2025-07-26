//! Abstractions over data persistence.
//!
//! Traits defined in this module describe the operations that can be performed
//! on the underlying storage. Diesel based implementations live in the
//! submodules.

pub mod errors;
pub mod hub;
pub mod menu;
pub mod role;
pub mod user;

use pushkind_common::pagination::Pagination;

use crate::domain::hub::{Hub, NewHub};
use crate::domain::menu::{Menu, NewMenu};
use crate::domain::role::{NewRole, Role};
use crate::domain::user::UserWithRoles;
use crate::domain::user::{NewUser, UpdateUser, User};
use crate::repository::errors::RepositoryResult;

#[derive(Debug, Clone)]
pub struct UserListQuery {
    pub hub_id: i32,
    pub role: Option<String>,
    pub search: Option<String>, // Only makes sense when using Search
    pub pagination: Option<Pagination>,
}

impl UserListQuery {
    pub fn new(hub_id: i32) -> Self {
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
    fn get_by_id(&self, id: i32) -> RepositoryResult<Option<UserWithRoles>>;
    fn get_by_email(&self, email: &str, hub_id: i32) -> RepositoryResult<Option<UserWithRoles>>;
    fn list(&self, query: UserListQuery) -> RepositoryResult<(usize, Vec<UserWithRoles>)>;
    fn verify_password(&self, password: &str, stored_hash: &str) -> bool;
    fn login(
        &self,
        email: &str,
        password: &str,
        hub_id: i32,
    ) -> RepositoryResult<Option<UserWithRoles>> {
        let ur = self.get_by_email(email, hub_id)?;
        if let Some(ur) = ur {
            if self.verify_password(password, &ur.user.password_hash) {
                return Ok(Some(ur));
            }
        }
        Ok(None)
    }
    fn get_roles(&self, user_id: i32) -> RepositoryResult<Vec<Role>>;
    fn search(&self, query: UserListQuery) -> RepositoryResult<(usize, Vec<UserWithRoles>)>;
}

pub trait UserWriter {
    fn create(&self, new_user: &NewUser) -> RepositoryResult<User>;
    fn assign_roles(&self, user_id: i32, role_ids: &[i32]) -> RepositoryResult<usize>;
    fn update(&self, user_id: i32, updates: &UpdateUser) -> RepositoryResult<User>;
    fn delete(&self, user_id: i32) -> RepositoryResult<usize>;
}

/// Convenience trait combining [`UserReader`] and [`UserWriter`].
pub trait UserRepository: UserReader + UserWriter {}

pub trait HubReader {
    fn get_by_id(&self, id: i32) -> RepositoryResult<Option<Hub>>;
    fn get_by_name(&self, name: &str) -> RepositoryResult<Option<Hub>>;
    fn list(&self) -> RepositoryResult<Vec<Hub>>;
}

pub trait HubWriter {
    fn create(&self, new_hub: &NewHub) -> RepositoryResult<Hub>;
    fn delete(&self, hub_id: i32) -> RepositoryResult<usize>;
}

pub trait HubRepository: HubReader + HubWriter {}
impl<T: HubReader + HubWriter> HubRepository for T {}

pub trait RoleReader {
    fn get_by_id(&self, id: i32) -> RepositoryResult<Option<Role>>;
    fn get_by_name(&self, name: &str) -> RepositoryResult<Option<Role>>;
    fn list(&self) -> RepositoryResult<Vec<Role>>;
}

pub trait RoleWriter {
    fn create(&self, new_role: &NewRole) -> RepositoryResult<Role>;
    fn delete(&self, role_id: i32) -> RepositoryResult<usize>;
}

/// Convenience trait combining [`RoleReader`] and [`RoleWriter`].
pub trait RoleRepository: RoleReader + RoleWriter {}

impl<T> RoleRepository for T where T: RoleReader + RoleWriter {}

pub trait MenuReader {
    fn list(&self, hub_id: i32) -> RepositoryResult<Vec<Menu>>;
}

pub trait MenuWriter {
    fn create(&self, new_menu: &NewMenu) -> RepositoryResult<Menu>;
    fn delete(&self, menu_id: i32) -> RepositoryResult<usize>;
}

/// Backwards compatibility alias combining [`MenuReader`] and [`MenuWriter`].
pub trait MenuRepository: MenuReader + MenuWriter {}
