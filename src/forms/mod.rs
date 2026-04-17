//! Request form definitions used by Actix handlers.
//!
//! These structs validate incoming payloads and translate them into domain
//! types so the service layer can remain framework-agnostic.

use std::borrow::Cow;

use pushkind_common::dto::mutation::{ApiFieldErrorDto, ApiMutationErrorDto};
use thiserror::Error;
use validator::{ValidationError, ValidationErrors};

pub mod auth;
pub mod main;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Field-level validation error emitted by the form layer.
pub struct FormFieldError {
    pub field: Cow<'static, str>,
    pub message: Cow<'static, str>,
}

#[derive(Debug, Error)]
/// Errors that can occur when processing form data.
pub enum FormError {
    #[error("{}", validation_errors_display(.0))]
    Validation(#[from] ValidationErrors),

    #[error("Укажите корректный электронный адрес.")]
    InvalidEmail,

    #[error("Пароль заполнен некорректно.")]
    InvalidPassword,

    #[error("Выберите хаб.")]
    InvalidHubId,

    #[error("Укажите имя.")]
    InvalidName,

    #[error("Укажите корректный URL.")]
    InvalidUrl,

    #[error("Роль заполнена некорректно.")]
    InvalidRoleId,
}

impl FormError {
    pub(crate) fn field_errors(&self) -> Vec<FormFieldError> {
        match self {
            Self::Validation(errors) => collect_validation_errors(errors),
            _ => self
                .field()
                .map(|field| vec![field_error(field, self.to_string())])
                .unwrap_or_default(),
        }
    }

    fn field(&self) -> Option<&'static str> {
        match self {
            Self::Validation(_) => None,
            Self::InvalidEmail => Some("email"),
            Self::InvalidPassword => Some("password"),
            Self::InvalidHubId => Some("hub_id"),
            Self::InvalidName => Some("name"),
            Self::InvalidUrl => Some("url"),
            Self::InvalidRoleId => Some("roles"),
        }
    }
}

fn collect_validation_errors(errors: &ValidationErrors) -> Vec<FormFieldError> {
    errors
        .field_errors()
        .iter()
        .flat_map(|(field, field_errors)| {
            field_errors.iter().map(|error| FormFieldError {
                field: field.clone(),
                message: validation_error_message(error),
            })
        })
        .collect()
}

fn validation_error_message(error: &ValidationError) -> Cow<'static, str> {
    error
        .message
        .clone()
        .unwrap_or(Cow::Borrowed("Поле заполнено некорректно."))
}

fn validation_errors_display(errors: &ValidationErrors) -> String {
    let messages = collect_validation_errors(errors)
        .into_iter()
        .map(|error| error.message.into_owned())
        .collect::<Vec<_>>();

    if messages.is_empty() {
        "Ошибка валидации формы.".to_string()
    } else {
        format!("Ошибка валидации формы: {}", messages.join("; "))
    }
}

fn field_error(field: &'static str, message: impl Into<Cow<'static, str>>) -> FormFieldError {
    FormFieldError {
        field: Cow::Borrowed(field),
        message: message.into(),
    }
}

impl From<&FormError> for ApiMutationErrorDto {
    fn from(error: &FormError) -> Self {
        Self {
            message: "Ошибка валидации формы.".to_string(),
            field_errors: error
                .field_errors()
                .into_iter()
                .map(|error| ApiFieldErrorDto {
                    field: error.field.to_string(),
                    message: error.message.into_owned(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::auth::RegisterForm;
    use crate::forms::main::AddMenuForm;
    use validator::Validate;

    fn field_errors(error: &FormError) -> Vec<(String, String)> {
        let mut field_errors = error
            .field_errors()
            .into_iter()
            .map(|error| (error.field.to_string(), error.message.into_owned()))
            .collect::<Vec<_>>();
        field_errors.sort();
        field_errors
    }

    #[test]
    fn validation_errors_use_messages_declared_by_forms() {
        let form = RegisterForm {
            email: "invalid".to_string(),
            password: String::new(),
            hub_id: 0,
        };

        let error = FormError::from(form.validate().expect_err("form should be invalid"));

        assert_eq!(
            field_errors(&error),
            vec![
                (
                    "email".to_string(),
                    "Укажите корректный электронный адрес.".to_string(),
                ),
                ("hub_id".to_string(), "Выберите хаб.".to_string()),
                ("password".to_string(), "Введите пароль.".to_string()),
            ]
        );
    }

    #[test]
    fn name_and_url_validation_messages_come_from_forms() {
        let form = AddMenuForm {
            name: String::new(),
            url: "invalid-url".to_string(),
        };

        let error = FormError::from(form.validate().expect_err("form should be invalid"));

        assert_eq!(
            field_errors(&error),
            vec![
                ("name".to_string(), "Укажите имя.".to_string()),
                ("url".to_string(), "Укажите корректный URL.".to_string()),
            ]
        );
    }

    #[test]
    fn conversion_error_messages_stay_in_forms_layer() {
        assert_eq!(
            field_errors(&FormError::InvalidRoleId),
            vec![(
                "roles".to_string(),
                "Роль заполнена некорректно.".to_string(),
            )]
        );
    }

    #[test]
    fn form_error_display_is_localized() {
        let validation_error = FormError::from(
            RegisterForm {
                email: "invalid".to_string(),
                password: String::new(),
                hub_id: 0,
            }
            .validate()
            .expect_err("form should be invalid"),
        );

        let message = validation_error.to_string();
        assert!(message.contains("Ошибка валидации формы:"));
        assert!(message.contains("Укажите корректный электронный адрес."));
        assert!(message.contains("Введите пароль."));
        assert!(message.contains("Выберите хаб."));
        assert_eq!(
            FormError::InvalidEmail.to_string(),
            "Укажите корректный электронный адрес."
        );
    }

    #[test]
    fn mutation_error_from_form_error_preserves_field_errors() {
        let dto = ApiMutationErrorDto::from(&FormError::InvalidEmail);

        assert_eq!(dto.message, "Ошибка валидации формы.");
        assert_eq!(dto.field_errors.len(), 1);
        assert_eq!(dto.field_errors[0].field, "email");
        assert_eq!(
            dto.field_errors[0].message,
            "Укажите корректный электронный адрес."
        );
    }
}
