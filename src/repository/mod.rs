pub mod hub;
pub mod role;
pub mod user;

use thiserror::Error;

use crate::domain::hub::{Hub, NewHub};
use crate::domain::role::{NewRole, Role};
use crate::domain::user::{NewUser, UpdateUser, User};

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Entity not found")]
    NotFound,

    #[error("Database error: {0}")]
    DatabaseError(#[from] anyhow::Error),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

pub trait UserRepository {
    fn get_by_id(&self, id: i32) -> anyhow::Result<Option<User>>;
    fn get_by_email(&self, email: &str, hub_id: i32) -> anyhow::Result<Option<User>>;
    fn create(&self, new_user: &NewUser) -> anyhow::Result<User>;
    fn list(&self, hub_id: i32) -> anyhow::Result<Vec<(User, Vec<Role>)>>;
    fn verify_password(&self, password: &str, stored_hash: &str) -> bool;
    fn login(&self, email: &str, password: &str, hub_id: i32) -> anyhow::Result<Option<User>> {
        let user = self.get_by_email(email, hub_id)?;
        if let Some(user) = user {
            if self.verify_password(password, &user.password_hash) {
                return Ok(Some(user));
            }
        }
        Ok(None)
    }
    fn get_roles(&self, user_id: i32) -> anyhow::Result<Vec<Role>>;
    fn assign_roles(&self, user_id: i32, role_ids: &[i32]) -> anyhow::Result<usize>;
    fn update(&self, user_id: i32, updates: &UpdateUser) -> anyhow::Result<User>;
    fn delete(&self, user_id: i32) -> anyhow::Result<()>;
}

pub trait HubRepository {
    fn get_by_id(&self, id: i32) -> anyhow::Result<Option<Hub>>;
    fn get_by_name(&self, name: &str) -> anyhow::Result<Option<Hub>>;
    fn create(&self, new_hub: &NewHub) -> anyhow::Result<Hub>;
    fn list(&self) -> anyhow::Result<Vec<Hub>>;
    fn delete(&self, hub_id: i32) -> anyhow::Result<usize>;
}

pub trait RoleRepository {
    fn get_by_id(&self, id: i32) -> anyhow::Result<Option<Role>>;
    fn get_by_name(&self, name: &str) -> anyhow::Result<Option<Role>>;
    fn create(&self, new_role: &NewRole) -> anyhow::Result<Role>;
    fn list(&self) -> anyhow::Result<Vec<Role>>;
    fn delete(&self, role_id: i32) -> anyhow::Result<usize>;
}
