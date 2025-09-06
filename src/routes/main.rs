//! General site routes and small API endpoints.

use actix_web::{HttpResponse, Responder, get, post, web};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use log::error;
use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::routes::render_template;
use pushkind_common::routes::{alert_level_to_str, redirect};
use tera::{Context, Tera};

use crate::forms::main::SaveUserForm;
use crate::repository::UserListQuery;
use crate::repository::{
    DieselRepository, HubReader, MenuReader, RoleReader, UserReader, UserWriter,
};

#[get("/")]
pub async fn show_index(
    user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
    flash_messages: IncomingFlashMessages,
    tera: web::Data<Tera>,
) -> impl Responder {
    let hub = match repo.get_hub_by_id(user.hub_id) {
        Ok(Some(hub)) => hub,
        Ok(None) => {
            error!("Hub not found");
            return HttpResponse::InternalServerError().finish();
        }
        Err(e) => {
            error!("Failed to get hub: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let users = match repo.list_users(UserListQuery::new(user.hub_id)) {
        Ok((_total, users)) => users,
        Err(e) => {
            error!("Failed to list users: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let roles = match repo.list_roles() {
        Ok(roles) => roles,
        Err(e) => {
            error!("Failed to list roles: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let hubs = match repo.list_hubs() {
        Ok(hubs) => hubs,
        Err(e) => {
            error!("Failed to list hubs: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let menu = match repo.list_menu(user.hub_id) {
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

    let user_name = match repo.get_user_by_email(&user.email, user.hub_id) {
        Ok(Some(user)) => user.user.name,
        Ok(None) => {
            error!("User not found");
            return HttpResponse::InternalServerError().finish();
        }
        Err(e) => {
            error!("Failed to get user: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut context = Context::new();
    context.insert("alerts", &alerts);
    context.insert("current_user", &user);
    context.insert("user_name", &user_name);
    context.insert("current_page", "index");
    context.insert("current_hub", &hub);
    context.insert("users", &users);
    context.insert("roles", &roles);
    context.insert("hubs", &hubs);
    context.insert("menu", &menu);

    render_template(&tera, "main/index.html", &context)
}

#[post("/user/save")]
pub async fn save_user(
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
    web::Form(form): web::Form<SaveUserForm>,
) -> impl Responder {
    let user_id = match current_user.sub.parse() {
        Ok(user_id) => user_id,
        Err(e) => {
            error!("Failed to parse user_id: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let update_user = form.into();
    match repo.update_user(user_id, current_user.hub_id, &update_user) {
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
