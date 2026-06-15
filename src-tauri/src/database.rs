use std::{path::Path, time::Duration};

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};

pub(crate) const DATABASE_FILE_NAME: &str = "database.sqlite";

pub struct SqliteRepository {
    pub(crate) pool: SqlitePool,
}

impl SqliteRepository {
    pub async fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("Failed to create database directory: {error}"))?;
        }

        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(5));
        let pool = SqlitePoolOptions::new()
            .max_connections(3)
            .connect_with(options)
            .await
            .map_err(|error| format!("Failed to open application database: {error}"))?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|error| format!("Failed to migrate application database: {error}"))?;

        Ok(Self { pool })
    }

    #[cfg(test)]
    pub(crate) async fn close(&self) {
        self.pool.close().await;
    }
}
