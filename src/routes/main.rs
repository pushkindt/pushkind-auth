//! General site routes and small API endpoints.

use actix_web::{HttpResponse, Responder, get, post, web};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use log::error;
use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::routes::render_template;
use pushkind_common::routes::{alert_level_to_str, redirect};
use tera::{Context, Tera};

use crate::forms::main::SaveUserForm;
use crate::repository::DieselRepository;
use crate::services::main as main_service;

#[get("/")]
pub async fn show_index(
    user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
    flash_messages: IncomingFlashMessages,
    tera: web::Data<Tera>,
) -> impl Responder {
    let data = match main_service::get_index_data(user.hub_id, &user.email, repo.get_ref()) {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to build index data: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let alerts = flash_messages
        .iter()
        .map(|f| (f.content(), alert_level_to_str(&f.level())))
        .collect::<Vec<_>>();

    let user_name = data.user_name;

    let mut context = Context::new();
    context.insert("alerts", &alerts);
    context.insert("current_user", &user);
    context.insert("user_name", &user_name);
    context.insert("current_page", "index");
    context.insert("current_hub", &data.hub);
    context.insert("users", &data.users);
    context.insert("roles", &data.roles);
    context.insert("hubs", &data.hubs);
    context.insert("menu", &data.menu);

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
    match main_service::update_current_user(user_id, current_user.hub_id, &update_user, repo.get_ref()) {
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
