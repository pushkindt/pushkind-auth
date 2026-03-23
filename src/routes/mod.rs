//! HTTP handlers and helpers.
use crate::dto::api::ApiMutationErrorDto;
use actix_web::{HttpResponse, http::StatusCode};
use pushkind_common::services::errors::ServiceError;
use url::Url;

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

pub(crate) fn mutation_error_status(err: &ServiceError) -> StatusCode {
    match err {
        ServiceError::Form(_) | ServiceError::TypeConstraint(_) => StatusCode::BAD_REQUEST,
        ServiceError::Unauthorized => StatusCode::FORBIDDEN,
        ServiceError::NotFound => StatusCode::NOT_FOUND,
        ServiceError::Conflict => StatusCode::CONFLICT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) fn mutation_error_response(err: &ServiceError) -> HttpResponse {
    HttpResponse::build(mutation_error_status(err)).json(ApiMutationErrorDto::from(err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pushkind_common::services::errors::ServiceError;

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

    #[test]
    fn mutation_error_status_uses_bad_request_for_form_errors() {
        assert_eq!(
            mutation_error_status(&ServiceError::Form("invalid".to_string())),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn mutation_error_status_uses_bad_request_for_type_constraints() {
        assert_eq!(
            mutation_error_status(&ServiceError::TypeConstraint("invalid".to_string())),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn mutation_error_status_uses_forbidden_for_unauthorized_errors() {
        assert_eq!(
            mutation_error_status(&ServiceError::Unauthorized),
            StatusCode::FORBIDDEN
        );
    }

    #[test]
    fn mutation_error_status_uses_not_found_for_missing_resources() {
        assert_eq!(
            mutation_error_status(&ServiceError::NotFound),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn mutation_error_status_uses_conflict_for_conflicts() {
        assert_eq!(
            mutation_error_status(&ServiceError::Conflict),
            StatusCode::CONFLICT
        );
    }

    #[test]
    fn mutation_error_status_uses_internal_server_error_for_other_errors() {
        assert_eq!(
            mutation_error_status(&ServiceError::Internal),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
