use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use pushkind_auth::db::{DbPool, establish_connection_pool};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!(); // assumes migrations/ exists

pub fn setup_test_pool() -> DbPool {
    std::fs::remove_file("test.db").ok(); // Clean up old DB

    let pool =
        establish_connection_pool("test.db").expect("Failed to establish SQLite connection.");
    let mut conn = pool
        .get()
        .expect("Failed to get SQLite connection from pool.");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Migrations failed");
    pool
}
