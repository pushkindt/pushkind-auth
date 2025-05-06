use actix_identity::Identity;
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use actix_web::{Responder, get, post, web};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use log::error;
use tera::Context;

use crate::TEMPLATES;
use crate::db::DbPool;
use crate::forms::auth::{LoginForm, RegisterForm};
use crate::repository::hub::DieselHubRepository;
use crate::repository::user::DieselUserRepository;
use crate::repository::{HubRepository, UserRepository};
use crate::routes::{alert_level_to_str, redirect};

#[post("/login")]
pub async fn login(
    request: HttpRequest,
    pool: web::Data<DbPool>,
    web::Form(form): web::Form<LoginForm>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    let mut repo = DieselUserRepository::new(&mut conn);

    let user = match repo.login(&form.email, &form.password, form.hub_id) {
        Ok(Some(user)) => user,
        Ok(None) => {
            FlashMessage::error("Неверный логин или пароль.").send();
            return redirect("/auth/signin");
        }
        Err(e) => {
            error!("Login error: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };
    match Identity::login(&request.extensions(), user.id.to_string()) {
        Ok(_) => redirect("/"),
        Err(e) => {
            error!("Failed to login: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/register")]
pub async fn register(
    pool: web::Data<DbPool>,
    web::Form(form): web::Form<RegisterForm>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    let mut repo = DieselUserRepository::new(&mut conn);

    match repo.create(&form.into()) {
        Ok(_) => {
            FlashMessage::success("Пользователь может войти.".to_string()).send();
        }
        Err(err) => {
            FlashMessage::error(format!("Ошибка при создании пользователя: {}", err)).send();
        }
    }
    redirect("/auth/signup")
}

#[post("/logout")]
pub async fn logout(user: Identity) -> impl Responder {
    user.logout();
    redirect("/auth/signin")
}

#[get("/signin")]
pub async fn signin(
    user: Option<Identity>,
    flash_messages: IncomingFlashMessages,
    pool: web::Data<DbPool>,
) -> impl Responder {
    if user.is_some() {
        return redirect("/");
    }

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    let mut repo = DieselHubRepository::new(&mut conn);

    let hubs = match repo.list() {
        Ok(hubs) => hubs,
        Err(e) => {
            error!("Failed to get hubs: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut context = Context::new();

    let alerts = flash_messages
        .iter()
        .map(|f| (f.content(), alert_level_to_str(&f.level())))
        .collect::<Vec<_>>();

    context.insert("alerts", &alerts);
    context.insert("hubs", &hubs);

    HttpResponse::Ok().body(
        TEMPLATES
            .render("auth/login.html", &context)
            .unwrap_or_else(|e| {
                error!("Failed to render template 'auth/login.html': {}", e);
                String::new()
            }),
    )
}

#[get("/signup")]
pub async fn signup(
    user: Option<Identity>,
    flash_messages: IncomingFlashMessages,
    pool: web::Data<DbPool>,
) -> impl Responder {
    if user.is_some() {
        return redirect("/");
    }

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    let mut repo = DieselHubRepository::new(&mut conn);

    let hubs = match repo.list() {
        Ok(hubs) => hubs,
        Err(e) => {
            error!("Failed to get hubs: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut context = Context::new();

    let alerts = flash_messages
        .iter()
        .map(|f| (f.content(), alert_level_to_str(&f.level())))
        .collect::<Vec<_>>();

    context.insert("alerts", &alerts);
    context.insert("hubs", &hubs);

    HttpResponse::Ok().body(
        TEMPLATES
            .render("auth/register.html", &context)
            .unwrap_or_else(|e| {
                error!("Failed to render template 'auth/register.html': {}", e);
                String::new()
            }),
    )
}
