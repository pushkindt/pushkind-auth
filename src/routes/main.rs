use actix_web::{HttpResponse, Responder, get, post, web};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use log::error;
use tera::Context;

use crate::TEMPLATES;
use crate::db::DbPool;
use crate::forms::main::SaveUserForm;
use crate::models::auth::AuthenticatedUser;
use crate::repository::hub::DieselHubRepository;
use crate::repository::role::DieselRoleRepository;
use crate::repository::user::DieselUserRepository;
use crate::repository::{HubRepository, RoleRepository, UserRepository};
use crate::routes::{alert_level_to_str, redirect};

#[get("/")]
pub async fn index(
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
    flash_messages: IncomingFlashMessages,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let user_id: i32 = match user.sub.parse() {
        Ok(user_id) => user_id,
        Err(e) => {
            error!("Failed to parse user_id: {}", e);
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

    let user = match repo.get_by_id(user_id) {
        Ok(Some(user)) => user,
        Ok(None) => {
            error!("User not found");
            return HttpResponse::InternalServerError().finish();
        }
        Err(e) => {
            error!("Failed to get user: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let user_roles = match repo.get_roles(user_id) {
        Ok(user_roles) => user_roles
            .into_iter()
            .map(|r| r.name)
            .collect::<Vec<String>>(),
        Err(e) => {
            error!("Failed to get user roles: {}", e);
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

    let mut repo = DieselHubRepository::new(&mut conn);

    let hubs = match repo.list() {
        Ok(hubs) => hubs,
        Err(e) => {
            error!("Failed to list hubs: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let alerts = flash_messages
        .iter()
        .map(|f| (f.content(), alert_level_to_str(&f.level())))
        .collect::<Vec<_>>();

    let mut context = Context::new();
    context.insert("alerts", &alerts);
    context.insert("current_user", &user);
    context.insert("current_user_roles", &user_roles);
    context.insert("current_page", "index");
    context.insert("users", &users);
    context.insert("roles", &roles);
    context.insert("hubs", &hubs);

    HttpResponse::Ok().body(
        TEMPLATES
            .render("main/index.html", &context)
            .unwrap_or_else(|e| {
                error!("Failed to render template 'main/index.html': {}", e);
                String::new()
            }),
    )
}

#[post("/user/save")]
pub async fn save_user(
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
    web::Form(form): web::Form<SaveUserForm>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    let mut repo = DieselUserRepository::new(&mut conn);

    let user_id = match user.sub.parse() {
        Ok(user_id) => user_id,
        Err(e) => {
            error!("Failed to parse user_id: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match repo.update(user_id, &form.into()) {
        Ok(_) => {
            FlashMessage::success("Параметры изменены.".to_string()).send();
        }
        Err(err) => {
            FlashMessage::error(format!("Ошибка при изменений параметров: {}", err)).send();
        }
    }
    redirect("/")
}
