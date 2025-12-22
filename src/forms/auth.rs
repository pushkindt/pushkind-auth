//! Authentication-related request payloads.
//!
//! These types validate login, registration, and password recovery inputs
//! before they are transformed into domain types.
use serde::Deserialize;
use validator::Validate;

use crate::domain::types::{HubId, UserEmail, UserPassword};
use crate::domain::user::NewUser as DomainNewUser;
use crate::forms::FormError;

#[derive(Deserialize, Validate, Clone)]
/// Form data submitted when a user logs in.
pub struct LoginForm {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    #[validate(range(min = 1))]
    pub hub_id: i32,
}

// Payload after validation and conversion to domain types.
pub struct LoginPayload {
    pub email: UserEmail,
    pub password: UserPassword,
    pub hub_id: HubId,
}

#[derive(Deserialize, Validate, Clone)]
/// Form data used during user registration.
pub struct RegisterForm {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    #[validate(range(min = 1))]
    pub hub_id: i32,
}

// Payload after validation and conversion to domain types.
pub struct RegisterPayload {
    pub email: UserEmail,
    pub password: UserPassword,
    pub hub_id: HubId,
}

#[derive(Deserialize, Validate, Clone)]
/// Form data used to recover a forgotten password.
pub struct RecoverForm {
    #[validate(email)]
    pub email: String,
    #[validate(range(min = 1))]
    pub hub_id: i32,
}

// Payload after validation and conversion to domain types.
pub struct RecoverPayload {
    pub email: UserEmail,
    pub hub_id: HubId,
}

impl TryFrom<LoginForm> for LoginPayload {
    type Error = FormError;

    fn try_from(form: LoginForm) -> Result<Self, Self::Error> {
        form.validate().map_err(FormError::Validation)?;
        Ok(Self {
            email: UserEmail::new(form.email).map_err(|_| FormError::InvalidEmail)?,
            password: UserPassword::new(form.password).map_err(|_| FormError::InvalidPassword)?,
            hub_id: HubId::new(form.hub_id).map_err(|_| FormError::InvalidHubId)?,
        })
    }
}

impl TryFrom<RegisterForm> for RegisterPayload {
    type Error = FormError;

    fn try_from(form: RegisterForm) -> Result<Self, Self::Error> {
        form.validate().map_err(FormError::Validation)?;
        Ok(Self {
            email: UserEmail::new(form.email).map_err(|_| FormError::InvalidEmail)?,
            password: UserPassword::new(form.password).map_err(|_| FormError::InvalidPassword)?,
            hub_id: HubId::new(form.hub_id).map_err(|_| FormError::InvalidHubId)?,
        })
    }
}

impl TryFrom<RecoverForm> for RecoverPayload {
    type Error = FormError;

    fn try_from(form: RecoverForm) -> Result<Self, Self::Error> {
        form.validate().map_err(FormError::Validation)?;
        Ok(Self {
            email: UserEmail::new(form.email).map_err(|_| FormError::InvalidEmail)?,
            hub_id: HubId::new(form.hub_id).map_err(|_| FormError::InvalidHubId)?,
        })
    }
}

impl From<RegisterPayload> for DomainNewUser {
    fn from(payload: RegisterPayload) -> Self {
        Self::new(payload.email, None, payload.hub_id, payload.password)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::types::{HubId, UserEmail, UserPassword};
    use crate::domain::user::NewUser as DomainNewUser;
    use crate::forms::auth::{RegisterForm, RegisterPayload};
    use validator::Validate;

    #[test]
    fn test_register_form_into_domain_new_user() {
        let form = RegisterForm {
            email: "test@example.com".to_string(),
            password: "secret".to_string(),
            hub_id: 7,
        };

        let payload: RegisterPayload = form.try_into().expect("conversion failed");

        let user: DomainNewUser = payload.into();

        assert_eq!(user.email, UserEmail::new("test@example.com").unwrap());
        assert_eq!(user.password, UserPassword::new("secret").unwrap());
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

        let payload: RegisterPayload = form.try_into().expect("conversion failed");

        let user: DomainNewUser = payload.into();

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
