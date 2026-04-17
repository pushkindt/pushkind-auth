#![allow(dead_code)]

//! Helpers for integration tests.

use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::sync::Once;
use std::time::Duration;

use actix_web::rt::time::sleep;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use pushkind_auth::{
    domain::{
        types::{UserEmail, UserName, UserPassword},
        user::{NewUser, UpdateUser},
    },
    repository::{DieselRepository, HubReader, RoleReader, UserWriter},
};
use pushkind_common::db::{DbPool, establish_connection_pool};
use reqwest::{Client, StatusCode};
use tempfile::NamedTempFile;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!(); // assumes migrations/ exists
pub const ADMIN_EMAIL: &str = "admin@hub";
pub const ADMIN_PASSWORD: &str = "password";
pub const USER_EMAIL: &str = "user@hub";
pub const USER_PASSWORD: &str = "password";

static FRONTEND_ASSETS: Once = Once::new();

pub struct SeededUsers {
    pub hub_id: i32,
    pub admin_user_id: i32,
    pub user_id: i32,
}

/// Temporary database used in integration tests.
pub struct TestDb {
    _tempfile: NamedTempFile,
    pool: DbPool,
}

pub struct TestApp {
    test_db: TestDb,
    address: String,
}

impl TestDb {
    pub fn new() -> Self {
        let tempfile = NamedTempFile::new().expect("Failed to create temp file");
        let pool = establish_connection_pool(tempfile.path().to_str().unwrap())
            .expect("Failed to establish SQLite connection.");
        let mut conn = pool
            .get()
            .expect("Failed to get SQLite connection from pool.");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Migrations failed");
        TestDb {
            _tempfile: tempfile,
            pool,
        }
    }

    pub fn pool(&self) -> DbPool {
        self.pool.clone()
    }

    pub fn get_db_path(&self) -> String {
        self._tempfile.path().to_str().unwrap().to_string()
    }
}

impl TestApp {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn db_pool(&self) -> DbPool {
        self.test_db.pool()
    }
}

async fn wait_until_server_is_ready(address: &str) {
    let client = Client::builder()
        .timeout(Duration::from_millis(100))
        .build()
        .expect("Failed to create the test HTTP client.");
    let health_url = format!("{address}/health");

    for _ in 0..20 {
        match client.get(&health_url).send().await {
            Ok(response) if response.status() == StatusCode::OK => return,
            Ok(_) | Err(_) => sleep(Duration::from_millis(25)).await,
        }
    }

    panic!("Test server did not become ready at {health_url}");
}

fn ensure_test_frontend_assets() {
    FRONTEND_ASSETS.call_once(|| {
        let fixtures = [
            (
                "assets/dist/auth/signin.html",
                "<!doctype html><html><body>auth-signin.tsx</body></html>",
            ),
            (
                "assets/dist/auth/signup.html",
                "<!doctype html><html><body>auth-signup.tsx</body></html>",
            ),
            (
                "assets/dist/app/index-admin.html",
                "<!doctype html><html><body>main-admin.tsx</body></html>",
            ),
            (
                "assets/dist/app/index-basic.html",
                "<!doctype html><html><body>main-basic.tsx</body></html>",
            ),
        ];

        for (path, contents) in fixtures {
            let path = Path::new(path);
            if path.exists() {
                continue;
            }

            let parent = path
                .parent()
                .expect("frontend fixture path should include a parent directory");
            fs::create_dir_all(parent).expect("failed to create frontend fixture directory");
            fs::write(path, contents).expect("failed to write frontend fixture file");
        }
    });
}

/// Launch the application in the background and return a handle for driving it.
pub async fn spawn_app() -> TestApp {
    ensure_test_frontend_assets();

    let test_db = TestDb::new();
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind a random local port.");
    let port = listener
        .local_addr()
        .expect("Failed to read the local socket address.")
        .port();

    let test_config = pushkind_auth::models::config::AppConfig {
        domain: "localhost".to_string(),
        database_url: test_db.get_db_path(),
        zmq_emailer_pub: "tcp://127.0.0.1:35559".to_string(),
        secret: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
    };

    let server = pushkind_auth::build_server(listener, test_config)
        .expect("Failed to build the test application server.");
    actix_web::rt::spawn(server);
    let address = format!("http://localhost:{port}");

    wait_until_server_is_ready(&address).await;

    TestApp { test_db, address }
}

pub fn build_reqwest_client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .expect("Can't create a request client")
}

pub fn setup_hub_with_admin(db_pool: DbPool) {
    let _ = setup_hub_with_users(db_pool);
}

pub fn setup_hub_with_users(db_pool: DbPool) -> SeededUsers {
    let repo = DieselRepository::new(db_pool);

    // Create hub
    let hub = repo.get_hub_by_name("default").unwrap().unwrap();

    let admin_name = UserName::new("Admin").unwrap();
    let admin_password = UserPassword::new(ADMIN_PASSWORD).unwrap();
    let admin = NewUser::new(
        UserEmail::new(ADMIN_EMAIL).unwrap(),
        Some(admin_name.clone()),
        hub.id,
        admin_password,
    );
    let admin = repo
        .create_user(&admin)
        .expect("Can't create admin test user");

    let admin_role = repo.get_role_by_name("admin").unwrap().unwrap();
    let updates = UpdateUser::new(admin_name.clone(), None, Some(vec![admin_role.id]));
    let _ = repo
        .update_user(admin.id, hub.id, &updates)
        .expect("Can't assign the admin role to the test user");

    let user_name = UserName::new("User").unwrap();
    let user_password = UserPassword::new(USER_PASSWORD).unwrap();
    let user = NewUser::new(
        UserEmail::new(USER_EMAIL).unwrap(),
        Some(user_name),
        hub.id,
        user_password,
    );
    let user = repo
        .create_user(&user)
        .expect("Can't create regular test user");

    SeededUsers {
        hub_id: hub.id.get(),
        admin_user_id: admin.id.get(),
        user_id: user.id.get(),
    }
}
