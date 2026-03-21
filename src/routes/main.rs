//! General site routes and small API endpoints.

use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use log::error;
use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::routes::{alert_level_to_str, redirect};

use crate::dto::frontend::{
    AdminDashboardBootstrap, AdminHubDto, AdminMenuItemDto, AdminRoleDto, AdminUserListItemDto,
    BasicDashboardBootstrap, CurrentHubDto, CurrentUserDto, FlashAlertDto, MenuItemDto,
    SharedShellBootstrap,
};
use crate::forms::main::SaveUserForm;
use crate::frontend::open_frontend_html;
use crate::repository::DieselRepository;
use crate::services::main as main_service;

fn flash_alerts(flash_messages: IncomingFlashMessages) -> Vec<FlashAlertDto> {
    flash_messages
        .iter()
        .map(|f| FlashAlertDto::new(f.content(), alert_level_to_str(&f.level())))
        .collect()
}

fn is_admin(user: &AuthenticatedUser) -> bool {
    user.roles
        .iter()
        .any(|role| role == crate::SERVICE_ACCESS_ROLE)
}

/// Displays the main dashboard via `GET /` for the authenticated user.
#[get("/")]
pub async fn show_index(request: HttpRequest, user: AuthenticatedUser) -> impl Responder {
    let path = if is_admin(&user) {
        "assets/dist/app/index-admin.html"
    } else {
        "assets/dist/app/index-basic.html"
    };

    match open_frontend_html(path).await {
        Ok(file) => file.into_response(&request),
        Err(err) => {
            error!("Failed to open dashboard frontend document: {err}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Returns typed bootstrap data for the basic dashboard via `GET /bootstrap/basic`.
#[get("/bootstrap/basic")]
pub async fn basic_dashboard_bootstrap(
    user: AuthenticatedUser,
    flash_messages: IncomingFlashMessages,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    if is_admin(&user) {
        return HttpResponse::Forbidden().finish();
    }

    let data = match main_service::get_index_data(&user, repo.get_ref()) {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to build basic dashboard data: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let frontend_bootstrap = BasicDashboardBootstrap {
        shell: SharedShellBootstrap {
            alerts: flash_alerts(flash_messages),
        },
        current_user: CurrentUserDto::from(&user),
        current_hub: CurrentHubDto::from(data.hub),
        current_page: "index".to_string(),
        menu: data.menu.into_iter().map(MenuItemDto::from).collect(),
        user_name: data.user_name,
    };

    HttpResponse::Ok().json(frontend_bootstrap)
}

/// Returns typed bootstrap data for the admin dashboard via `GET /bootstrap/admin`.
#[get("/bootstrap/admin")]
pub async fn admin_dashboard_bootstrap(
    user: AuthenticatedUser,
    flash_messages: IncomingFlashMessages,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    if !is_admin(&user) {
        return HttpResponse::Forbidden().finish();
    }

    let data = match main_service::get_index_data(&user, repo.get_ref()) {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to build admin dashboard data: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let frontend_bootstrap = AdminDashboardBootstrap {
        shell: SharedShellBootstrap {
            alerts: flash_alerts(flash_messages),
        },
        current_user: CurrentUserDto::from(&user),
        current_hub: CurrentHubDto::from(data.hub),
        current_page: "index".to_string(),
        menu: data
            .menu
            .clone()
            .into_iter()
            .map(MenuItemDto::from)
            .collect(),
        roles: data.roles.into_iter().map(AdminRoleDto::from).collect(),
        hubs: data.hubs.into_iter().map(AdminHubDto::from).collect(),
        admin_menu: data.menu.into_iter().map(AdminMenuItemDto::from).collect(),
        users: data
            .users
            .into_iter()
            .map(AdminUserListItemDto::from)
            .collect(),
    };

    HttpResponse::Ok().json(frontend_bootstrap)
}

/// Saves profile updates for the current user via `POST /user/save`.
#[post("/user/save")]
pub async fn save_user(
    web::Form(form): web::Form<SaveUserForm>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    match main_service::update_current_user(form, &current_user, repo.get_ref()) {
        Ok(_) => {
            FlashMessage::success("Параметры изменены.".to_string()).send();
        }
        Err(pushkind_common::services::errors::ServiceError::Form(e)) => {
            log::error!("Failed to validate settings: {e}");
            FlashMessage::error("Ошибка валидации формы").send();
        }
        Err(err) => {
            log::error!("Failed to update settings: {err}");
            FlashMessage::error("Ошибка при изменении параметров").send();
        }
    }
    redirect("/")
}
