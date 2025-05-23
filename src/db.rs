use std::time::Duration;

use anyhow::Context;
use diesel::connection::SimpleConnection;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;
use log::error;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

#[derive(Debug)]
pub struct ConnectionOptions {
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
    pub busy_timeout: Option<Duration>,
}

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for ConnectionOptions {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        (|| {
            if self.enable_wal {
                conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
            }
            if self.enable_foreign_keys {
                conn.batch_execute("PRAGMA foreign_keys = ON;")?;
            }
            if let Some(d) = self.busy_timeout {
                conn.batch_execute(&format!("PRAGMA busy_timeout = {};", d.as_millis()))?;
            }
            Ok(())
        })()
        .map_err(diesel::r2d2::Error::QueryError)
    }
}

pub fn establish_connection_pool(database_url: &str) -> anyhow::Result<DbPool> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder()
        .connection_customizer(Box::new(ConnectionOptions {
            enable_wal: true,
            enable_foreign_keys: true,
            busy_timeout: Some(Duration::from_secs(30)),
        }))
        .build(manager)
        .context("Failed to build Diesel SQLite connection pool")
}

pub fn get_connection(pool: &DbPool) -> anyhow::Result<DbConnection> {
    match pool.get() {
        Ok(conn) => Ok(conn),
        Err(e) => {
            error!("Failed to get connection from pool: {}", e);
            Err(anyhow::anyhow!("Failed to get connection from pool: {}", e))
        }
    }
}
