//! HTTP handlers and helpers.

use actix_web::HttpResponse;
use actix_web::http::header;
use actix_web_flash_messages::{FlashMessage, Level};
use lazy_static::lazy_static;
use log::error;
use tera::{Context, Tera};
use url::Url;

use crate::models::auth::AuthenticatedUser;

pub mod admin;
pub mod api;
pub mod auth;
pub mod main;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {e}");
                ::std::process::exit(1);
            }
        }
    };
}

fn alert_level_to_str(level: &Level) -> &'static str {
    match level {
        Level::Error => "danger",
        Level::Warning => "warning",
        Level::Success => "success",
        _ => "info",
    }
}

fn redirect(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, location))
        .finish()
}

fn check_role<I, S>(role: &str, roles: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    roles.into_iter().any(|r| r.as_ref() == role)
}

fn ensure_role(
    user: &AuthenticatedUser,
    role: &str,
    redirect_url: Option<&str>,
) -> Result<(), HttpResponse> {
    if check_role(role, &user.roles) {
        Ok(())
    } else {
        FlashMessage::error("Недостаточно прав.").send();
        Err(redirect(redirect_url.unwrap_or("/")))
    }
}

fn render_template(template: &str, context: &Context) -> HttpResponse {
    HttpResponse::Ok().body(TEMPLATES.render(template, context).unwrap_or_else(|e| {
        error!("Failed to render template '{template}': {e}");
        String::new()
    }))
}

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

fn get_success_and_failure_redirects(
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web_flash_messages::Level;

    #[test]
    fn check_role_detects_role() {
        assert!(check_role("admin", &["user", "admin"]));
        assert!(!check_role("admin", &["user", "manager"]));
    }

    #[test]
    fn alert_level_mappings() {
        assert_eq!(alert_level_to_str(&Level::Error), "danger");
        assert_eq!(alert_level_to_str(&Level::Warning), "warning");
        assert_eq!(alert_level_to_str(&Level::Success), "success");
        assert_eq!(alert_level_to_str(&Level::Info), "info");
        assert_eq!(alert_level_to_str(&Level::Debug), "info");
    }

    #[test]
    fn redirect_sets_location_header() {
        let resp = redirect("/target");
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        assert_eq!(resp.headers().get(header::LOCATION).unwrap(), "/target");
    }

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
