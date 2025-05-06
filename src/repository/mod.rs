pub mod hub;
pub mod role;
pub mod user;

use crate::domain::hub::{Hub, NewHub};
use crate::domain::role::{NewRole, Role};
use crate::domain::user::{NewUser, User};

pub trait UserRepository {
    fn get_by_id(&mut self, id: i32) -> anyhow::Result<Option<User>>;
    fn get_by_email(&mut self, email: &str, hub_id: i32) -> anyhow::Result<Option<User>>;
    fn create(&mut self, new_user: &NewUser) -> anyhow::Result<User>;
    fn list(&mut self) -> anyhow::Result<Vec<User>>;
    fn verify_password(&self, password: &str, stored_hash: &str) -> bool;
    fn login(&mut self, email: &str, password: &str, hub_id: i32) -> anyhow::Result<Option<User>> {
        let user = self.get_by_email(email, hub_id)?;
        if let Some(user) = user {
            if self.verify_password(password, &user.password_hash) {
                return Ok(Some(user));
            }
        }
        Ok(None)
    }
}

pub trait HubRepository {
    fn get_by_id(&mut self, id: i32) -> anyhow::Result<Option<Hub>>;
    fn get_by_name(&mut self, name: &str) -> anyhow::Result<Option<Hub>>;
    fn create(&mut self, new_hub: &NewHub) -> anyhow::Result<Hub>;
    fn list(&mut self) -> anyhow::Result<Vec<Hub>>;
}

pub trait RoleRepository {
    fn get_by_id(&mut self, id: i32) -> anyhow::Result<Option<Role>>;
    fn get_by_name(&mut self, name: &str) -> anyhow::Result<Option<Role>>;
    fn create(&mut self, new_role: &NewRole) -> anyhow::Result<Role>;
    fn list(&mut self) -> anyhow::Result<Vec<Role>>;
}
