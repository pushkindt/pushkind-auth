//! Authentication-related request payloads.
//!
//! These types validate login, registration, and password recovery inputs
//! before they are transformed into domain types.
use serde::Deserialize;
use validator::Validate;

use crate::domain::types::{HubId, TypeConstraintError, UserEmail};
use crate::domain::user::NewUser as DomainNewUser;

#[derive(Deserialize, Validate)]
/// Form data submitted when a user logs in.
pub struct LoginForm {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    #[validate(range(min = 1))]
    pub hub_id: i32,
}

#[derive(Deserialize, Validate)]
/// Form data used during user registration.
pub struct RegisterForm {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    #[validate(range(min = 1))]
    pub hub_id: i32,
}

#[derive(Deserialize, Validate)]
/// Form data used to recover a forgotten password.
pub struct RecoverForm {
    #[validate(email)]
    pub email: String,
    #[validate(range(min = 1))]
    pub hub_id: i32,
}

impl RegisterForm {
    pub fn into_domain(self) -> Result<DomainNewUser, TypeConstraintError> {
        Ok(DomainNewUser::new(
            UserEmail::new(self.email)?,
            None,
            HubId::new(self.hub_id)?,
            self.password,
        ))
    }
}

impl TryFrom<RegisterForm> for DomainNewUser {
    type Error = TypeConstraintError;

    fn try_from(form: RegisterForm) -> Result<Self, Self::Error> {
        form.into_domain()
    }
}

impl LoginForm {
    pub fn email(&self) -> Result<UserEmail, TypeConstraintError> {
        UserEmail::new(&self.email)
    }

    pub fn hub_id(&self) -> Result<HubId, TypeConstraintError> {
        HubId::new(self.hub_id)
    }
}

impl RecoverForm {
    pub fn email(&self) -> Result<UserEmail, TypeConstraintError> {
        UserEmail::new(&self.email)
    }

    pub fn hub_id(&self) -> Result<HubId, TypeConstraintError> {
        HubId::new(self.hub_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::types::{HubId, UserEmail};
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

        let user: DomainNewUser = form.into_domain().expect("conversion failed");

        assert_eq!(user.email, UserEmail::new("test@example.com").unwrap());
        assert_eq!(user.password, "secret");
        assert_eq!(user.hub_id, HubId::new(7).unwrap());
        assert_eq!(user.name, None);
    }

    #[test]
    fn test_register_form_normalizes_email_case() {
        let form = RegisterForm {
            email: "Test@Example.COM".to_string(),
            password: "secret".to_string(),
            hub_id: 3,
        };

        let user: DomainNewUser = form.into_domain().expect("conversion failed");

        assert_eq!(user.email.as_str(), "test@example.com");
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
