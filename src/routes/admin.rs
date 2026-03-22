//! Administrative endpoints used to manage users, roles and hubs.

use actix_web::{HttpRequest, HttpResponse, Responder, post, web};
use log::error;
use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::routes::redirect;
use pushkind_common::services::errors::ServiceError;

use crate::dto::admin::UserModalData;
use crate::dto::api::{ApiMutationErrorDto, ApiMutationSuccessDto};
use crate::dto::frontend::{AdminEditableUserDto, AdminUserModalBootstrap, RoleOptionDto};
use crate::forms::main::{
    AddHubForm, AddHubPayload, AddMenuForm, AddMenuPayload, AddRoleForm, AddRolePayload,
    UpdateUserForm, UpdateUserPayload,
};
use crate::repository::DieselRepository;
use crate::routes::{form_error_response, wants_json};
use crate::services::admin as admin_service;

/// Handles `POST /role/add` to create a new role and flash the outcome.
#[post("/role/add")]
pub async fn add_role(
    request: HttpRequest,
    web::Form(form): web::Form<AddRoleForm>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let wants_json = wants_json(&request);
    let payload = match AddRolePayload::try_from(form) {
        Ok(payload) => payload,
        Err(error) => {
            if wants_json {
                return HttpResponse::BadRequest().json(form_error_response(&error));
            }
            log::error!("Invalid role data: {error}");
            return redirect("/");
        }
    };

    match admin_service::create_role(payload, &current_user, repo.get_ref()) {
        Ok(_) => {
            if wants_json {
                return HttpResponse::Created().json(ApiMutationSuccessDto {
                    message: "Роль добавлена.".to_string(),
                    redirect_to: None,
                });
            }
        }
        Err(ServiceError::Form(e)) => {
            log::error!("Invalid role data: {e}");
            if wants_json {
                return HttpResponse::BadRequest().json(ApiMutationErrorDto {
                    message: "Ошибка валидации формы.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(ServiceError::Conflict) => {
            if wants_json {
                return HttpResponse::Conflict().json(ApiMutationErrorDto {
                    message: "Роль уже существует.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(ServiceError::Unauthorized) => {
            if wants_json {
                return HttpResponse::Forbidden().json(ApiMutationErrorDto {
                    message: "Недостаточно прав.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(err) => {
            log::error!("Failed to add role: {err}");
            if wants_json {
                return HttpResponse::InternalServerError().json(ApiMutationErrorDto {
                    message: "Ошибка при добавлении роли.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
    }
    redirect("/")
}

/// Builds modal data for a user via `POST /user/modal/{user_id}`.
#[post("/user/modal/{user_id}")]
pub async fn user_modal(
    user_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let user_id = user_id.into_inner();

    match admin_service::user_modal_data(user_id, &current_user, repo.get_ref()) {
        Ok(data) => {
            let UserModalData { user, roles } = data;
            HttpResponse::Ok().json(AdminUserModalBootstrap {
                user: user.map(AdminEditableUserDto::from),
                roles: roles.into_iter().map(RoleOptionDto::from).collect(),
            })
        }
        Err(ServiceError::Unauthorized) => HttpResponse::Unauthorized().finish(),
        Err(e) => {
            error!("Failed to build user modal data: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Deletes a user by id for `POST /user/delete/{user_id}`.
#[post("/user/delete/{user_id}")]
pub async fn delete_user(
    request: HttpRequest,
    user_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let wants_json = wants_json(&request);
    let target_id = user_id.into_inner();

    match admin_service::delete_user_by_id(target_id, &current_user, repo.get_ref()) {
        Ok(_) => {
            if wants_json {
                return HttpResponse::Ok().json(ApiMutationSuccessDto {
                    message: "Пользователь удалён.".to_string(),
                    redirect_to: None,
                });
            }
        }
        Err(ServiceError::NotFound) => {
            if wants_json {
                return HttpResponse::NotFound().json(ApiMutationErrorDto {
                    message: "Пользователь не найден.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(ServiceError::Unauthorized) => {
            if wants_json {
                return HttpResponse::Forbidden().json(ApiMutationErrorDto {
                    message: "Недостаточно прав.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(err) => {
            log::error!("Failed to delete user: {err}");
            if wants_json {
                return HttpResponse::InternalServerError().json(ApiMutationErrorDto {
                    message: "Ошибка при удалении пользователя.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
    }
    redirect("/")
}

/// Updates user data and role assignments from the admin form.
#[post("/user/update/{user_id}")]
pub async fn update_user(
    request: HttpRequest,
    user_id: web::Path<i32>,
    form: web::Bytes,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let wants_json = wants_json(&request);
    let form: UpdateUserForm = match serde_html_form::from_bytes(&form) {
        Ok(form) => form,
        Err(err) => {
            log::error!("Failed to process form: {err}");
            if wants_json {
                return HttpResponse::BadRequest().json(ApiMutationErrorDto {
                    message: "Ошибка при обработке формы.".to_string(),
                    field_errors: Vec::new(),
                });
            }
            return redirect("/");
        }
    };
    let target_id = user_id.into_inner();
    let payload = match UpdateUserPayload::try_from(form) {
        Ok(payload) => payload,
        Err(error) => {
            if wants_json {
                return HttpResponse::BadRequest().json(form_error_response(&error));
            }
            log::error!("Invalid user data: {error}");
            return redirect("/");
        }
    };

    match admin_service::assign_roles_and_update_user(
        target_id,
        payload,
        &current_user,
        repo.get_ref(),
    ) {
        Ok(_) => {
            if wants_json {
                return HttpResponse::Ok().json(ApiMutationSuccessDto {
                    message: "Пользователь изменён.".to_string(),
                    redirect_to: None,
                });
            }
        }
        Err(ServiceError::Form(e)) => {
            log::error!("Invalid user data: {e}");
            if wants_json {
                return HttpResponse::BadRequest().json(ApiMutationErrorDto {
                    message: "Ошибка валидации формы.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(ServiceError::NotFound) => {
            if wants_json {
                return HttpResponse::NotFound().json(ApiMutationErrorDto {
                    message: "Пользователь не найден.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(ServiceError::Unauthorized) => {
            if wants_json {
                return HttpResponse::Forbidden().json(ApiMutationErrorDto {
                    message: "Недостаточно прав.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(err) => {
            log::error!("Failed to update user: {err}");
            if wants_json {
                return HttpResponse::InternalServerError().json(ApiMutationErrorDto {
                    message: "Ошибка при изменении пользователя.".to_string(),
                    field_errors: Vec::new(),
                });
            }
            return HttpResponse::InternalServerError().finish();
        }
    }
    redirect("/")
}

/// Handles `POST /hub/add` to create a hub for the current tenant.
#[post("/hub/add")]
pub async fn add_hub(
    request: HttpRequest,
    web::Form(form): web::Form<AddHubForm>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let wants_json = wants_json(&request);
    let payload = match AddHubPayload::try_from(form) {
        Ok(payload) => payload,
        Err(error) => {
            if wants_json {
                return HttpResponse::BadRequest().json(form_error_response(&error));
            }
            log::error!("Invalid hub data: {error}");
            return redirect("/");
        }
    };

    match admin_service::create_hub(payload, &current_user, repo.get_ref()) {
        Ok(_) => {
            if wants_json {
                return HttpResponse::Created().json(ApiMutationSuccessDto {
                    message: "Хаб добавлен.".to_string(),
                    redirect_to: None,
                });
            }
        }
        Err(ServiceError::Form(e)) => {
            log::error!("Invalid hub data: {e}");
            if wants_json {
                return HttpResponse::BadRequest().json(ApiMutationErrorDto {
                    message: "Ошибка валидации формы.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(ServiceError::Unauthorized) => {
            if wants_json {
                return HttpResponse::Forbidden().json(ApiMutationErrorDto {
                    message: "Недостаточно прав.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(err) => {
            log::error!("Failed to add hub: {err}");
            if wants_json {
                return HttpResponse::InternalServerError().json(ApiMutationErrorDto {
                    message: "Ошибка при добавлении хаба.".to_string(),
                    field_errors: Vec::new(),
                });
            }
            return HttpResponse::InternalServerError().finish();
        }
    }
    redirect("/")
}

/// Deletes a role via `POST /role/delete/{role_id}`.
#[post("/role/delete/{role_id}")]
pub async fn delete_role(
    request: HttpRequest,
    role_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let wants_json = wants_json(&request);
    let role_id = role_id.into_inner();

    match admin_service::delete_role_by_id(role_id, &current_user, repo.get_ref()) {
        Ok(_) => {
            if wants_json {
                return HttpResponse::Ok().json(ApiMutationSuccessDto {
                    message: "Роль удалена.".to_string(),
                    redirect_to: None,
                });
            }
        }
        Err(ServiceError::NotFound) => {
            if wants_json {
                return HttpResponse::NotFound().json(ApiMutationErrorDto {
                    message: "Роль не найдена.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(ServiceError::Unauthorized) => {
            if wants_json {
                return HttpResponse::Forbidden().json(ApiMutationErrorDto {
                    message: "Недостаточно прав.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(err) => {
            log::error!("Failed to delete role: {err}");
            if wants_json {
                return HttpResponse::InternalServerError().json(ApiMutationErrorDto {
                    message: "Ошибка при удалении роли.".to_string(),
                    field_errors: Vec::new(),
                });
            }
            return HttpResponse::InternalServerError().finish();
        }
    }
    redirect("/")
}

/// Removes a hub owned by the current tenant.
#[post("/hub/delete/{hub_id}")]
pub async fn delete_hub(
    request: HttpRequest,
    hub_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let wants_json = wants_json(&request);
    let hub_id = hub_id.into_inner();

    match admin_service::delete_hub_by_id(hub_id, &current_user, repo.get_ref()) {
        Ok(_) => {
            if wants_json {
                return HttpResponse::Ok().json(ApiMutationSuccessDto {
                    message: "Хаб удалён.".to_string(),
                    redirect_to: None,
                });
            }
        }
        Err(ServiceError::NotFound) => {
            if wants_json {
                return HttpResponse::NotFound().json(ApiMutationErrorDto {
                    message: "Хаб не найден.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(ServiceError::Unauthorized) => {
            if wants_json {
                return HttpResponse::Forbidden().json(ApiMutationErrorDto {
                    message: "Недостаточно прав.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(err) => {
            log::error!("Failed to delete hub: {err}");
            if wants_json {
                return HttpResponse::InternalServerError().json(ApiMutationErrorDto {
                    message: "Ошибка при удалении хаба.".to_string(),
                    field_errors: Vec::new(),
                });
            }
            return HttpResponse::InternalServerError().finish();
        }
    }
    redirect("/")
}

/// Handles `POST /menu/add` to create a menu entry.
#[post("/menu/add")]
pub async fn add_menu(
    request: HttpRequest,
    web::Form(form): web::Form<AddMenuForm>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let wants_json = wants_json(&request);
    let payload = match AddMenuPayload::try_from(form) {
        Ok(payload) => payload,
        Err(error) => {
            if wants_json {
                return HttpResponse::BadRequest().json(form_error_response(&error));
            }
            log::error!("Invalid menu data: {error}");
            return redirect("/");
        }
    };

    match admin_service::create_menu(payload, &current_user, repo.get_ref()) {
        Ok(_) => {
            if wants_json {
                return HttpResponse::Created().json(ApiMutationSuccessDto {
                    message: "Меню добавлено.".to_string(),
                    redirect_to: None,
                });
            }
        }
        Err(ServiceError::Form(e)) => {
            log::error!("Invalid menu data: {e}");
            if wants_json {
                return HttpResponse::BadRequest().json(ApiMutationErrorDto {
                    message: "Ошибка валидации формы.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(ServiceError::Unauthorized) => {
            if wants_json {
                return HttpResponse::Forbidden().json(ApiMutationErrorDto {
                    message: "Недостаточно прав.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(err) => {
            log::error!("Failed to add menu: {err}");
            if wants_json {
                return HttpResponse::InternalServerError().json(ApiMutationErrorDto {
                    message: "Ошибка при добавлении меню.".to_string(),
                    field_errors: Vec::new(),
                });
            }
            return HttpResponse::InternalServerError().finish();
        }
    }
    redirect("/")
}

/// Deletes a menu item via `POST /menu/delete/{menu_id}`.
#[post("/menu/delete/{menu_id}")]
pub async fn delete_menu(
    request: HttpRequest,
    menu_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let wants_json = wants_json(&request);
    let menu_id = menu_id.into_inner();
    match admin_service::delete_menu_by_id(menu_id, &current_user, repo.get_ref()) {
        Ok(_) => {
            if wants_json {
                return HttpResponse::Ok().json(ApiMutationSuccessDto {
                    message: "Меню удалено.".to_string(),
                    redirect_to: None,
                });
            }
        }
        Err(ServiceError::NotFound) => {
            if wants_json {
                return HttpResponse::NotFound().json(ApiMutationErrorDto {
                    message: "Меню не найдено.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(ServiceError::Unauthorized) => {
            if wants_json {
                return HttpResponse::Forbidden().json(ApiMutationErrorDto {
                    message: "Недостаточно прав.".to_string(),
                    field_errors: Vec::new(),
                });
            }
        }
        Err(err) => {
            log::error!("Failed to delete menu: {err}");
            if wants_json {
                return HttpResponse::InternalServerError().json(ApiMutationErrorDto {
                    message: "Ошибка при удалении меню.".to_string(),
                    field_errors: Vec::new(),
                });
            }
            return HttpResponse::InternalServerError().finish();
        }
    }
    redirect("/")
}
