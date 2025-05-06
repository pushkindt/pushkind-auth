use serde::Deserialize;

use crate::domain::user::NewUser as DomainNewUser;

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    pub hub_id: i32,
}

#[derive(Deserialize)]
pub struct RegisterForm {
    pub email: String,
    pub password: String,
    pub hub_id: i32,
}

impl From<RegisterForm> for DomainNewUser {
    fn from(form: RegisterForm) -> Self {
        Self {
            name: None,
            email: form.email,
            password: form.password,
            hub_id: form.hub_id,
        }
    }
}
