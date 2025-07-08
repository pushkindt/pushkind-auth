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

impl<'a> From<&'a RegisterForm> for DomainNewUser<'a> {
    fn from(form: &'a RegisterForm) -> Self {
        DomainNewUser {
            email: form.email.as_str(),
            name: None,
            hub_id: form.hub_id,
            password: form.password.as_str(),
        }
    }
}
