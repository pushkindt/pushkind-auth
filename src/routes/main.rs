use actix_web::{HttpResponse, Responder, get, web};
use log::error;
use tera::Context;

use crate::TEMPLATES;
use crate::db::DbPool;
use crate::models::auth::AuthenticatedUser;
use crate::repository::role::DieselRoleRepository;
use crate::repository::user::DieselUserRepository;
use crate::repository::{RoleRepository, UserRepository};

#[get("/")]
pub async fn index(user: AuthenticatedUser, pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    let mut repo = DieselUserRepository::new(&mut conn);

    let users = match repo.list(user.hub_id) {
        Ok(users) => users,
        Err(e) => {
            error!("Failed to list users: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut repo = DieselRoleRepository::new(&mut conn);

    let roles = match repo.list() {
        Ok(roles) => roles,
        Err(e) => {
            error!("Failed to list roles: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut context = Context::new();
    context.insert("current_user", &user);
    context.insert("current_page", "index");
    context.insert("users", &users);
    context.insert("roles", &roles);

    HttpResponse::Ok().body(
        TEMPLATES
            .render("main/index.html", &context)
            .unwrap_or_else(|e| {
                error!("Failed to render template 'main/index.html': {}", e);
                String::new()
            }),
    )
}
