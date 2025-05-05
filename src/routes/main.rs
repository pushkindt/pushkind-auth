use actix_web::{HttpResponse, Responder, get, web};
use tera::Context;

use crate::TEMPLATES;
use crate::db::DbPool;
use crate::models::auth::AuthenticatedUser;

#[get("/")]
pub async fn index(user: AuthenticatedUser, pool: web::Data<DbPool>) -> impl Responder {
    let mut context = Context::new();

    context.insert("current_user", &user);
    context.insert("current_page", "index");

    HttpResponse::Ok().body(
        TEMPLATES
            .render("main/index.html", &context)
            .unwrap_or_default(),
    )
}
