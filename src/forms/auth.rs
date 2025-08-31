use serde::Deserialize;
use validator::Validate;

use crate::domain::user::NewUser as DomainNewUser;

#[derive(Deserialize, Validate)]
/// Form data submitted when a user logs in.
pub struct LoginForm {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    pub hub_id: i32,
}

#[derive(Deserialize, Validate)]
/// Form data used during user registration.
pub struct RegisterForm {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    pub hub_id: i32,
}

impl<'a> From<RegisterForm> for DomainNewUser {
    fn from(form: RegisterForm) -> Self {
        DomainNewUser::new(form.email, None, form.hub_id, form.password)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::user::NewUser as DomainNewUser;
    use crate::forms::auth::RegisterForm;
    use validator::Validate;

    #[test]
    fn test_register_form_into_domain_new_user() {
        let form = RegisterForm {
            email: "test@example.com".to_string(),
            password: "secret".to_string(),
            hub_id: 7,
        };

        let user: DomainNewUser = form.into();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password, "secret");
        assert_eq!(user.hub_id, 7);
        assert_eq!(user.name, None);
    }

    #[test]
    fn test_register_form_email_validation() {
        let form = RegisterForm {
            email: "test".to_string(),
            password: "secret".to_string(),
            hub_id: 7,
        };
        assert!(form.validate().is_err())
    }

    #[test]
    fn test_register_form_password_validation() {
        let form = RegisterForm {
            email: "test@example.com".to_string(),
            password: "".to_string(),
            hub_id: 7,
        };
        assert!(form.validate().is_err())
    }
}
