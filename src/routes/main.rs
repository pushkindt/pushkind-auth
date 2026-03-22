//! General site routes and small API endpoints.

use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use log::error;
use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::routes::redirect;

use crate::dto::api::{ApiMutationErrorDto, ApiMutationSuccessDto};
use crate::forms::main::{SaveUserForm, SaveUserPayload};
use crate::frontend::open_frontend_html;
use crate::repository::DieselRepository;
use crate::routes::{form_error_response, wants_json};
use crate::services::main as main_service;

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

/// Saves profile updates for the current user via `POST /user/save`.
#[post("/user/save")]
pub async fn save_user(
    request: HttpRequest,
    web::Form(form): web::Form<SaveUserForm>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let wants_json = wants_json(&request);
    let payload = match SaveUserPayload::try_from(form) {
        Ok(payload) => payload,
        Err(error) => {
            if wants_json {
                return HttpResponse::BadRequest().json(form_error_response(&error));
            }

            log::error!("Failed to validate settings: {error}");
            return redirect("/");
        }
    };

    match main_service::update_current_user(payload, &current_user, repo.get_ref()) {
        Ok(_) => {
            if wants_json {
                return HttpResponse::Ok().json(ApiMutationSuccessDto {
                    message: "Параметры изменены.".to_string(),
                    redirect_to: None,
                });
            }
        }
        Err(pushkind_common::services::errors::ServiceError::Form(e)) => {
            log::error!("Failed to validate settings: {e}");
            if wants_json {
                return HttpResponse::BadRequest().json(ApiMutationErrorDto {
                    message: "Ошибка валидации формы.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(err) => {
            log::error!("Failed to update settings: {err}");
            if wants_json {
                return HttpResponse::InternalServerError().json(ApiMutationErrorDto {
                    message: "Ошибка при изменении параметров.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
    }
    redirect("/")
}
