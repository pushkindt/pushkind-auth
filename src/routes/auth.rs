//! Authentication and session management endpoints.

use actix_identity::Identity;
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use actix_web::{Responder, get, post, web};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use log::error;
use pushkind_common::models::auth::AuthenticatedUser;
use pushkind_common::models::config::CommonServerConfig;
use pushkind_common::routes::render_template;
use pushkind_common::routes::{alert_level_to_str, redirect};
use serde::Deserialize;
use tera::{Context, Tera};
use validator::Validate;

use crate::forms::auth::{LoginForm, RegisterForm};
use crate::models::config::ServerConfig;
use crate::repository::{DieselRepository, HubReader, UserReader, UserWriter};
use crate::routes::get_success_and_failure_redirects;

#[derive(Deserialize)]
struct AuthQueryParams {
    next: Option<String>,
}

#[post("/login")]
pub async fn login(
    request: HttpRequest,
    repo: web::Data<DieselRepository>,
    server_config: web::Data<ServerConfig>,
    common_config: web::Data<CommonServerConfig>,
    web::Form(form): web::Form<LoginForm>,
    query_params: web::Query<AuthQueryParams>,
) -> impl Responder {
    let (success_redirect_url, failure_redirect_url) = get_success_and_failure_redirects(
        "/auth/signin",
        query_params.next.as_deref(),
        &server_config.domain,
    );

    if let Err(e) = form.validate() {
        log::error!("Failed to validate form: {e}");
        FlashMessage::error("Ошибка валидации формы").send();
        return redirect(&failure_redirect_url);
    }

    let user_roles = match repo.login(&form.email, &form.password, form.hub_id) {
        Ok(Some(user_roles)) => user_roles,
        Ok(None) => {
            FlashMessage::error("Неверный логин или пароль.").send();
            return redirect(&failure_redirect_url);
        }
        Err(e) => {
            error!("Login error: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut claims = AuthenticatedUser::from(user_roles);

    let jwt = match claims.to_jwt(&common_config.secret) {
        Ok(jwt) => jwt,
        Err(e) => {
            error!("Failed to encode claims: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    match Identity::login(&request.extensions(), jwt) {
        Ok(_) => redirect(&success_redirect_url),
        Err(e) => {
            error!("Failed to login: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/register")]
pub async fn register(
    repo: web::Data<DieselRepository>,
    server_config: web::Data<ServerConfig>,
    web::Form(form): web::Form<RegisterForm>,
    query_params: web::Query<AuthQueryParams>,
) -> impl Responder {
    let (_, failure_redirect_url) = get_success_and_failure_redirects(
        "/auth/signup",
        query_params.next.as_deref(),
        &server_config.domain,
    );

    if let Err(e) = form.validate() {
        log::error!("Failed to validate form: {e}");
        FlashMessage::error("Ошибка валидации формы").send();
        return redirect(&failure_redirect_url);
    }

    let new_user = form.into();
    match repo.create_user(&new_user) {
        Ok(_) => {
            FlashMessage::success("Пользователь может войти.".to_string()).send();
        }
        Err(err) => {
            log::error!("Failed to create user: {err}");
            FlashMessage::error("Ошибка при создании пользователя").send();
        }
    }
    redirect(&failure_redirect_url)
}

#[get("/signin")]
pub async fn signin(
    user: Option<Identity>,
    flash_messages: IncomingFlashMessages,
    repo: web::Data<DieselRepository>,
    query_params: web::Query<AuthQueryParams>,
    tera: web::Data<Tera>,
) -> impl Responder {
    if user.is_some() {
        return redirect("/");
    }

    let hubs = match repo.list_hubs() {
        Ok(hubs) => hubs,
        Err(e) => {
            error!("Failed to get hubs: {e}");
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
    context.insert("next", &query_params.next);

    render_template(&tera, "auth/login.html", &context)
}

#[get("/signup")]
pub async fn signup(
    user: Option<Identity>,
    flash_messages: IncomingFlashMessages,
    repo: web::Data<DieselRepository>,
    query_params: web::Query<AuthQueryParams>,
    tera: web::Data<Tera>,
) -> impl Responder {
    if user.is_some() {
        return redirect("/");
    }

    let hubs = match repo.list_hubs() {
        Ok(hubs) => hubs,
        Err(e) => {
            error!("Failed to get hubs: {e}");
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
    context.insert("next", &query_params.next);

    render_template(&tera, "auth/register.html", &context)
}
