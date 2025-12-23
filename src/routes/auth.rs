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

/// Reissues a session from a short-lived token via `GET /login`.
#[get("/login")]
pub async fn login_token(
    query_params: web::Query<LoginTokenParams>,
    request: HttpRequest,
    repo: web::Data<DieselRepository>,
    common_config: web::Data<CommonServerConfig>,
) -> impl Responder {
    let jwt = match auth_service::reissue_session_from_token(
        &query_params.token,
        7,
        &common_config.secret,
        repo.get_ref(),
    ) {
        Ok(jwt) => jwt,
        Err(e) => {
            log::error!("Failed to reissue session: {e}");
            FlashMessage::error("Ошибка при аутентификации пользователя").send();
            return redirect("/auth/signin");
        }
    };
    if let Err(e) = Identity::login(&request.extensions(), jwt.token) {
        log::error!("Failed to login: {e}");
        FlashMessage::error("Ошибка при аутентификации пользователя").send();
        return redirect("/auth/signin");
    }
    redirect("/")
}

/// Authenticates a user with credentials via `POST /login`.
#[post("/login")]
pub async fn login(
    web::Form(form): web::Form<LoginForm>,
    query_params: web::Query<AuthQueryParams>,
    request: HttpRequest,
    repo: web::Data<DieselRepository>,
    server_config: web::Data<ServerConfig>,
    common_config: web::Data<CommonServerConfig>,
) -> impl Responder {
    let (success_redirect_url, failure_redirect_url) = get_success_and_failure_redirects(
        "/auth/signin",
        query_params.next.as_deref(),
        &server_config.domain,
    );

    let jwt = match auth_service::login_and_issue_token(form, &common_config.secret, repo.get_ref())
    {
        Ok(jwt) => jwt,
        Err(ServiceError::Unauthorized) => {
            FlashMessage::error("Неверный логин или пароль.").send();
            return redirect(&failure_redirect_url);
        }
        Err(ServiceError::Form(e)) => {
            log::error!("Invalid login data: {e}");
            FlashMessage::error("Ошибка валидации формы").send();
            return redirect(&failure_redirect_url);
        }
        Err(e) => {
            log::error!("Login error: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    match Identity::login(&request.extensions(), jwt.token) {
        Ok(_) => redirect(&success_redirect_url),
        Err(e) => {
            log::error!("Failed to login: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Registers a new user account via `POST /register`.
#[post("/register")]
pub async fn register(
    web::Form(form): web::Form<RegisterForm>,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    match auth_service::register_user(form, repo.get_ref()) {
        Ok(_) => {
            FlashMessage::success("Пользователь может войти.".to_string()).send();
            redirect("/auth/signin")
        }
        Err(ServiceError::Conflict) => {
            FlashMessage::error("Пользователь с таким email уже существует.").send();
            redirect("/auth/signup")
        }
        Err(ServiceError::Form(e)) => {
            log::error!("Failed to convert form: {e}");
            FlashMessage::error("Ошибка валидации формы").send();
            redirect("/auth/signup")
        }
        Err(err) => {
            log::error!("Failed to create user: {err}");
            FlashMessage::error("Ошибка при создании пользователя").send();
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Renders the sign-in page via `GET /signin`.
#[get("/signin")]
pub async fn signin_page(
    query_params: web::Query<AuthQueryParams>,
    user: Option<Identity>,
    flash_messages: IncomingFlashMessages,
    repo: web::Data<DieselRepository>,
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

/// Renders the registration page via `GET /signup`.
#[get("/signup")]
pub async fn signup_page(
    query_params: web::Query<AuthQueryParams>,
    user: Option<Identity>,
    flash_messages: IncomingFlashMessages,
    repo: web::Data<DieselRepository>,
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

/// Sends a recovery email and issues a passwordless login link.
#[post("/recover")]
pub async fn recover_password(
    web::Form(form): web::Form<RecoverForm>,
    request: HttpRequest,
    zmq_sender: web::Data<Arc<ZmqSender>>,
    repo: web::Data<DieselRepository>,
    common_config: web::Data<CommonServerConfig>,
) -> impl Responder {
    // Build base URL from current request: schema://host
    let base_url = {
        let conn_info = request.connection_info();
        format!("{}://{}", conn_info.scheme(), conn_info.host())
    };

    match auth_service::send_recovery_email(
        form,
        &base_url,
        zmq_sender.get_ref().as_ref(),
        repo.get_ref(),
        &common_config.secret,
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().body("Ссылка для входа выслана на электронную почту."),
        Err(ServiceError::NotFound) => {
            FlashMessage::error("Пользователь не найден").send();
            redirect("/auth/signin")
        }
        Err(ServiceError::Form(e)) => {
            log::error!("Invalid recovery data: {e}");
            FlashMessage::error("Ошибка валидации формы").send();
            redirect("/auth/signin")
        }
        Err(err) => {
            log::error!("Failed to send recovery email: {err}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
