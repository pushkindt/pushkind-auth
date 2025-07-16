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

use crate::domain::hub::{Hub, NewHub};
use crate::domain::menu::{Menu, NewMenu};
use crate::domain::role::{NewRole, Role};
use crate::domain::user::{NewUser, UpdateUser, User};
use crate::repository::errors::RepositoryResult;

pub trait UserRepository {
    fn get_by_id(&self, id: i32) -> RepositoryResult<Option<User>>;
    fn get_by_email(&self, email: &str, hub_id: i32) -> RepositoryResult<Option<User>>;
    fn create(&self, new_user: &NewUser) -> RepositoryResult<User>;
    fn list(&self, hub_id: i32) -> RepositoryResult<Vec<(User, Vec<Role>)>>;
    fn verify_password(&self, password: &str, stored_hash: &str) -> bool;
    fn login(&self, email: &str, password: &str, hub_id: i32) -> RepositoryResult<Option<User>> {
        let user = self.get_by_email(email, hub_id)?;
        if let Some(user) = user {
            if self.verify_password(password, &user.password_hash) {
                return Ok(Some(user));
            }
        }
        Ok(None)
    }
    fn get_roles(&self, user_id: i32) -> RepositoryResult<Vec<Role>>;
    fn assign_roles(&self, user_id: i32, role_ids: &[i32]) -> RepositoryResult<usize>;
    fn update(&self, user_id: i32, updates: &UpdateUser) -> RepositoryResult<User>;
    fn delete(&self, user_id: i32) -> RepositoryResult<()>;
    fn search(&self, hub_id: i32, role: &str, query: &str) -> RepositoryResult<Vec<User>>;
}

pub trait HubRepository {
    fn get_by_id(&self, id: i32) -> RepositoryResult<Option<Hub>>;
    fn get_by_name(&self, name: &str) -> RepositoryResult<Option<Hub>>;
    fn create(&self, new_hub: &NewHub) -> RepositoryResult<Hub>;
    fn list(&self) -> RepositoryResult<Vec<Hub>>;
    fn delete(&self, hub_id: i32) -> RepositoryResult<usize>;
}

pub trait RoleRepository {
    fn get_by_id(&self, id: i32) -> RepositoryResult<Option<Role>>;
    fn get_by_name(&self, name: &str) -> RepositoryResult<Option<Role>>;
    fn create(&self, new_role: &NewRole) -> RepositoryResult<Role>;
    fn list(&self) -> RepositoryResult<Vec<Role>>;
    fn delete(&self, role_id: i32) -> RepositoryResult<usize>;
}

pub trait MenuReader {
    fn list(&self, hub_id: i32) -> RepositoryResult<Vec<Menu>>;
}

pub trait MenuWriter {
    fn create(&self, new_menu: &NewMenu) -> RepositoryResult<Menu>;
    fn delete(&self, menu_id: i32) -> RepositoryResult<usize>;
}

/// Backwards compatibility alias combining [`MenuReader`] and [`MenuWriter`].
pub trait MenuRepository: MenuReader + MenuWriter {}
