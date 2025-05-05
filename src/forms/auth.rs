use bcrypt::{DEFAULT_COST, hash, verify};
use serde::Deserialize;

use crate::domain::user::NewUser as DomainNewUser;

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    pub hub_id: i32,
}

impl LoginForm {
    pub fn verify_password(&self, password_hash: &str) -> bool {
        verify(self.password.as_str(), password_hash).unwrap_or(false)
    }
}

#[derive(Deserialize)]
pub struct RegisterForm {
    pub email: String,
    pub password: String,
    pub hub_id: i32,
}

impl TryFrom<RegisterForm> for DomainNewUser {
    type Error = anyhow::Error;

    fn try_from(form: RegisterForm) -> Result<Self, Self::Error> {
        let password_hash = hash(form.password, DEFAULT_COST)?;

        Ok(Self {
            name: None,
            email: form.email,
            password_hash,
            hub_id: form.hub_id,
        })
    }
}
