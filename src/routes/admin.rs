//! Administrative endpoints used to manage users, roles and hubs.

use actix_web::{HttpResponse, Responder, post, web};
use actix_web_flash_messages::FlashMessage;
use log::error;
use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::routes::redirect;
use pushkind_common::routes::render_template;
use pushkind_common::services::errors::ServiceError;
use tera::{Context, Tera};

use crate::dto::admin::UserModalData;
use crate::forms::main::{AddHubForm, AddMenuForm, AddRoleForm, UpdateUserForm};
use crate::repository::DieselRepository;
use crate::services::admin as admin_service;

/// Handles `POST /role/add` to create a new role and flash the outcome.
#[post("/role/add")]
pub async fn add_role(
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
    web::Form(form): web::Form<AddRoleForm>,
) -> impl Responder {
    match admin_service::create_role(&current_user, form, repo.get_ref()) {
        Ok(_) => {
            FlashMessage::success("Роль добавлена.").send();
        }
        Err(ServiceError::Form(e)) => {
            log::error!("Invalid role data: {e}");
            FlashMessage::error("Ошибка валидации формы").send();
        }
        Err(ServiceError::Conflict) => {
            FlashMessage::error("Роль уже существует.").send();
        }
        Err(ServiceError::Unauthorized) => {
            FlashMessage::error("Недостаточно прав.").send();
        }
        Err(err) => {
            log::error!("Failed to add role: {err}");
            FlashMessage::error("Ошибка при добавлении роли").send();
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
    tera: web::Data<Tera>,
) -> impl Responder {
    let mut context = Context::new();

    let user_id = user_id.into_inner();

    match admin_service::user_modal_data(&current_user, user_id, repo.get_ref()) {
        Ok(data) => {
            let UserModalData { user, roles } = data;
            if let Some(user) = user {
                context.insert("user", &user);
            }
            context.insert("roles", &roles);
        }
        Err(ServiceError::Unauthorized) => {
            FlashMessage::error("Недостаточно прав.").send();
            return HttpResponse::Unauthorized().finish();
        }
        Err(e) => {
            error!("Failed to build user modal data: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    }

    render_template(&tera, "main/modal_body.html", &context)
}

/// Deletes a user by id for `POST /user/delete/{user_id}`.
#[post("/user/delete/{user_id}")]
pub async fn delete_user(
    user_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let target_id = user_id.into_inner();

    match admin_service::delete_user_by_id(&current_user, target_id, repo.get_ref()) {
        Ok(_) => {
            FlashMessage::success("Пользователь удалён.").send();
        }
        Err(ServiceError::NotFound) => {
            FlashMessage::error("Пользователь не найден.").send();
        }
        Err(ServiceError::Unauthorized) => {
            FlashMessage::error("Недостаточно прав.").send();
        }
        Err(err) => {
            log::error!("Failed to delete user: {err}");
            FlashMessage::error("Ошибка при удалении пользователя").send();
        }
    }
    redirect("/")
}

/// Updates user data and role assignments from the admin form.
#[post("/user/update/{user_id}")]
pub async fn update_user(
    user_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
    form: web::Bytes,
) -> impl Responder {
    let form: UpdateUserForm = match serde_html_form::from_bytes(&form) {
        Ok(form) => form,
        Err(err) => {
            log::error!("Failed to process form: {err}");
            FlashMessage::error("Ошибка при обработке формы").send();
            return redirect("/");
        }
    };
    let target_id = user_id.into_inner();

    match admin_service::assign_roles_and_update_user(
        &current_user,
        target_id,
        form,
        repo.get_ref(),
    ) {
        Ok(_) => {
            FlashMessage::success("Пользователь изменён.").send();
        }
        Err(ServiceError::Form(e)) => {
            log::error!("Invalid user data: {e}");
            FlashMessage::error("Ошибка валидации формы").send();
        }
        Err(ServiceError::NotFound) => {
            FlashMessage::error("Пользователь не найден.").send();
        }
        Err(ServiceError::Unauthorized) => {
            FlashMessage::error("Недостаточно прав.").send();
        }
        Err(err) => {
            log::error!("Failed to update user: {err}");
            return HttpResponse::InternalServerError().finish();
        }
    }
    redirect("/")
}

/// Handles `POST /hub/add` to create a hub for the current tenant.
#[post("/hub/add")]
pub async fn add_hub(
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
    web::Form(form): web::Form<AddHubForm>,
) -> impl Responder {
    match admin_service::create_hub(&current_user, form, repo.get_ref()) {
        Ok(_) => {
            FlashMessage::success("Хаб добавлен.").send();
        }
        Err(ServiceError::Form(e)) => {
            log::error!("Invalid hub data: {e}");
            FlashMessage::error("Ошибка валидации формы").send();
        }
        Err(ServiceError::Unauthorized) => {
            FlashMessage::error("Недостаточно прав.").send();
        }
        Err(err) => {
            log::error!("Failed to add hub: {err}");
            return HttpResponse::InternalServerError().finish();
        }
    }
    redirect("/")
}

/// Deletes a role via `POST /role/delete/{role_id}`.
#[post("/role/delete/{role_id}")]
pub async fn delete_role(
    role_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let role_id = role_id.into_inner();

    match admin_service::delete_role_by_id(&current_user, role_id, repo.get_ref()) {
        Ok(_) => {
            FlashMessage::success("Роль удалена.").send();
        }
        Err(ServiceError::NotFound) => {
            FlashMessage::error("Роль не найдена.").send();
        }
        Err(ServiceError::Unauthorized) => {
            FlashMessage::error("Недостаточно прав.").send();
        }
        Err(err) => {
            log::error!("Failed to delete role: {err}");
            return HttpResponse::InternalServerError().finish();
        }
    }
    redirect("/")
}

/// Removes a hub owned by the current tenant.
#[post("/hub/delete/{hub_id}")]
pub async fn delete_hub(
    hub_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let hub_id = hub_id.into_inner();

    match admin_service::delete_hub_by_id(&current_user, hub_id, repo.get_ref()) {
        Ok(_) => {
            FlashMessage::success("Хаб удалён.").send();
        }
        Err(ServiceError::NotFound) => {
            FlashMessage::error("Хаб не найден.").send();
        }
        Err(ServiceError::Unauthorized) => {
            FlashMessage::error("Недостаточно прав.").send();
        }
        Err(err) => {
            log::error!("Failed to delete hub: {err}");
            return HttpResponse::InternalServerError().finish();
        }
    }
    redirect("/")
}

/// Handles `POST /menu/add` to create a menu entry.
#[post("/menu/add")]
pub async fn add_menu(
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
    web::Form(form): web::Form<AddMenuForm>,
) -> impl Responder {
    match admin_service::create_menu(&current_user, form, repo.get_ref()) {
        Ok(_) => {
            FlashMessage::success("Меню добавлено.").send();
        }
        Err(ServiceError::Form(e)) => {
            log::error!("Invalid menu data: {e}");
            FlashMessage::error("Ошибка валидации формы").send();
        }
        Err(ServiceError::Unauthorized) => {
            FlashMessage::error("Недостаточно прав.").send();
        }
        Err(err) => {
            log::error!("Failed to add menu: {err}");
            return HttpResponse::InternalServerError().finish();
        }
    }
    redirect("/")
}

/// Deletes a menu item via `POST /menu/delete/{menu_id}`.
#[post("/menu/delete/{menu_id}")]
pub async fn delete_menu(
    menu_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let menu_id = menu_id.into_inner();
    match admin_service::delete_menu_by_id(&current_user, menu_id, repo.get_ref()) {
        Ok(_) => {
            FlashMessage::success("Меню удалено.").send();
        }
        Err(ServiceError::NotFound) => {
            FlashMessage::error("Меню не найдено.").send();
        }
        Err(ServiceError::Unauthorized) => {
            FlashMessage::error("Недостаточно прав.").send();
        }
        Err(err) => {
            log::error!("Failed to delete menu: {err}");
            return HttpResponse::InternalServerError().finish();
        }
    }
    redirect("/")
}
