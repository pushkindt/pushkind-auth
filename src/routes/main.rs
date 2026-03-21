//! General site routes and small API endpoints.

use actix_web::{HttpResponse, Responder, get, post, web};
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
use crate::frontend::{FrontendAssetManifest, FrontendMountLayout, render_frontend_page};
use crate::repository::DieselRepository;
use crate::services::main as main_service;

/// Displays the main dashboard via `GET /` for the authenticated user.
#[get("/")]
pub async fn show_index(
    user: AuthenticatedUser,
    flash_messages: IncomingFlashMessages,
    repo: web::Data<DieselRepository>,
    frontend_assets: web::Data<FrontendAssetManifest>,
) -> impl Responder {
    let data = match main_service::get_index_data(&user, repo.get_ref()) {
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

    if user
        .roles
        .iter()
        .any(|role| role == crate::SERVICE_ACCESS_ROLE)
    {
        let frontend_assets = match frontend_assets.assets_for("src/entries/main-admin.tsx") {
            Ok(assets) => assets,
            Err(err) => {
                error!("Failed to resolve admin dashboard frontend assets: {err}");
                return HttpResponse::InternalServerError().finish();
            }
        };
        let frontend_bootstrap = AdminDashboardBootstrap {
            shell: SharedShellBootstrap {
                alerts: alerts
                    .iter()
                    .map(|(message, level)| FlashAlertDto::new(*message, *level))
                    .collect(),
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
        match render_frontend_page(
            &frontend_assets,
            &frontend_bootstrap,
            FrontendMountLayout::Container,
        ) {
            Ok(response) => response,
            Err(err) => {
                error!("Failed to render admin frontend shell: {err}");
                HttpResponse::InternalServerError().finish()
            }
        }
    } else {
        let frontend_assets = match frontend_assets.assets_for("src/entries/main-basic.tsx") {
            Ok(assets) => assets,
            Err(err) => {
                error!("Failed to resolve basic dashboard frontend assets: {err}");
                return HttpResponse::InternalServerError().finish();
            }
        };
        let frontend_bootstrap = BasicDashboardBootstrap {
            shell: SharedShellBootstrap {
                alerts: alerts
                    .iter()
                    .map(|(message, level)| FlashAlertDto::new(*message, *level))
                    .collect(),
            },
            current_user: CurrentUserDto::from(&user),
            current_hub: CurrentHubDto::from(data.hub),
            current_page: "index".to_string(),
            menu: data.menu.into_iter().map(MenuItemDto::from).collect(),
            user_name,
        };
        match render_frontend_page(
            &frontend_assets,
            &frontend_bootstrap,
            FrontendMountLayout::Container,
        ) {
            Ok(response) => response,
            Err(err) => {
                error!("Failed to render basic frontend shell: {err}");
                HttpResponse::InternalServerError().finish()
            }
        }
    }
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
