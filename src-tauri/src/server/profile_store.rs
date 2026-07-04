use sqlx::{Row, SqlitePool};

use super::models::{ServerType, StoredServerProfile};

const ACTIVE_SERVER_PROFILE_KEY: &str = "active_server_profile_id";

pub struct ServerProfileStore {
    pool: SqlitePool,
}

impl ServerProfileStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn load(&self) -> Result<Option<StoredServerProfile>, String> {
        let Some(active_profile_id) = self.active_profile_id().await? else {
            return self.load_first().await;
        };

        match self.load_by_id(&active_profile_id).await? {
            Some(profile) => Ok(Some(profile)),
            None => self.load_first().await,
        }
    }

    pub async fn load_by_id(&self, id: &str) -> Result<Option<StoredServerProfile>, String> {
        let row = sqlx::query(
            "SELECT id, server_type, url, username
             FROM server_profiles
             WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| format!("Failed to load server profile: {error}"))?;

        row.map(profile_from_row).transpose()
    }

    pub async fn load_all(&self) -> Result<Vec<StoredServerProfile>, String> {
        let rows = sqlx::query(
            "SELECT id, server_type, url, username
             FROM server_profiles
             ORDER BY rowid",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| format!("Failed to load server profiles: {error}"))?;

        rows.into_iter().map(profile_from_row).collect()
    }

    pub async fn save(&self, profile: &StoredServerProfile) -> Result<(), String> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| format!("Failed to start server profile save: {error}"))?;
        save_profile(&mut transaction, profile).await?;
        save_active_profile_id(&mut transaction, &profile.id).await?;
        transaction
            .commit()
            .await
            .map_err(|error| format!("Failed to save server profile: {error}"))
    }

    pub async fn delete(&self, id: Option<&str>) -> Result<Option<StoredServerProfile>, String> {
        let deleted = match id {
            Some(id) => self.load_by_id(id).await?,
            None => match self.active_profile_id().await? {
                Some(active_profile_id) => match self.load_by_id(&active_profile_id).await? {
                    Some(profile) => Some(profile),
                    None => self.load_first().await?,
                },
                None => self.load_first().await?,
            },
        };
        let Some(deleted) = deleted else {
            return Ok(None);
        };

        let active_profile_id = self.active_profile_id().await?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| format!("Failed to start server profile delete: {error}"))?;

        sqlx::query("DELETE FROM server_profiles WHERE id = ?")
            .bind(&deleted.id)
            .execute(&mut *transaction)
            .await
            .map_err(|error| format!("Failed to delete server profile: {error}"))?;

        if active_profile_id.as_deref() == Some(&deleted.id) {
            let next_profile_id =
                sqlx::query("SELECT id FROM server_profiles ORDER BY rowid LIMIT 1")
                    .fetch_optional(&mut *transaction)
                    .await
                    .map_err(|error| format!("Failed to choose next server profile: {error}"))?
                    .map(|row| row.get::<String, _>("id"));

            match next_profile_id {
                Some(next_profile_id) => {
                    save_active_profile_id(&mut transaction, &next_profile_id).await?
                }
                None => clear_active_profile_id(&mut transaction).await?,
            }
        }

        transaction
            .commit()
            .await
            .map_err(|error| format!("Failed to delete server profile: {error}"))?;

        Ok(Some(deleted))
    }

    async fn active_profile_id(&self) -> Result<Option<String>, String> {
        sqlx::query("SELECT value FROM app_state WHERE key = ?")
            .bind(ACTIVE_SERVER_PROFILE_KEY)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| format!("Failed to load active server profile: {error}"))
            .map(|row| row.map(|row| row.get("value")))
    }

    async fn load_first(&self) -> Result<Option<StoredServerProfile>, String> {
        let row = sqlx::query(
            "SELECT id, server_type, url, username
             FROM server_profiles
             ORDER BY rowid
             LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| format!("Failed to load server profile: {error}"))?;

        row.map(profile_from_row).transpose()
    }
}

