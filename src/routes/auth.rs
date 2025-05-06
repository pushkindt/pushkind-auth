use actix_identity::Identity;
use actix_web::http::header;
use actix_web::{HttpMessage, Responder, get, post, web};
use actix_web::{HttpRequest, HttpResponse};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use log::error;
use tera::Context;

use crate::TEMPLATES;
use crate::db::DbPool;
use crate::domain::user::NewUser;
use crate::forms::auth::{LoginForm, RegisterForm};
use crate::repository::hub::DieselHubRepository;
use crate::repository::user::DieselUserRepository;
use crate::repository::{HubRepository, UserRepository};
use crate::routes::alert_level_to_str;

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

    match repo.get_by_email(&form.email, form.hub_id) {
        Ok(Some(user)) => {
            if form.verify_password(&user.password_hash) {
                if let Err(err) = Identity::login(&request.extensions(), user.id.to_string()) {
                    error!("Failed to log in user {}: {}", &form.email, err);
                    return HttpResponse::InternalServerError().finish();
                }

                HttpResponse::SeeOther()
                    .insert_header((header::LOCATION, "/"))
                    .finish()
            } else {
                FlashMessage::error("Неверный пароль.".to_string()).send();
                HttpResponse::SeeOther()
                    .insert_header((header::LOCATION, "/auth/signin"))
                    .finish()
            }
        }
        _ => {
            FlashMessage::error("Пользователь не существует.".to_string()).send();
            HttpResponse::SeeOther()
                .insert_header((header::LOCATION, "/auth/signin"))
                .finish()
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

    let new_user: NewUser = match form.try_into() {
        Ok(user) => user,
        Err(err) => {
            FlashMessage::error(format!("Ошибка при создании пользователя: {}", err)).send();
            return HttpResponse::SeeOther()
                .insert_header((header::LOCATION, "/auth/signup"))
                .finish();
        }
    };

    match repo.create(&new_user) {
        Ok(_) => {
            FlashMessage::success("Пользователь может войти.".to_string()).send();
        }
        Err(err) => {
            FlashMessage::error(format!("Ошибка при создании пользователя: {}", err)).send();
        }
    }
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, "/auth/signup"))
        .finish()
}

#[post("/logout")]
pub async fn logout(user: Identity) -> impl Responder {
    user.logout();
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, "/auth/signin"))
        .finish()
}

#[get("/signin")]
pub async fn signin(
    user: Option<Identity>,
    flash_messages: IncomingFlashMessages,
    pool: web::Data<DbPool>,
) -> impl Responder {
    if user.is_some() {
        return HttpResponse::SeeOther()
            .insert_header((header::LOCATION, "/"))
            .finish();
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
        return HttpResponse::SeeOther()
            .insert_header((header::LOCATION, "/"))
            .finish();
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
