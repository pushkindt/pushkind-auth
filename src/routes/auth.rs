//! Authentication and session management endpoints.

use std::sync::Arc;

use actix_identity::Identity;
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use actix_web::{Responder, get, post, web};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use pushkind_common::models::config::CommonServerConfig;
use pushkind_common::routes::render_template;
use pushkind_common::routes::{alert_level_to_str, redirect};
use pushkind_common::services::errors::ServiceError;
use pushkind_common::zmq::ZmqSender;
use serde::Deserialize;
use tera::{Context, Tera};
use validator::Validate;

use crate::forms::auth::{LoginForm, RecoverForm, RegisterForm};
use crate::models::config::ServerConfig;
use crate::repository::DieselRepository;
use crate::routes::get_success_and_failure_redirects;
use crate::services::auth as auth_service;

#[derive(Deserialize)]
struct AuthQueryParams {
    next: Option<String>,
}

#[derive(Deserialize)]
struct LoginTokenParams {
    token: String,
}

#[get("/login")]
pub async fn login_token(
    request: HttpRequest,
    repo: web::Data<DieselRepository>,
    common_config: web::Data<CommonServerConfig>,
    query_params: web::Query<LoginTokenParams>,
) -> impl Responder {
    let jwt = match auth_service::reissue_session_from_token(
        &query_params.token,
        &common_config.secret,
        7,
        repo.get_ref(),
    ) {
        Ok(jwt) => jwt,
        Err(e) => {
            log::error!("Failed to reissue session: {e}");
            FlashMessage::error("Ошибка при аутентификации пользователя").send();
            return redirect("/signin");
        }
    };
    if let Err(e) = Identity::login(&request.extensions(), jwt) {
        log::error!("Failed to login: {e}");
        FlashMessage::error("Ошибка при аутентификации пользователя").send();
        return redirect("/signin");
    }
    redirect("/")
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

    let jwt = match auth_service::login_and_issue_token(
        &form.email,
        &form.password,
        form.hub_id,
        repo.get_ref(),
        &common_config.secret,
    ) {
        Ok(jwt) => jwt,
        Err(ServiceError::Unauthorized) => {
            FlashMessage::error("Неверный логин или пароль.").send();
            return redirect(&failure_redirect_url);
        }
        Err(e) => {
            log::error!("Login error: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    match Identity::login(&request.extensions(), jwt) {
        Ok(_) => redirect(&success_redirect_url),
        Err(e) => {
            log::error!("Failed to login: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/register")]
pub async fn register(
    repo: web::Data<DieselRepository>,
    web::Form(form): web::Form<RegisterForm>,
) -> impl Responder {
    if let Err(e) = form.validate() {
        log::error!("Failed to validate form: {e}");
        FlashMessage::error("Ошибка валидации формы").send();
        return redirect("/auth/signup");
    }

    let new_user = form.into();
    match auth_service::register_user(&new_user, repo.get_ref()) {
        Ok(_) => {
            FlashMessage::success("Пользователь может войти.".to_string()).send();
            redirect("/auth/signin")
        }
        Err(ServiceError::Conflict) => {
            FlashMessage::error("Пользователь с таким email уже существует.").send();
            redirect("/auth/signup")
        }
        Err(err) => {
            log::error!("Failed to create user: {err}");
            FlashMessage::error("Ошибка при создании пользователя").send();
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/signin")]
pub async fn signin_page(
    user: Option<Identity>,
    flash_messages: IncomingFlashMessages,
    repo: web::Data<DieselRepository>,
    query_params: web::Query<AuthQueryParams>,
    tera: web::Data<Tera>,
) -> impl Responder {
    if user.is_some() {
        return redirect("/");
    }

    let hubs = match auth_service::list_hubs(repo.get_ref()) {
        Ok(hubs) => hubs,
        Err(e) => {
            log::error!("Failed to get hubs: {e}");
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
pub async fn signup_page(
    user: Option<Identity>,
    flash_messages: IncomingFlashMessages,
    repo: web::Data<DieselRepository>,
    query_params: web::Query<AuthQueryParams>,
    tera: web::Data<Tera>,
) -> impl Responder {
    if user.is_some() {
        return redirect("/");
    }

    let hubs = match auth_service::list_hubs(repo.get_ref()) {
        Ok(hubs) => hubs,
        Err(e) => {
            log::error!("Failed to get hubs: {e}");
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

#[post("/recover")]
pub async fn recover_password(
    request: HttpRequest,
    zmq_sender: web::Data<Arc<ZmqSender>>,
    repo: web::Data<DieselRepository>,
    common_config: web::Data<CommonServerConfig>,
    web::Form(form): web::Form<RecoverForm>,
) -> impl Responder {
    if let Err(e) = form.validate() {
        log::error!("Failed to validate form: {e}");
        FlashMessage::error("Ошибка валидации формы").send();
        return redirect("/auth/signin");
    }

    // Build base URL from current request: schema://host
    let base_url = {
        let conn_info = request.connection_info();
        format!("{}://{}", conn_info.scheme(), conn_info.host())
    };

    match auth_service::send_recovery_email(
        zmq_sender.get_ref().as_ref(),
        repo.get_ref(),
        &common_config.secret,
        form.hub_id,
        &form.email,
        &base_url,
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().body("Ссылка для входа выслана на электронную почту."),
        Err(ServiceError::NotFound) => {
            FlashMessage::error("Пользователь не найден").send();
            redirect("/auth/signin")
        }
        Err(err) => {
            log::error!("Failed to send recovery email: {err}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
