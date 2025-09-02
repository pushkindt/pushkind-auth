//! Administrative endpoints used to manage users, roles and hubs.

use actix_web::{HttpResponse, Responder, post, web};
use actix_web_flash_messages::FlashMessage;
use log::error;
use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::routes::render_template;
use pushkind_common::routes::{ensure_role, redirect};
use tera::{Context, Tera};

use crate::domain::hub::NewHub;
use crate::domain::role::NewRole;
use crate::forms::main::{AddHubForm, AddMenuForm, AddRoleForm, UpdateUserForm};
use crate::repository::{
    DieselRepository, HubWriter, MenuReader, MenuWriter, RoleReader, RoleWriter, UserReader,
    UserWriter,
};

#[post("/role/add")]
pub async fn add_role(
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
    web::Form(form): web::Form<AddRoleForm>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&current_user, "admin", None) {
        return resp;
    }

    let new_role: NewRole = form.into();

    match repo.create_role(&new_role) {
        Ok(_) => {
            FlashMessage::success("Роль добавлена.").send();
        }
        Err(err) => {
            log::error!("Failed to add role: {err}");
            FlashMessage::error("Ошибка при добавлении роли").send();
        }
    }
    redirect("/")
}

#[post("/user/modal/{user_id}")]
pub async fn user_modal(
    user_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
    tera: web::Data<Tera>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&current_user, "admin", None) {
        return resp;
    }

    let mut context = Context::new();

    let user_id = user_id.into_inner();

    if let Ok(Some(user)) = repo.get_user_by_id(user_id, current_user.hub_id) {
        context.insert("user", &user.user);
    }

    if let Ok(roles) = repo.list_roles() {
        context.insert("roles", &roles);
    }

    render_template(&tera, "main/modal_body.html", &context)
}

#[post("/user/delete/{user_id}")]
pub async fn delete_user(
    user_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&current_user, "admin", None) {
        return resp;
    }

    let user = match repo.get_user_by_id(user_id.into_inner(), current_user.hub_id) {
        Ok(Some(user)) => user.user,
        Ok(None) => {
            FlashMessage::error("Пользователь не найден.").send();
            return redirect("/");
        }
        Err(err) => {
            log::error!("Failed to get user: {err}");
            FlashMessage::error("Ошибка при получении пользователя").send();
            return redirect("/");
        }
    };

    let current_user_id: i32 = match current_user.sub.parse() {
        Ok(user_id) => user_id,
        Err(e) => {
            error!("Failed to parse user_id: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    if user.id == current_user_id {
        FlashMessage::error("Недостаточно прав.").send();
        return redirect("/");
    }

    match repo.delete_user(user.id) {
        Ok(_) => {
            FlashMessage::success("Пользователь удалён.").send();
        }
        Err(err) => {
            log::error!("Failed to delete user: {err}");
            FlashMessage::error("Ошибка при удалении пользователя").send();
        }
    }
    redirect("/")
}

#[post("/user/update")]
pub async fn update_user(
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
    form: web::Bytes,
) -> impl Responder {
    if let Err(resp) = ensure_role(&current_user, "admin", None) {
        return resp;
    }

    let form: UpdateUserForm = match serde_html_form::from_bytes(&form) {
        Ok(form) => form,
        Err(err) => {
            log::error!("Failed to process form: {err}");
            FlashMessage::error("Ошибка при обработке формы").send();
            return redirect("/");
        }
    };

    let user = match repo.get_user_by_id(form.id, current_user.hub_id) {
        Ok(Some(user)) => user.user,
        Ok(None) => {
            FlashMessage::error("Пользователь не найден.").send();
            return redirect("/");
        }
        Err(err) => {
            log::error!("Failed to get user: {err}");
            FlashMessage::error("Ошибка при получении пользователя").send();
            return redirect("/");
        }
    };

    match repo.assign_roles_to_user(form.id, &form.roles) {
        Ok(_) => {
            FlashMessage::success("Роли назначены.").send();
        }
        Err(err) => {
            log::error!("Failed to assign roles: {err}");
            FlashMessage::error("Ошибка при назначении ролей").send();
        }
    }

    let update_user = form.into();

    match repo.update_user(user.id, user.hub_id, &update_user) {
        Ok(_) => {
            FlashMessage::success("Пользователь изменён.").send();
        }
        Err(err) => {
            log::error!("Failed to update user: {err}");
            FlashMessage::error("Ошибка при изменении пользователя").send();
        }
    }
    redirect("/")
}

#[post("/hub/add")]
pub async fn add_hub(
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
    web::Form(form): web::Form<AddHubForm>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&current_user, "admin", None) {
        return resp;
    }

    let new_hub: NewHub = form.into();

    match repo.create_hub(&new_hub) {
        Ok(_) => {
            FlashMessage::success("Хаб добавлен.").send();
        }
        Err(err) => {
            log::error!("Failed to add hub: {err}");
            FlashMessage::error("Ошибка при добавлении хаба").send();
        }
    }
    redirect("/")
}

#[post("/role/delete/{role_id}")]
pub async fn delete_role(
    role_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&current_user, "admin", None) {
        return resp;
    }

    let role_id = role_id.into_inner();

    if role_id == 1 {
        FlashMessage::error("Недостаточно прав.").send();
        return redirect("/");
    }

    match repo.delete_role(role_id) {
        Ok(_) => {
            FlashMessage::success("Роль удалена.").send();
        }
        Err(err) => {
            log::error!("Failed to delete role: {err}");
            FlashMessage::error("Ошибка при удалении роли").send();
        }
    }
    redirect("/")
}

#[post("/hub/delete/{hub_id}")]
pub async fn delete_hub(
    hub_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&current_user, "admin", None) {
        return resp;
    }

    let hub_id = hub_id.into_inner();

    if current_user.hub_id == hub_id {
        FlashMessage::error("Недостаточно прав.").send();
        return redirect("/");
    }

    match repo.delete_hub(hub_id) {
        Ok(_) => {
            FlashMessage::success("Хаб удалён.").send();
        }
        Err(err) => {
            log::error!("Failed to delete hub: {err}");
            FlashMessage::error("Ошибка при удалении хаба").send();
        }
    }
    redirect("/")
}

#[post("/menu/add")]
pub async fn add_menu(
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
    web::Form(form): web::Form<AddMenuForm>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&current_user, "admin", None) {
        return resp;
    }

    let new_menu = form.to_new_menu(current_user.hub_id);

    match repo.create_menu(&new_menu) {
        Ok(_) => {
            FlashMessage::success("Меню добавлено.").send();
        }
        Err(err) => {
            log::error!("Failed to add menu: {err}");
            FlashMessage::error("Ошибка при добавлении меню").send();
        }
    }
    redirect("/")
}

#[post("/menu/delete/{menu_id}")]
pub async fn delete_menu(
    menu_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&current_user, "admin", None) {
        return resp;
    }

    let menu = match repo.get_menu_by_id(menu_id.into_inner(), current_user.hub_id) {
        Ok(Some(menu)) => menu,
        Ok(None) => {
            FlashMessage::error("Меню не найдено.").send();
            return redirect("/");
        }
        Err(err) => {
            log::error!("Failed to get menu: {err}");
            FlashMessage::error("Ошибка при получении меню").send();
            return redirect("/");
        }
    };

    match repo.delete_menu(menu.id) {
        Ok(_) => {
            FlashMessage::success("Меню удалено.").send();
        }
        Err(err) => {
            log::error!("Failed to delete menu: {err}");
            FlashMessage::error("Ошибка при удалении меню").send();
        }
    }
    redirect("/")
}
