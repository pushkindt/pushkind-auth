//! Authentication and session management endpoints.

use actix_identity::Identity;
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use actix_web::{Responder, get, post, web};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use log::error;
use serde::Deserialize;
use tera::Context;
use validator::Validate;

use crate::db::DbPool;
use crate::forms::auth::{LoginForm, RegisterForm};
use crate::models::auth::AuthenticatedUser;
use crate::models::config::ServerConfig;
use crate::repository::hub::DieselHubRepository;
use crate::repository::user::DieselUserRepository;
use crate::repository::{HubReader, UserReader, UserWriter};
use crate::routes::{
    alert_level_to_str, get_success_and_failure_redirects, redirect, render_template,
};

#[derive(Deserialize)]
struct AuthQueryParams {
    next: Option<String>,
}

#[post("/login")]
pub async fn login(
    request: HttpRequest,
    pool: web::Data<DbPool>,
    server_config: web::Data<ServerConfig>,
    web::Form(form): web::Form<LoginForm>,
    query_params: web::Query<AuthQueryParams>,
) -> impl Responder {
    let repo = DieselUserRepository::new(&pool);

    let (success_redirect_url, failure_redirect_url) = get_success_and_failure_redirects(
        "/auth/signin",
        query_params.next.as_deref(),
        &server_config.domain,
    );

    if let Err(e) = form.validate() {
        FlashMessage::error(format!("Ошибка валидации формы: {e}")).send();
        return redirect(&failure_redirect_url);
    }

    let user = match repo.login(&form.email, &form.password, form.hub_id) {
        Ok(Some(user)) => user,
        Ok(None) => {
            FlashMessage::error("Неверный логин или пароль.").send();
            return redirect(&failure_redirect_url);
        }
        Err(e) => {
            error!("Login error: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let roles = match repo.get_roles(user.id) {
        Ok(roles) => roles,
        Err(e) => {
            error!("Failed to get user roles: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut claims = AuthenticatedUser::from_user_roles(&user, &roles);

    let jwt = match claims.to_jwt(&server_config.secret) {
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
    pool: web::Data<DbPool>,
    server_config: web::Data<ServerConfig>,
    web::Form(form): web::Form<RegisterForm>,
    query_params: web::Query<AuthQueryParams>,
) -> impl Responder {
    let repo = DieselUserRepository::new(&pool);

    let (_, failure_redirect_url) = get_success_and_failure_redirects(
        "/auth/signup",
        query_params.next.as_deref(),
        &server_config.domain,
    );

    if let Err(e) = form.validate() {
        FlashMessage::error(format!("Ошибка валидации формы: {e}")).send();
        return redirect(&failure_redirect_url);
    }

    let new_user = (&form).into();
    match repo.create(&new_user) {
        Ok(_) => {
            FlashMessage::success("Пользователь может войти.".to_string()).send();
        }
        Err(err) => {
            FlashMessage::error(format!("Ошибка при создании пользователя: {err}")).send();
        }
    }
    redirect(&failure_redirect_url)
}

#[post("/logout")]
pub async fn logout(
    user: Identity,
    server_config: web::Data<ServerConfig>,
    query_params: web::Query<AuthQueryParams>,
) -> impl Responder {
    let (_, failure_redirect_url) = get_success_and_failure_redirects(
        "/auth/signin",
        query_params.next.as_deref(),
        &server_config.domain,
    );

    user.logout();
    redirect(&failure_redirect_url)
}

#[get("/signin")]
pub async fn signin(
    user: Option<Identity>,
    flash_messages: IncomingFlashMessages,
    pool: web::Data<DbPool>,
    query_params: web::Query<AuthQueryParams>,
) -> impl Responder {
    if user.is_some() {
        return redirect("/");
    }

    let repo = DieselHubRepository::new(&pool);

    let hubs = match repo.list() {
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

    render_template("auth/login.html", &context)
}

#[get("/signup")]
pub async fn signup(
    user: Option<Identity>,
    flash_messages: IncomingFlashMessages,
    pool: web::Data<DbPool>,
    query_params: web::Query<AuthQueryParams>,
) -> impl Responder {
    if user.is_some() {
        return redirect("/");
    }

    let repo = DieselHubRepository::new(&pool);

    let hubs = match repo.list() {
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

    render_template("auth/register.html", &context)
}
