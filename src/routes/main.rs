//! General site routes and small API endpoints.

use actix_web::{HttpResponse, Responder, get, post, web};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use log::error;
use tera::Context;

use crate::db::DbPool;
use crate::forms::main::SaveUserForm;
use crate::models::auth::AuthenticatedUser;
use crate::repository::hub::DieselHubRepository;
use crate::repository::menu::DieselMenuRepository;
use crate::repository::role::DieselRoleRepository;
use crate::repository::user::DieselUserRepository;
use crate::repository::{HubRepository, MenuRepository, RoleRepository, UserRepository};
use crate::routes::{alert_level_to_str, redirect, render_template};

#[get("/")]
pub async fn index(
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
    flash_messages: IncomingFlashMessages,
) -> impl Responder {
    let user_id: i32 = match user.sub.parse() {
        Ok(user_id) => user_id,
        Err(e) => {
            error!("Failed to parse user_id: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let repo = DieselUserRepository::new(&pool);

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

    let repo = DieselRoleRepository::new(&pool);

    let roles = match repo.list() {
        Ok(roles) => roles,
        Err(e) => {
            error!("Failed to list roles: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let repo = DieselHubRepository::new(&pool);

    let hubs = match repo.list() {
        Ok(hubs) => hubs,
        Err(e) => {
            error!("Failed to list hubs: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let repo = DieselMenuRepository::new(&pool);

    let menu = match repo.list(user.hub_id) {
        Ok(menu) => menu,
        Err(e) => {
            error!("Failed to list menu: {}", e);
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
    context.insert("menu", &menu);

    render_template("main/index.html", &context)
}

#[post("/user/save")]
pub async fn save_user(
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
    web::Form(form): web::Form<SaveUserForm>,
) -> impl Responder {
    let repo = DieselUserRepository::new(&pool);

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
            FlashMessage::error(format!("Ошибка при изменении параметров: {}", err)).send();
        }
    }
    redirect("/")
}

#[get("/api/v1/id")]
pub async fn api_v1_id(user: AuthenticatedUser) -> impl Responder {
    HttpResponse::Ok().json(user)
}

#[get("/api/v1/users")]
pub async fn api_v1_users(user: AuthenticatedUser, pool: web::Data<DbPool>) -> impl Responder {
    let repo = DieselUserRepository::new(&pool);

    match repo.list(user.hub_id) {
        Ok(users) => {
            let users: Vec<AuthenticatedUser> = users
                .iter()
                .map(|(user, roles)| AuthenticatedUser::from_user(user, roles))
                .collect();

            HttpResponse::Ok().json(users)
        }
        Err(e) => {
            error!("Failed to list users: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
