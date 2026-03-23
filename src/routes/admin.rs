//! Administrative endpoints used to manage users, roles and hubs.

use actix_web::{HttpResponse, Responder, post, web};
use log::error;
use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::services::errors::ServiceError;

use crate::dto::admin::UserModalData;
use crate::dto::api::{ApiMutationErrorDto, ApiMutationSuccessDto};
use crate::dto::frontend::{AdminEditableUserDto, AdminUserModalBootstrap, RoleOptionDto};
use crate::forms::main::{
    AddHubForm, AddHubPayload, AddMenuForm, AddMenuPayload, AddRoleForm, AddRolePayload,
    UpdateUserForm, UpdateUserPayload,
};
use crate::repository::DieselRepository;
use crate::routes::{MutationResource, mutation_error_response};
use crate::services::admin as admin_service;

/// Handles `POST /role/add` to create a new role and flash the outcome.
#[post("/role/add")]
pub async fn add_role(
    web::Form(form): web::Form<AddRoleForm>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let payload = match AddRolePayload::try_from(form) {
        Ok(payload) => payload,
        Err(error) => {
            log::error!("Invalid role data: {error}");
            return HttpResponse::BadRequest().json(ApiMutationErrorDto::from(&error));
        }
    };

    match admin_service::create_role(payload, &current_user, repo.get_ref()) {
        Ok(_) => HttpResponse::Created().json(ApiMutationSuccessDto {
            message: "Роль добавлена.".to_string(),
            redirect_to: None,
        }),
        Err(err) => {
            log::error!("Failed to add role: {err}");
            mutation_error_response(MutationResource::Role, &err)
        }
    }
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
    user_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let target_id = user_id.into_inner();

    match admin_service::delete_user_by_id(target_id, &current_user, repo.get_ref()) {
        Ok(_) => HttpResponse::Ok().json(ApiMutationSuccessDto {
            message: "Пользователь удалён.".to_string(),
            redirect_to: None,
        }),
        Err(err) => {
            log::error!("Failed to delete user: {err}");
            mutation_error_response(MutationResource::User, &err)
        }
    }
}

/// Updates user data and role assignments from the admin form.
#[post("/user/update/{user_id}")]
pub async fn update_user(
    user_id: web::Path<i32>,
    form: web::Bytes,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let form: UpdateUserForm = match serde_html_form::from_bytes(&form) {
        Ok(form) => form,
        Err(err) => {
            log::error!("Failed to process form: {err}");

            return HttpResponse::BadRequest().json(ApiMutationErrorDto {
                message: "Ошибка при обработке формы.".to_string(),
                field_errors: Vec::new(),
            });
        }
    };
    let target_id = user_id.into_inner();
    let payload = match UpdateUserPayload::try_from(form) {
        Ok(payload) => payload,
        Err(error) => {
            log::error!("Invalid user data: {error}");
            return HttpResponse::BadRequest().json(ApiMutationErrorDto::from(&error));
        }
    };

    match admin_service::assign_roles_and_update_user(
        target_id,
        payload,
        &current_user,
        repo.get_ref(),
    ) {
        Ok(_) => HttpResponse::Ok().json(ApiMutationSuccessDto {
            message: "Пользователь изменён.".to_string(),
            redirect_to: None,
        }),
        Err(err) => {
            log::error!("Failed to update user: {err}");
            mutation_error_response(MutationResource::User, &err)
        }
    }
}

/// Handles `POST /hub/add` to create a hub for the current tenant.
#[post("/hub/add")]
pub async fn add_hub(
    web::Form(form): web::Form<AddHubForm>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let payload = match AddHubPayload::try_from(form) {
        Ok(payload) => payload,
        Err(error) => {
            log::error!("Invalid hub data: {error}");
            return HttpResponse::BadRequest().json(ApiMutationErrorDto::from(&error));
        }
    };

    match admin_service::create_hub(payload, &current_user, repo.get_ref()) {
        Ok(_) => HttpResponse::Created().json(ApiMutationSuccessDto {
            message: "Хаб добавлен.".to_string(),
            redirect_to: None,
        }),
        Err(err) => {
            log::error!("Failed to add hub: {err}");
            mutation_error_response(MutationResource::Hub, &err)
        }
    }
}

/// Deletes a role via `POST /role/delete/{role_id}`.
#[post("/role/delete/{role_id}")]
pub async fn delete_role(
    role_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let role_id = role_id.into_inner();

    match admin_service::delete_role_by_id(role_id, &current_user, repo.get_ref()) {
        Ok(_) => HttpResponse::Ok().json(ApiMutationSuccessDto {
            message: "Роль удалена.".to_string(),
            redirect_to: None,
        }),
        Err(err) => {
            log::error!("Failed to delete role: {err}");
            mutation_error_response(MutationResource::Role, &err)
        }
    }
}

/// Removes a hub owned by the current tenant.
#[post("/hub/delete/{hub_id}")]
pub async fn delete_hub(
    hub_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let hub_id = hub_id.into_inner();

    match admin_service::delete_hub_by_id(hub_id, &current_user, repo.get_ref()) {
        Ok(_) => HttpResponse::Ok().json(ApiMutationSuccessDto {
            message: "Хаб удалён.".to_string(),
            redirect_to: None,
        }),
        Err(err) => {
            log::error!("Failed to delete hub: {err}");
            mutation_error_response(MutationResource::Hub, &err)
        }
    }
}

/// Handles `POST /menu/add` to create a menu entry.
#[post("/menu/add")]
pub async fn add_menu(
    web::Form(form): web::Form<AddMenuForm>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let payload = match AddMenuPayload::try_from(form) {
        Ok(payload) => payload,
        Err(error) => {
            log::error!("Invalid menu data: {error}");
            return HttpResponse::BadRequest().json(ApiMutationErrorDto::from(&error));
        }
    };

    match admin_service::create_menu(payload, &current_user, repo.get_ref()) {
        Ok(_) => HttpResponse::Created().json(ApiMutationSuccessDto {
            message: "Меню добавлено.".to_string(),
            redirect_to: None,
        }),
        Err(err) => {
            log::error!("Failed to add menu: {err}");
            mutation_error_response(MutationResource::Menu, &err)
        }
    }
}

/// Deletes a menu item via `POST /menu/delete/{menu_id}`.
#[post("/menu/delete/{menu_id}")]
pub async fn delete_menu(
    menu_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let menu_id = menu_id.into_inner();
    match admin_service::delete_menu_by_id(menu_id, &current_user, repo.get_ref()) {
        Ok(_) => HttpResponse::Ok().json(ApiMutationSuccessDto {
            message: "Меню удалено.".to_string(),
            redirect_to: None,
        }),
        Err(err) => {
            log::error!("Failed to delete menu: {err}");
            mutation_error_response(MutationResource::Menu, &err)
        }
    }
}
