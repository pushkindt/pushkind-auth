//! Main library crate for the Pushkind authentication service.
//!
//! This crate exposes the domain models, database access layer and HTTP
//! handlers that make up the application. It is used by `main.rs` to build
//! the Actix-Web application and can also be reused for integration tests.

use std::sync::Arc;

use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::cookie::{Key, time::Duration};
use actix_web::{App, HttpServer, web};
use actix_web_flash_messages::{FlashMessagesFramework, storage::CookieMessageStore};

use pushkind_common::db::establish_connection_pool;
use pushkind_common::middleware::RedirectUnauthorized;
use pushkind_common::models::config::CommonServerConfig;
use pushkind_common::routes::logout;
use pushkind_common::zmq::{ZmqSender, ZmqSenderOptions};
use tera::Tera;

use crate::middleware::RequireUserExists;
use crate::models::config::ServerConfig;
use crate::repository::DieselRepository;
use crate::routes::admin::{
    add_hub, add_menu, add_role, delete_hub, delete_menu, delete_role, delete_user, update_user,
    user_modal,
};
use crate::routes::api::{api_v1_id, api_v1_users};
use crate::routes::auth::{
    login, login_token, recover_password, register, signin_page, signup_page,
};
use crate::routes::main::{save_user, show_index};

pub mod domain;
pub mod forms;
pub mod middleware;
pub mod models;
pub mod repository;
pub mod routes;
pub mod schema;
pub mod services;

pub const SERVICE_ACCESS_ROLE: &str = "admin";
const AUTH_SERVICE_URL: &str = "/auth/signin";

pub async fn run(server_config: ServerConfig) -> std::io::Result<()> {
    let common_config = CommonServerConfig {
        auth_service_url: AUTH_SERVICE_URL.to_string(),
        secret: server_config.secret.clone(),
    };

    let zmq_sender = ZmqSender::start(ZmqSenderOptions::pub_default(
        &server_config.zmq_emailer_pub,
    ))
    .map_err(|e| std::io::Error::other(format!("Failed to start ZMQ sender: {e}")))?;

    let zmq_sender = Arc::new(zmq_sender);

    let pool = establish_connection_pool(&server_config.database_url).map_err(|e| {
        std::io::Error::other(format!("Failed to establish database connection: {e}"))
    })?;

    let repo = DieselRepository::new(pool);

    let secret_key = Key::from(server_config.secret.as_bytes());

    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();

    let tera = Tera::new(&server_config.templates_dir)
        .map_err(|e| std::io::Error::other(format!("Template parsing error(s): {e}")))?;

    let bind_address = (server_config.address.clone(), server_config.port);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(message_framework.clone())
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(7)))
                    .cookie_secure(false) // set to true in prod
                    .cookie_domain(Some(format!(".{}", server_config.domain)))
                    .build(),
            )
            .wrap(actix_web::middleware::Compress::default())
            .wrap(actix_web::middleware::Logger::default())
            .service(
                web::scope("/auth")
                    .service(logout)
                    .service(login)
                    .service(login_token)
                    .service(signin_page)
                    .service(signup_page)
                    .service(register)
                    .service(recover_password),
            )
            .service(
                web::scope("/admin")
                    .wrap(RequireUserExists)
                    .wrap(RedirectUnauthorized)
                    .service(add_role)
                    .service(user_modal)
                    .service(delete_user)
                    .service(update_user)
                    .service(add_hub)
                    .service(delete_hub)
                    .service(delete_role)
                    .service(add_menu)
                    .service(delete_menu),
            )
            .service(web::scope("/api").service(api_v1_id).service(api_v1_users))
            .service(
                web::scope("")
                    .wrap(RequireUserExists)
                    .wrap(RedirectUnauthorized)
                    .service(show_index)
                    .service(save_user),
            )
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(repo.clone()))
            .app_data(web::Data::new(server_config.clone()))
            .app_data(web::Data::new(common_config.clone()))
            .app_data(web::Data::new(zmq_sender.clone()))
    })
    .bind(bind_address)?
    .run()
    .await
}
