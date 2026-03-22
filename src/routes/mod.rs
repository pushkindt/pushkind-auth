//! HTTP handlers and helpers.
use crate::dto::api::{ApiFieldErrorDto, ApiMutationErrorDto};
use crate::forms::FormError;
use actix_web::HttpRequest;
use url::Url;
use validator::ValidationErrors;

pub mod admin;
pub mod api;
pub mod auth;
pub mod main;

fn is_valid_next(next: &str, domain: &str) -> bool {
    if next.starts_with("//") {
        return false;
    }
    if let Ok(url) = Url::parse(next) {
        match url.host_str() {
            Some(host) => host == domain || host.ends_with(&format!(".{domain}")),
            None => false,
        }
    } else {
        true
    }
}

pub(crate) fn get_success_and_failure_redirects(
    base_url: &str,
    next: Option<&str>,
    domain: &str,
) -> (String, String) {
    let next_valid = next.and_then(|n| {
        if !n.is_empty() && is_valid_next(n, domain) {
            Some(n)
        } else {
            None
        }
    });

    let success_redirect_url = next_valid
        .map(|s| s.to_string())
        .unwrap_or_else(|| "/".to_string());

    let failure_redirect_url = next_valid
        .map(|s| format!("{base_url}?next={s}"))
        .unwrap_or_else(|| base_url.to_string());

    (success_redirect_url, failure_redirect_url)
}

pub(crate) fn wants_json(request: &HttpRequest) -> bool {
    request
        .headers()
        .get(actix_web::http::header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("application/json"))
}

fn validation_message(field: &str, code: &str) -> String {
    match (field, code) {
        ("email", _) => "Укажите корректный электронный адрес.".to_string(),
        ("hub_id", _) => "Выберите хаб.".to_string(),
        ("name", _) => "Укажите имя.".to_string(),
        ("password", "length") => "Введите пароль.".to_string(),
        ("password", _) => "Пароль заполнен некорректно.".to_string(),
        ("url", _) => "Укажите корректный URL.".to_string(),
        ("roles", _) => "Роль заполнена некорректно.".to_string(),
        _ => "Поле заполнено некорректно.".to_string(),
    }
}

fn map_validation_errors(errors: &ValidationErrors) -> Vec<ApiFieldErrorDto> {
    errors
        .field_errors()
        .iter()
        .flat_map(|(field, field_errors)| {
            field_errors.iter().map(|error| ApiFieldErrorDto {
                field: (*field).to_string(),
                message: validation_message(field, error.code.as_ref()),
            })
        })
        .collect()
}

pub(crate) fn form_error_response(error: &FormError) -> ApiMutationErrorDto {
    match error {
        FormError::Validation(errors) => ApiMutationErrorDto {
            message: "Ошибка валидации формы.".to_string(),
            field_errors: map_validation_errors(errors),
        },
        FormError::InvalidEmail => ApiMutationErrorDto {
            message: "Ошибка валидации формы.".to_string(),
            field_errors: vec![ApiFieldErrorDto {
                field: "email".to_string(),
                message: "Укажите корректный электронный адрес.".to_string(),
            }],
        },
        FormError::InvalidPassword => ApiMutationErrorDto {
            message: "Ошибка валидации формы.".to_string(),
            field_errors: vec![ApiFieldErrorDto {
                field: "password".to_string(),
                message: "Пароль заполнен некорректно.".to_string(),
            }],
        },
        FormError::InvalidHubId => ApiMutationErrorDto {
            message: "Ошибка валидации формы.".to_string(),
            field_errors: vec![ApiFieldErrorDto {
                field: "hub_id".to_string(),
                message: "Выберите хаб.".to_string(),
            }],
        },
        FormError::InvalidName => ApiMutationErrorDto {
            message: "Ошибка валидации формы.".to_string(),
            field_errors: vec![ApiFieldErrorDto {
                field: "name".to_string(),
                message: "Укажите имя.".to_string(),
            }],
        },
        FormError::InvalidUrl => ApiMutationErrorDto {
            message: "Ошибка валидации формы.".to_string(),
            field_errors: vec![ApiFieldErrorDto {
                field: "url".to_string(),
                message: "Укажите корректный URL.".to_string(),
            }],
        },
        FormError::InvalidRoleId => ApiMutationErrorDto {
            message: "Ошибка валидации формы.".to_string(),
            field_errors: vec![ApiFieldErrorDto {
                field: "roles".to_string(),
                message: "Роль заполнена некорректно.".to_string(),
            }],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirects_with_next_param() {
        let (success, failure) =
            get_success_and_failure_redirects("/auth/signin", Some("/dashboard"), "example.com");
        assert_eq!(success, "/dashboard");
        assert_eq!(failure, "/auth/signin?next=/dashboard");
    }

    #[test]
    fn redirects_without_next_param() {
        let (success, failure) =
            get_success_and_failure_redirects("/auth/signup", None, "example.com");
        assert_eq!(success, "/");
        assert_eq!(failure, "/auth/signup");
    }

    #[test]
    fn redirects_with_empty_next() {
        let (success, failure) =
            get_success_and_failure_redirects("/auth/signin", Some(""), "example.com");
        assert_eq!(success, "/");
        assert_eq!(failure, "/auth/signin");
    }

    #[test]
    fn invalid_domain_next_defaults_to_base() {
        let (success, failure) = get_success_and_failure_redirects(
            "/auth/signin",
            Some("http://evil.com"),
            "example.com",
        );
        assert_eq!(success, "/");
        assert_eq!(failure, "/auth/signin");
    }
}
