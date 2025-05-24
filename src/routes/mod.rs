use actix_web::HttpResponse;
use actix_web::http::header;
use actix_web_flash_messages::Level;

pub mod admin;
pub mod auth;
pub mod main;

pub fn alert_level_to_str(level: &Level) -> &'static str {
    match level {
        Level::Error => "danger",
        Level::Warning => "warning",
        Level::Success => "success",
        _ => "info",
    }
}

pub fn redirect(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, location))
        .finish()
}
