//! Authentication and session management endpoints.

use std::collections::HashMap;
use std::sync::Arc;

use actix_identity::Identity;
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use actix_web::{Responder, get, post, web};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use log::error;
use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::domain::emailer::email::{NewEmail, NewEmailRecipient};
use pushkind_common::models::config::CommonServerConfig;
use pushkind_common::models::emailer::zmq::ZMQSendEmailMessage;
use pushkind_common::routes::render_template;
use pushkind_common::routes::{alert_level_to_str, redirect};
use pushkind_common::zmq::ZmqSender;
use serde::Deserialize;
use tera::{Context, Tera};
use validator::Validate;

use crate::forms::auth::{LoginForm, RecoverForm, RegisterForm};
use crate::models::config::ServerConfig;
use crate::repository::{DieselRepository, HubReader, UserReader, UserWriter};
use crate::routes::get_success_and_failure_redirects;

#[derive(Deserialize)]
struct AuthQueryParams {
    next: Option<String>,
    token: Option<String>,
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

    let claims = AuthenticatedUser::from(user_roles);

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
    request: HttpRequest,
    user: Option<Identity>,
    flash_messages: IncomingFlashMessages,
    repo: web::Data<DieselRepository>,
    query_params: web::Query<AuthQueryParams>,
    tera: web::Data<Tera>,
    common_config: web::Data<CommonServerConfig>,
) -> impl Responder {
    if user.is_some() {
        return redirect("/");
    }

    if let Some(token) = query_params.token.as_deref() {
        let user = match AuthenticatedUser::from_jwt(token, &common_config.secret) {
            Ok(user) => user,
            Err(e) => {
                error!("Failed to get user by token: {e}");
                FlashMessage::error("Ошибка при аутентификации пользователя").send();
                return redirect("/signin");
            }
        };

        match repo.get_user_by_email(&user.email, user.hub_id) {
            Ok(Some(_)) => (),
            Ok(None) => {
                error!("User not found");
                FlashMessage::error("Пользователь не найден").send();
                return redirect("/signin");
            }
            Err(e) => {
                error!("Failed to get user by email: {e}");
                return HttpResponse::InternalServerError().finish();
            }
        }

        match Identity::login(&request.extensions(), token.to_string()) {
            Ok(_) => return redirect("/"),
            Err(e) => {
                error!("Failed to login: {e}");
                return redirect("/signin");
            }
        }
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

    let mut user: AuthenticatedUser = match repo.get_user_by_email(&form.email, form.hub_id) {
        Ok(Some(user)) => user.into(),
        Ok(None) => {
            FlashMessage::error("Пользователь не найден").send();
            return redirect("/auth/signin");
        }
        Err(e) => {
            error!("Failed to get user by email: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };
    user.set_expiration(1);

    let jwt = match user.to_jwt(&common_config.secret) {
        Ok(jwt) => jwt,
        Err(e) => {
            error!("Failed to encode claims: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Build full URL from current request: schema://host{auth_service_url}?token={jwt}
    let recovery_url = {
        let conn_info = request.connection_info();
        let scheme = conn_info.scheme();
        let host = conn_info.host();
        format!("{}://{}/auth/signin?token={}", scheme, host, jwt)
    };

    let new_email = NewEmail {
        message: "Для входа в систему перейдите по ссылке: {recovery_url}\nЕсли вы не запрашивали восстановление, проигнорируйте это письмо.".to_string(),
        subject: Some("Восстановление пароля".to_string()),
        attachment: None,
        attachment_name: None,
        attachment_mime: None,
        hub_id: form.hub_id,
        recipients: vec![NewEmailRecipient {
            address: form.email,
            name: "".to_string(),
            fields: HashMap::from([("recovery_url".to_string(), recovery_url)]),
        }],
    };

    let zmq_message = ZMQSendEmailMessage::NewEmail(Box::new((user, new_email)));

    match zmq_sender.send_json(&zmq_message).await {
        Ok(_) => HttpResponse::Ok().body("Ссылка для входа выслана на электронную почту."),
        Err(err) => {
            HttpResponse::Ok().body(format!("Ошибка при добавлении сообщения в очередь: {err}"))
        }
    }
}
