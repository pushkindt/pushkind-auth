//! Helpers for integration tests.

use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use pushkind_common::db::{DbPool, establish_connection_pool};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!(); // assumes migrations/ exists

/// Temporary database used in integration tests.
pub struct TestDb {
    filename: String,
    pool: Option<DbPool>,
}

impl TestDb {
    pub fn new(filename: &str) -> Self {
        TestDb::remove_old_files(filename); // Clean up old DB

        let pool =
            establish_connection_pool(filename).expect("Failed to establish SQLite connection.");
        let mut conn = pool
            .get()
            .expect("Failed to get SQLite connection from pool.");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Migrations failed");
        TestDb {
            filename: filename.to_string(),
            pool: Some(pool),
        }
    }
    pub fn pool(&self) -> DbPool {
        self.pool
            .as_ref()
            .expect("TestDb pool should be available")
            .clone()
    }
    fn remove_old_files(filename: &str) {
        std::fs::remove_file(filename).ok();
        std::fs::remove_file(format!("{}-shm", filename)).ok();
        std::fs::remove_file(format!("{}-wal", filename)).ok();
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        if let Some(pool) = self.pool.take() {
            drop(pool);
        }
        TestDb::remove_old_files(&self.filename); // Clean up old DB
    }
}
