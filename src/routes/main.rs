//! General site routes and small API endpoints.

use actix_web::{HttpResponse, Responder, get, post, web};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use log::error;
use pushkind_common::db::DbPool;
use pushkind_common::models::auth::AuthenticatedUser;
use pushkind_common::routes::{alert_level_to_str, redirect};
use tera::Context;

use crate::forms::main::SaveUserForm;
use crate::repository::UserListQuery;
use crate::repository::hub::DieselHubRepository;
use crate::repository::menu::DieselMenuRepository;
use crate::repository::role::DieselRoleRepository;
use crate::repository::user::DieselUserRepository;
use crate::repository::{HubReader, MenuReader, RoleReader, UserReader, UserWriter};
use crate::routes::render_template;

#[get("/")]
pub async fn index(
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
    flash_messages: IncomingFlashMessages,
) -> impl Responder {

    let repo = DieselUserRepository::new(&pool);

    let users = match repo.list(UserListQuery::new(user.hub_id)) {
        Ok((_total, users)) => users,
        Err(e) => {
            error!("Failed to list users: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let repo = DieselRoleRepository::new(&pool);

    let roles = match repo.list() {
        Ok(roles) => roles,
        Err(e) => {
            error!("Failed to list roles: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let repo = DieselHubRepository::new(&pool);

    let hubs = match repo.list() {
        Ok(hubs) => hubs,
        Err(e) => {
            error!("Failed to list hubs: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let repo = DieselMenuRepository::new(&pool);

    let menu = match repo.list(user.hub_id) {
        Ok(menu) => menu,
        Err(e) => {
            error!("Failed to list menu: {e}");
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
            error!("Failed to parse user_id: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let update_user = (&form).into();
    match repo.update(user_id, &update_user) {
        Ok(_) => {
            FlashMessage::success("Параметры изменены.".to_string()).send();
        }
        Err(err) => {
            log::error!("Failed to update settings: {err}");
            FlashMessage::error("Ошибка при изменении параметров").send();
        }
    }
    redirect("/")
}
