use std::env;

use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_web::{App, HttpServer, middleware, web};
use actix_web_flash_messages::{FlashMessagesFramework, storage::CookieMessageStore};
use dotenvy::dotenv;
use log::error;

use pushkind_auth::db::establish_connection_pool;
use pushkind_auth::middleware::RedirectUnauthorized;
use pushkind_auth::routes::auth::{login, logout, register, signin, signup};
use pushkind_auth::routes::main::index;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL").unwrap_or("app.db".to_string());
    let port = env::var("PORT").unwrap_or("8080".to_string());
    let port = port.parse::<u16>().unwrap_or(8080);
    let address = env::var("ADDRESS").unwrap_or("127.0.0.1".to_string());
    let secret = env::var("SECRET_KEY");

    let secret_key = match &secret {
        Ok(key) => Key::from(key.as_bytes()),
        Err(_) => Key::generate(),
    };

    let pool = match establish_connection_pool(&database_url) {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to establish database connection: {}", e);
            std::process::exit(1);
        }
    };

    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();

    HttpServer::new(move || {
        App::new()
            .wrap(message_framework.clone())
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/auth")
                    .service(logout)
                    .service(login)
                    .service(signin)
                    .service(signup)
                    .service(register),
            )
            .service(web::scope("").wrap(RedirectUnauthorized).service(index))
            .app_data(web::Data::new(pool.clone()))
    })
    .bind((address, port))?
    .run()
    .await
}
