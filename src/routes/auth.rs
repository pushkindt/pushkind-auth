//! Authentication and session management endpoints.

use std::sync::Arc;

use actix_identity::Identity;
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use actix_web::{Responder, get, post, web};
use pushkind_common::models::config::CommonServerConfig;
use pushkind_common::routes::redirect;
use pushkind_common::services::errors::ServiceError;
use pushkind_common::zmq::ZmqSender;
use serde::Deserialize;

use crate::dto::api::{ApiMutationErrorDto, ApiMutationSuccessDto};
use crate::forms::FormError;
use crate::forms::auth::{
    LoginForm, LoginPayload, RecoverForm, RecoverPayload, RegisterForm, RegisterPayload,
};
use crate::frontend::open_frontend_html;
use crate::models::config::ServerConfig;
use crate::repository::DieselRepository;
use crate::routes::{form_error_response, get_success_and_failure_redirects, wants_json};
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
            return redirect("/auth/signin");
        }
    };
    if let Err(e) = Identity::login(&request.extensions(), jwt.token) {
        log::error!("Failed to login: {e}");
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
    let wants_json = wants_json(&request);
    let (success_redirect_url, failure_redirect_url) = get_success_and_failure_redirects(
        "/auth/signin",
        query_params.next.as_deref(),
        &server_config.domain,
    );

    let payload = match LoginPayload::try_from(form) {
        Ok(payload) => payload,
        Err(error) => {
            if wants_json {
                return HttpResponse::BadRequest().json(form_error_response(&error));
            }
            log::error!("Invalid login data: {error}");
            return redirect(&failure_redirect_url);
        }
    };

    let jwt =
        match auth_service::login_and_issue_token(payload, &common_config.secret, repo.get_ref()) {
            Ok(jwt) => jwt,
            Err(ServiceError::Unauthorized) => {
                if wants_json {
                    return HttpResponse::Unauthorized().json(ApiMutationErrorDto {
                        message: "Неверный логин или пароль.".to_string(),
                        field_errors: Vec::new(),
                    });
                }
                return redirect(&failure_redirect_url);
            }
            Err(ServiceError::Form(e)) => {
                log::error!("Invalid login data: {e}");
                if wants_json {
                    return HttpResponse::BadRequest().json(ApiMutationErrorDto {
                        message: "Ошибка валидации формы.".to_string(),
                        field_errors: Vec::new(),
                    });
                }
                return redirect(&failure_redirect_url);
            }
            Err(e) => {
                log::error!("Login error: {e}");
                if wants_json {
                    return HttpResponse::InternalServerError().json(ApiMutationErrorDto {
                        message: "Ошибка при аутентификации пользователя.".to_string(),
                        field_errors: Vec::new(),
                    });
                }
                return HttpResponse::InternalServerError().finish();
            }
        };

    match Identity::login(&request.extensions(), jwt.token) {
        Ok(_) => {
            if wants_json {
                HttpResponse::Ok().json(ApiMutationSuccessDto {
                    message: "Авторизация выполнена.".to_string(),
                    redirect_to: Some(success_redirect_url),
                })
            } else {
                redirect(&success_redirect_url)
            }
        }
        Err(e) => {
            log::error!("Failed to login: {e}");
            if wants_json {
                HttpResponse::InternalServerError().json(ApiMutationErrorDto {
                    message: "Ошибка при аутентификации пользователя.".to_string(),
                    field_errors: Vec::new(),
                })
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

/// Registers a new user account via `POST /register`.
#[post("/register")]
pub async fn register(
    request: HttpRequest,
    web::Form(form): web::Form<RegisterForm>,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let wants_json = wants_json(&request);
    let payload = match RegisterPayload::try_from(form) {
        Ok(payload) => payload,
        Err(error) => {
            if wants_json {
                return HttpResponse::BadRequest().json(form_error_response(&error));
            }
            log::error!("Failed to convert form: {error}");
            return redirect("/auth/signup");
        }
    };

    match auth_service::register_user(payload, repo.get_ref()) {
        Ok(_) => {
            if wants_json {
                return HttpResponse::Created().json(ApiMutationSuccessDto {
                    message: "Пользователь может войти.".to_string(),
                    redirect_to: Some("/auth/signin".to_string()),
                });
            }
            redirect("/auth/signin")
        }
        Err(ServiceError::Conflict) => {
            if wants_json {
                return HttpResponse::Conflict().json(ApiMutationErrorDto {
                    message: "Пользователь с таким email уже существует.".to_string(),
                    field_errors: Vec::new(),
                });
            }
            redirect("/auth/signup")
        }
        Err(ServiceError::Form(e)) => {
            log::error!("Failed to convert form: {e}");
            if wants_json {
                return HttpResponse::BadRequest().json(ApiMutationErrorDto {
                    message: "Ошибка валидации формы.".to_string(),
                    field_errors: Vec::new(),
                });
            }
            redirect("/auth/signup")
        }
        Err(err) => {
            log::error!("Failed to create user: {err}");
            if wants_json {
                return HttpResponse::InternalServerError().json(ApiMutationErrorDto {
                    message: "Ошибка при создании пользователя.".to_string(),
                    field_errors: Vec::new(),
                });
            }
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Renders the sign-in page via `GET /signin`.
#[get("/signin")]
pub async fn signin_page(request: HttpRequest, user: Option<Identity>) -> impl Responder {
    if user.is_some() {
        return redirect("/");
    }

    match open_frontend_html("assets/dist/auth/signin.html").await {
        Ok(file) => file.into_response(&request),
        Err(err) => {
            log::error!("Failed to open sign-in frontend document: {err}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Renders the registration page via `GET /signup`.
#[get("/signup")]
pub async fn signup_page(request: HttpRequest, user: Option<Identity>) -> impl Responder {
    if user.is_some() {
        return redirect("/");
    }

    match open_frontend_html("assets/dist/auth/signup.html").await {
        Ok(file) => file.into_response(&request),
        Err(err) => {
            log::error!("Failed to open sign-up frontend document: {err}");
            HttpResponse::InternalServerError().finish()
        }
    }
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
    let wants_json = wants_json(&request);
    let payload = match RecoverPayload::try_from(form) {
        Ok(payload) => payload,
        Err(error) => {
            if wants_json {
                return HttpResponse::BadRequest().json(form_error_response(&error));
            }

            match error {
                FormError::Validation(validation_error) => {
                    log::error!("Invalid recovery data: {validation_error}");
                }
                _ => log::error!("Invalid recovery data: {error}"),
            }
            return redirect("/auth/signin");
        }
    };

    // Build base URL from current request: schema://host
    let base_url = {
        let conn_info = request.connection_info();
        format!("{}://{}", conn_info.scheme(), conn_info.host())
    };

    match auth_service::send_recovery_email(
        payload,
        &base_url,
        zmq_sender.get_ref().as_ref(),
        repo.get_ref(),
        &common_config.secret,
    )
    .await
    {
        Ok(_) => {
            if wants_json {
                HttpResponse::Ok().json(ApiMutationSuccessDto {
                    message: "Ссылка для входа выслана на электронную почту.".to_string(),
                    redirect_to: None,
                })
            } else {
                HttpResponse::Ok().body("Ссылка для входа выслана на электронную почту.")
            }
        }
        Err(ServiceError::NotFound) => {
            if wants_json {
                return HttpResponse::NotFound().json(ApiMutationErrorDto {
                    message: "Пользователь не найден.".to_string(),
                    field_errors: Vec::new(),
                });
            }
            redirect("/auth/signin")
        }
        Err(ServiceError::Form(e)) => {
            log::error!("Invalid recovery data: {e}");
            if wants_json {
                return HttpResponse::BadRequest().json(ApiMutationErrorDto {
                    message: "Ошибка валидации формы.".to_string(),
                    field_errors: Vec::new(),
                });
            }
            redirect("/auth/signin")
        }
        Err(err) => {
            log::error!("Failed to send recovery email: {err}");
            if wants_json {
                return HttpResponse::InternalServerError().json(ApiMutationErrorDto {
                    message: "Ошибка при отправке ссылки для входа.".to_string(),
                    field_errors: Vec::new(),
                });
            }
            HttpResponse::InternalServerError().finish()
        }
    }
}
