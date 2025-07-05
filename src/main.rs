use std::env;

use actix_identity::IdentityMiddleware;
use actix_session::config::CookieContentSecurity;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_web::{App, HttpServer, middleware, web};
use actix_web_flash_messages::{FlashMessagesFramework, storage::CookieMessageStore};
use dotenvy::dotenv;
use log::error;

use pushkind_auth::db::establish_connection_pool;
use pushkind_auth::middleware::RedirectUnauthorized;
use pushkind_auth::models::config::ServerConfig;
use pushkind_auth::routes::admin::{
    add_hub, add_menu, add_role, delete_hub, delete_menu, delete_role, delete_user, update_user,
    user_modal,
};
use pushkind_auth::routes::auth::{login, logout, register, signin, signup};
use pushkind_auth::routes::main::{api_v1_id, api_v1_users, index, save_user};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL").unwrap_or("app.db".to_string());
    let port = env::var("PORT").unwrap_or("8080".to_string());
    let port = port.parse::<u16>().unwrap_or(8080);
    let address = env::var("ADDRESS").unwrap_or("127.0.0.1".to_string());

    let secret = env::var("SECRET_KEY");
    let secret = match secret {
        Ok(secret) => secret,
        Err(_) => {
            error!("SECRET_KEY environment variable not set");
            std::process::exit(1);
        }
    };
    let secret_key = Key::from(secret.as_bytes());
    let server_config = ServerConfig { secret };

    let domain = env::var("DOMAIN").unwrap_or("localhost".to_string());

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
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false) // set to true in prod
                    .cookie_domain(Some(format!(".{domain}")))
                    .build(),
            )
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
            .service(
                web::scope("/admin")
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
            .service(
                web::scope("")
                    .wrap(RedirectUnauthorized)
                    .service(index)
                    .service(save_user)
                    .service(api_v1_id)
                    .service(api_v1_users),
            )
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(server_config.clone()))
    })
    .bind((address, port))?
    .run()
    .await
}
