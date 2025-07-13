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

#[cfg(test)]
mod tests {
    use crate::domain::user::NewUser as DomainNewUser;
    use crate::forms::auth::RegisterForm;

    #[test]
    fn test_register_form_into_domain_new_user() {
        let form = RegisterForm {
            email: "test@example.com".to_string(),
            password: "secret".to_string(),
            hub_id: 7,
        };

        let user: DomainNewUser = (&form).into();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password, "secret");
        assert_eq!(user.hub_id, 7);
        assert_eq!(user.name, None);
    }
}
