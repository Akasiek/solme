use async_trait::async_trait;

use super::AppSettingKey;
use crate::database::SqliteRepository;

#[derive(Clone, Debug, Eq, PartialEq, sqlx::FromRow)]
pub struct StoredAppSetting {
    pub key: String,
    pub value: String,
}

#[async_trait]
pub trait AppSettingsRepository: Send + Sync {
    async fn load_app_settings(&self) -> Result<Vec<StoredAppSetting>, String>;
    async fn save_app_setting(&self, key: AppSettingKey, value: &str) -> Result<(), String>;
}

#[async_trait]
impl AppSettingsRepository for SqliteRepository {
    async fn load_app_settings(&self) -> Result<Vec<StoredAppSetting>, String> {
        sqlx::query_as("SELECT key, value FROM app_settings ORDER BY key")
            .fetch_all(&self.pool)
            .await
            .map_err(|error| format!("Failed to read application setting: {error}"))
    }

    async fn save_app_setting(&self, key: AppSettingKey, value: &str) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO app_settings (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(key.as_str())
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(|error| format!("Failed to save application setting: {error}"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use uuid::Uuid;

    use super::{AppSettingKey, AppSettingsRepository, StoredAppSetting};
    use crate::database::{SqliteRepository, DATABASE_FILE_NAME};

    #[test]
    fn persists_application_setting_after_reopening_database() {
        tauri::async_runtime::block_on(async {
            let directory =
                std::env::temp_dir().join(format!("solme-app-settings-{}", Uuid::new_v4()));
            let database_path = directory.join(DATABASE_FILE_NAME);
            let repository = SqliteRepository::open(&database_path).await.unwrap();

            repository
                .save_app_setting(AppSettingKey::AwayLyricsEnabled, "false")
                .await
                .unwrap();
            repository.close().await;

            let repository = SqliteRepository::open(&database_path).await.unwrap();
            assert_eq!(
                repository.load_app_settings().await.unwrap(),
                vec![StoredAppSetting {
                    key: "lyrics.away_enabled".to_string(),
                    value: "false".to_string(),
                }]
            );
            repository.close().await;

            for path in [
                database_path.clone(),
                database_path.with_extension("sqlite-shm"),
                database_path.with_extension("sqlite-wal"),
            ] {
                if path.exists() {
                    fs::remove_file(path).unwrap();
                }
            }
            fs::remove_dir(directory).unwrap();
        });
    }
}
