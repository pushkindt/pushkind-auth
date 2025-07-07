use pushkind_auth::routes::auth::get_success_and_failure_redirects;
use pushkind_auth::routes::alert_level_to_str;
use actix_web_flash_messages::Level;

#[test]
fn test_get_success_and_failure_redirects_with_next() {
    let (success, failure) = get_success_and_failure_redirects("/auth/signin", Some("/dashboard"));
    assert_eq!(success, "/dashboard");
    assert_eq!(failure, "/auth/signin?next=/dashboard");
}

#[test]
fn test_get_success_and_failure_redirects_without_next() {
    let (success, failure) = get_success_and_failure_redirects("/auth/signup", None);
    assert_eq!(success, "/");
    assert_eq!(failure, "/auth/signup");
}

#[test]
fn test_get_success_and_failure_redirects_with_empty_next() {
    let (success, failure) = get_success_and_failure_redirects("/auth/signin", Some(""));
    assert_eq!(success, "/");
    assert_eq!(failure, "/auth/signin");
}

#[test]
fn test_alert_level_to_str_mappings() {
    assert_eq!(alert_level_to_str(&Level::Error), "danger");
    assert_eq!(alert_level_to_str(&Level::Warning), "warning");
    assert_eq!(alert_level_to_str(&Level::Success), "success");
    assert_eq!(alert_level_to_str(&Level::Info), "info");
    assert_eq!(alert_level_to_str(&Level::Debug), "info");
}

