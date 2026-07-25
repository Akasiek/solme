use sqlx::SqlitePool;

use super::{models::StoredServerProfile, query};

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
        query::load_profile(&self.pool, id).await
    }

    pub async fn load_all(&self) -> Result<Vec<StoredServerProfile>, String> {
        query::load_profiles(&self.pool).await
    }

    pub async fn save(&self, profile: &StoredServerProfile) -> Result<(), String> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| format!("Failed to start server profile save: {error}"))?;
        query::save_profile(&mut transaction, profile).await?;
        query::save_active_profile_id(&mut transaction, &profile.id).await?;
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

        query::delete_profile_data(&mut transaction, &deleted.id).await?;
        query::delete_profile(&mut transaction, &deleted.id).await?;

        if active_profile_id.as_deref() == Some(&deleted.id) {
            match query::next_profile_id(&mut transaction).await? {
                Some(next_profile_id) => {
                    query::save_active_profile_id(&mut transaction, &next_profile_id).await?
                }
                None => query::clear_active_profile_id(&mut transaction).await?,
            }
        }

        transaction
            .commit()
            .await
            .map_err(|error| format!("Failed to delete server profile: {error}"))?;

        Ok(Some(deleted))
    }

    async fn active_profile_id(&self) -> Result<Option<String>, String> {
        query::active_profile_id(&self.pool).await
    }

    async fn load_first(&self) -> Result<Option<StoredServerProfile>, String> {
        query::load_first_profile(&self.pool).await
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{query, ServerProfileStore};
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
                secondary_url: Some("https://music-fallback.example.com".to_string()),
                username: "listener".to_string(),
            };

            store.save(&profile).await.unwrap();

            let loaded = store.load().await.unwrap().unwrap();
            assert_eq!(loaded.id, profile.id);
            assert_eq!(loaded.url, profile.url);
            assert_eq!(loaded.secondary_url, profile.secondary_url);
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
                secondary_url: None,
                username: "first".to_string(),
            };
            let second = StoredServerProfile {
                id: Uuid::new_v4().to_string(),
                server_type: ServerType::Navidrome,
                url: "https://second.example.com".to_string(),
                secondary_url: Some("https://second-fallback.example.com".to_string()),
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

    #[test]
    fn deleting_profile_removes_only_its_persisted_data() {
        tauri::async_runtime::block_on(async {
            let (store, repository, directory) = store().await;
            let deleted = profile("deleted");
            let retained = profile("retained");
            store.save(&deleted).await.unwrap();
            store.save(&retained).await.unwrap();
            query::seed_profile_data(&repository.pool, &deleted.id, "deleted").await;
            query::seed_profile_data(&repository.pool, &retained.id, "retained").await;

            store.delete(Some(&deleted.id)).await.unwrap();

            assert!(store.load_by_id(&deleted.id).await.unwrap().is_none());
            assert!(store.load_by_id(&retained.id).await.unwrap().is_some());
            for table in query::profile_data_tables() {
                assert_eq!(
                    query::profile_row_count(&repository.pool, table, &deleted.id).await,
                    0
                );
                assert_eq!(
                    query::profile_row_count(&repository.pool, table, &retained.id).await,
                    1
                );
            }

            repository.close().await;
            std::fs::remove_dir_all(directory).unwrap();
        });
    }

    fn profile(name: &str) -> StoredServerProfile {
        StoredServerProfile {
            id: Uuid::new_v4().to_string(),
            server_type: ServerType::Navidrome,
            url: format!("https://{name}.example.com"),
            secondary_url: None,
            username: name.to_string(),
        }
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
