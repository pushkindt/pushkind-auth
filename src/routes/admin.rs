//! Administrative endpoints used to manage users, roles and hubs.

use actix_web::{HttpResponse, Responder, post, web};
use actix_web_flash_messages::FlashMessage;
use log::error;
use pushkind_common::db::DbPool;
use pushkind_common::models::auth::AuthenticatedUser;
use pushkind_common::routes::render_template;
use pushkind_common::routes::{ensure_role, redirect};
use tera::{Context, Tera};

use crate::domain::hub::NewHub;
use crate::domain::menu::NewMenu;
use crate::domain::role::NewRole;
use crate::forms::main::{AddHubForm, AddMenuForm, AddRoleForm, UpdateUserForm};
use crate::repository::hub::DieselHubRepository;
use crate::repository::menu::DieselMenuRepository;
use crate::repository::role::DieselRoleRepository;
use crate::repository::user::DieselUserRepository;
use crate::repository::{HubWriter, MenuWriter, RoleReader, RoleWriter, UserReader, UserWriter};

#[post("/role/add")]
pub async fn add_role(
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
    web::Form(form): web::Form<AddRoleForm>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&user, "admin", None) {
        return resp;
    }

    let repo = DieselRoleRepository::new(&pool);

    let new_role: NewRole = (&form).into();

    match repo.create(&new_role) {
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
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
    tera: web::Data<Tera>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&user, "admin", None) {
        return resp;
    }

    let mut context = Context::new();

    let user_id = user_id.into_inner();

    let repo = DieselUserRepository::new(&pool);

    if let Ok(Some(user)) = repo.get_by_id(user_id) {
        context.insert("user", &user.user);
    }

    let repo = DieselRoleRepository::new(&pool);
    if let Ok(roles) = repo.list() {
        context.insert("roles", &roles);
    }

    render_template(&tera, "main/modal_body.html", &context)
}

#[post("/user/delete/{user_id}")]
pub async fn delete_user(
    user_id: web::Path<i32>,
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&user, "admin", None) {
        return resp;
    }

    let user_id = user_id.into_inner();

    let current_user_id: i32 = match user.sub.parse() {
        Ok(user_id) => user_id,
        Err(e) => {
            error!("Failed to parse user_id: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    if user_id == current_user_id {
        FlashMessage::error("Недостаточно прав.").send();
        return redirect("/");
    }

    let repo = DieselUserRepository::new(&pool);

    match repo.delete(user_id) {
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
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
    form: web::Bytes,
) -> impl Responder {
    if let Err(resp) = ensure_role(&user, "admin", None) {
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

    let repo = DieselUserRepository::new(&pool);
    match repo.assign_roles(form.id, &form.roles) {
        Ok(_) => {
            FlashMessage::success("Роли назначены.").send();
        }
        Err(err) => {
            log::error!("Failed to assign roles: {err}");
            FlashMessage::error("Ошибка при назначении ролей").send();
        }
    }

    let update_user = (&form).into();

    match repo.update(form.id, &update_user) {
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
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
    web::Form(form): web::Form<AddHubForm>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&user, "admin", None) {
        return resp;
    }

    let repo = DieselHubRepository::new(&pool);

    let new_hub: NewHub = (&form).into();

    match repo.create(&new_hub) {
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
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&user, "admin", None) {
        return resp;
    }

    let role_id = role_id.into_inner();

    if role_id == 1 {
        FlashMessage::error("Недостаточно прав.").send();
        return redirect("/");
    }

    let repo = DieselRoleRepository::new(&pool);

    match repo.delete(role_id) {
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
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&user, "admin", None) {
        return resp;
    }

    let hub_id = hub_id.into_inner();

    if user.hub_id == hub_id {
        FlashMessage::error("Недостаточно прав.").send();
        return redirect("/");
    }

    let repo = DieselHubRepository::new(&pool);

    match repo.delete(hub_id) {
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
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
    web::Form(form): web::Form<AddMenuForm>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&user, "admin", None) {
        return resp;
    }

    let repo = DieselMenuRepository::new(&pool);

    let new_menu = NewMenu {
        name: form.name.as_str(),
        url: form.url.as_str(),
        hub_id: user.hub_id,
    };

    match repo.create(&new_menu) {
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
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
) -> impl Responder {
    if let Err(resp) = ensure_role(&user, "admin", None) {
        return resp;
    }

    let menu_id = menu_id.into_inner();

    let repo = DieselMenuRepository::new(&pool);

    match repo.delete(menu_id) {
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
