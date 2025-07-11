//! HTTP handlers and helpers.

use actix_web::HttpResponse;
use actix_web::http::header;
use actix_web_flash_messages::{FlashMessage, Level};
use lazy_static::lazy_static;
use log::error;
use tera::{Context, Tera};

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

pub fn alert_level_to_str(level: &Level) -> &'static str {
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

fn ensure_role(user: &AuthenticatedUser, role: &str) -> Result<(), HttpResponse> {
    if user.roles.iter().any(|r| r == role) {
        Ok(())
    } else {
        FlashMessage::error("Недостаточно прав.").send();
        Err(redirect("/"))
    }
}

fn render_template(template: &str, context: &Context) -> HttpResponse {
    HttpResponse::Ok().body(TEMPLATES.render(template, context).unwrap_or_else(|e| {
        error!("Failed to render template '{template}': {e}");
        String::new()
    }))
}
