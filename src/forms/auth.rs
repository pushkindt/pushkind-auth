use serde::Deserialize;

use crate::domain::user::NewUser as DomainNewUser;

#[derive(Deserialize)]
/// Form data submitted when a user logs in.
pub struct LoginForm {
    pub email: String,
    pub password: String,
    pub hub_id: i32,
}

#[derive(Deserialize)]
/// Form data used during user registration.
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
