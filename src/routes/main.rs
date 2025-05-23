use actix_web::{HttpResponse, Responder, get, post, web};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use log::error;
use tera::Context;

use crate::TEMPLATES;
use crate::db::DbPool;
use crate::forms::main::{AddHubForm, AddRoleForm, SaveUserForm, UpdateUserForm};
use crate::models::auth::AuthenticatedUser;
use crate::repository::hub::DieselHubRepository;
use crate::repository::role::DieselRoleRepository;
use crate::repository::user::DieselUserRepository;
use crate::repository::{HubRepository, RoleRepository, UserRepository};
use crate::routes::{alert_level_to_str, redirect};

#[get("/")]
pub async fn index(
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
    flash_messages: IncomingFlashMessages,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let user_id: i32 = match user.sub.parse() {
        Ok(user_id) => user_id,
        Err(e) => {
            error!("Failed to parse user_id: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut repo = DieselUserRepository::new(&mut conn);

    let users = match repo.list(user.hub_id) {
        Ok(users) => users,
        Err(e) => {
            error!("Failed to list users: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let user = match repo.get_by_id(user_id) {
        Ok(Some(user)) => user,
        Ok(None) => {
            error!("User not found");
            return HttpResponse::InternalServerError().finish();
        }
        Err(e) => {
            error!("Failed to get user: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let user_roles = match repo.get_roles(user_id) {
        Ok(user_roles) => user_roles
            .into_iter()
            .map(|r| r.name)
            .collect::<Vec<String>>(),
        Err(e) => {
            error!("Failed to get user roles: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut repo = DieselRoleRepository::new(&mut conn);

    let roles = match repo.list() {
        Ok(roles) => roles,
        Err(e) => {
            error!("Failed to list roles: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut repo = DieselHubRepository::new(&mut conn);

    let hubs = match repo.list() {
        Ok(hubs) => hubs,
        Err(e) => {
            error!("Failed to list hubs: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let alerts = flash_messages
        .iter()
        .map(|f| (f.content(), alert_level_to_str(&f.level())))
        .collect::<Vec<_>>();

    let mut context = Context::new();
    context.insert("alerts", &alerts);
    context.insert("current_user", &user);
    context.insert("current_user_roles", &user_roles);
    context.insert("current_page", "index");
    context.insert("users", &users);
    context.insert("roles", &roles);
    context.insert("hubs", &hubs);

    HttpResponse::Ok().body(
        TEMPLATES
            .render("main/index.html", &context)
            .unwrap_or_else(|e| {
                error!("Failed to render template 'main/index.html': {}", e);
                String::new()
            }),
    )
}

#[post("/user/save")]
pub async fn save_user(
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
    web::Form(form): web::Form<SaveUserForm>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    let mut repo = DieselUserRepository::new(&mut conn);

    let user_id = match user.sub.parse() {
        Ok(user_id) => user_id,
        Err(e) => {
            error!("Failed to parse user_id: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match repo.update(user_id, &form.into()) {
        Ok(_) => {
            FlashMessage::success("Параметры изменены.".to_string()).send();
        }
        Err(err) => {
            FlashMessage::error(format!("Ошибка при изменений параметров: {}", err)).send();
        }
    }
    redirect("/")
}

#[post("/role/add")]
pub async fn add_role(
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
    web::Form(form): web::Form<AddRoleForm>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if !user.roles.iter().any(|role| role == "admin") {
        FlashMessage::error("Недостаточно прав.".to_string()).send();
        return redirect("/");
    }

    let mut repo = DieselRoleRepository::new(&mut conn);

    match repo.create(&form.into()) {
        Ok(_) => {
            FlashMessage::success("Роль добавлена.".to_string()).send();
        }
        Err(err) => {
            FlashMessage::error(format!("Ошибка при добавлении роли: {}", err)).send();
        }
    }
    redirect("/")
}

#[post("/user/modal/{user_id}")]
pub async fn user_modal(
    user_id: web::Path<i32>,
    admin_user: AuthenticatedUser,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if !admin_user.roles.iter().any(|role| role == "admin") {
        return HttpResponse::InternalServerError().finish();
    }

    let mut context = Context::new();

    let user_id = user_id.into_inner();

    let mut repo = DieselUserRepository::new(&mut conn);

    if let Ok(Some(user)) = repo.get_by_id(user_id) {
        context.insert("user", &user);

        if let Ok(user_roles) = repo.get_roles(user_id) {
            let user_roles = user_roles
                .into_iter()
                .map(|r| r.name)
                .collect::<Vec<String>>();
            context.insert("user_roles", &user_roles);
        }
    }

    let mut repo = DieselRoleRepository::new(&mut conn);
    if let Ok(roles) = repo.list() {
        context.insert("roles", &roles);
    }

    HttpResponse::Ok().body(
        TEMPLATES
            .render("main/modal_body.html", &context)
            .unwrap_or_else(|e| {
                error!("Failed to render template 'main/modal_body.html': {}", e);
                String::new()
            }),
    )
}

#[post("/user/delete/{user_id}")]
pub async fn delete_user(
    user_id: web::Path<i32>,
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let user_id = user_id.into_inner();

    let current_user_id: i32 = match user.sub.parse() {
        Ok(user_id) => user_id,
        Err(e) => {
            error!("Failed to parse user_id: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if !user.roles.iter().any(|role| role == "admin") || user_id == current_user_id {
        FlashMessage::error("Недостаточно прав.".to_string()).send();
        return redirect("/");
    }

    let mut repo = DieselUserRepository::new(&mut conn);

    match repo.delete(user_id) {
        Ok(_) => {
            FlashMessage::success("Пользователь удалён.".to_string()).send();
        }
        Err(err) => {
            FlashMessage::error(format!("Ошибка при удалении пользователя: {}", err)).send();
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
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if !user.roles.iter().any(|role| role == "admin") {
        FlashMessage::error("Недостаточно прав.".to_string()).send();
        return redirect("/");
    }

    let form: UpdateUserForm = match serde_html_form::from_bytes(&form) {
        Ok(form) => form,
        Err(err) => {
            FlashMessage::error(format!("Ошибка при обработке формы: {}", err)).send();
            return redirect("/");
        }
    };

    let mut repo = DieselUserRepository::new(&mut conn);
    match repo.assign_roles(form.id, &form.roles) {
        Ok(_) => {
            FlashMessage::success("Роли назначены.".to_string()).send();
        }
        Err(err) => {
            FlashMessage::error(format!("Ошибка при назначении ролей: {}", err)).send();
        }
    }

    match repo.update(form.id, &form.into()) {
        Ok(_) => {
            FlashMessage::success("Пользователь изменен.".to_string()).send();
        }
        Err(err) => {
            FlashMessage::error(format!("Ошибка при изменений пользователя: {}", err)).send();
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
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if !user.roles.iter().any(|role| role == "admin") {
        FlashMessage::error("Недостаточно прав.".to_string()).send();
        return redirect("/");
    }

    let mut repo = DieselHubRepository::new(&mut conn);

    match repo.create(&form.into()) {
        Ok(_) => {
            FlashMessage::success("Хаб добавлен.".to_string()).send();
        }
        Err(err) => {
            FlashMessage::error(format!("Ошибка при добавлении хаба: {}", err)).send();
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
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let role_id = role_id.into_inner();

    if !user.roles.iter().any(|role| role == "admin") || role_id == 1 {
        FlashMessage::error("Недостаточно прав.".to_string()).send();
        return redirect("/");
    }

    let mut repo = DieselRoleRepository::new(&mut conn);

    match repo.delete(role_id) {
        Ok(_) => {
            FlashMessage::success("Роль удалёна.".to_string()).send();
        }
        Err(err) => {
            FlashMessage::error(format!("Ошибка при удалении роли: {}", err)).send();
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
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let hub_id = hub_id.into_inner();

    if !user.roles.iter().any(|role| role == "admin") || hub_id == 1 || user.hub_id == hub_id {
        FlashMessage::error("Недостаточно прав.".to_string()).send();
        return redirect("/");
    }

    let mut repo = DieselHubRepository::new(&mut conn);

    match repo.delete(hub_id) {
        Ok(_) => {
            FlashMessage::success("Хаб удалён.".to_string()).send();
        }
        Err(err) => {
            FlashMessage::error(format!("Ошибка при удалении хаба: {}", err)).send();
        }
    }
    redirect("/")
}
