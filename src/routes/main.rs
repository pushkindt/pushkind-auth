use actix_web::{HttpResponse, Responder, get};
use log::error;
use tera::Context;

use crate::TEMPLATES;
use crate::models::auth::AuthenticatedUser;

#[get("/")]
pub async fn index(user: AuthenticatedUser) -> impl Responder {
    let mut context = Context::new();

    context.insert("current_user", &user);
    context.insert("current_page", "index");

    HttpResponse::Ok().body(
        TEMPLATES
            .render("main/index.html", &context)
            .unwrap_or_else(|e| {
                error!("Failed to render template 'main/index.html': {}", e);
                String::new()
            }),
    )
}