async fn save_profile(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    profile: &StoredServerProfile,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO server_profiles (id, server_type, url, username)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             server_type = excluded.server_type,
             url = excluded.url,
             username = excluded.username",
    )
    .bind(&profile.id)
    .bind(profile.server_type.as_storage_value())
    .bind(&profile.url)
    .bind(&profile.username)
    .execute(&mut **transaction)
    .await
    .map_err(|error| format!("Failed to save server profile: {error}"))?;
    Ok(())
}

async fn save_active_profile_id(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    profile_id: &str,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO app_state (key, value)
         VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(ACTIVE_SERVER_PROFILE_KEY)
    .bind(profile_id)
    .execute(&mut **transaction)
    .await
    .map_err(|error| format!("Failed to save active server profile: {error}"))?;
    Ok(())
}

async fn clear_active_profile_id(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM app_state WHERE key = ?")
        .bind(ACTIVE_SERVER_PROFILE_KEY)
        .execute(&mut **transaction)
        .await
        .map_err(|error| format!("Failed to clear active server profile: {error}"))?;
    Ok(())
}

fn profile_from_row(row: sqlx::sqlite::SqliteRow) -> Result<StoredServerProfile, String> {
    let server_type = ServerType::from_storage_value(row.get("server_type"))?;
    Ok(StoredServerProfile {
        id: row.get("id"),
        server_type,
        url: row.get("url"),
        username: row.get("username"),
    })
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::ServerProfileStore;
    use crate::{
        database::{SqliteRepository, DATABASE_FILE_NAME},
        server::models::{ServerType, StoredServerProfile},
    };

    #[test]
    fn saves_profile() {
        tauri::async_runtime::block_on(async {
            let (store, repository, directory) = store().await;
            let profile = StoredServerProfile {
                id: Uuid::new_v4().to_string(),
                server_type: ServerType::Navidrome,
                url: "https://music.example.com".to_string(),
                username: "listener".to_string(),
            };

            store.save(&profile).await.unwrap();

            let loaded = store.load().await.unwrap().unwrap();
            assert_eq!(loaded.id, profile.id);
            assert_eq!(loaded.url, profile.url);
            assert_eq!(loaded.username, profile.username);

            store.delete(Some(&profile.id)).await.unwrap();
            repository.close().await;
            std::fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn saves_multiple_profiles() {
        tauri::async_runtime::block_on(async {
            let (store, repository, directory) = store().await;
            let first = StoredServerProfile {
                id: Uuid::new_v4().to_string(),
                server_type: ServerType::Navidrome,
                url: "https://first.example.com".to_string(),
                username: "first".to_string(),
            };
            let second = StoredServerProfile {
                id: Uuid::new_v4().to_string(),
                server_type: ServerType::Navidrome,
                url: "https://second.example.com".to_string(),
                username: "second".to_string(),
            };

            store.save(&first).await.unwrap();
            store.save(&second).await.unwrap();

            let profiles = store.load_all().await.unwrap();
            assert_eq!(profiles.len(), 2);
            assert_eq!(store.load().await.unwrap().unwrap().id, second.id);
            assert_eq!(
                store.load_by_id(&first.id).await.unwrap().unwrap().url,
                first.url
            );

            store.delete(Some(&second.id)).await.unwrap();
            assert_eq!(store.load().await.unwrap().unwrap().id, first.id);

            store.delete(Some(&first.id)).await.unwrap();
            assert!(store.load().await.unwrap().is_none());
            repository.close().await;
            std::fs::remove_dir_all(directory).unwrap();
        });
    }

    async fn store() -> (ServerProfileStore, SqliteRepository, std::path::PathBuf) {
        let directory = std::env::temp_dir().join(format!("solme-profile-{}", Uuid::new_v4()));
        let repository = SqliteRepository::open(&directory.join(DATABASE_FILE_NAME))
            .await
            .unwrap();
        let store = ServerProfileStore::new(repository.pool.clone());
        (store, repository, directory)
    }
}
