use async_trait::async_trait;

use crate::database::SqliteRepository;

use super::{Preference, PreferenceKey};

#[async_trait]
pub trait PreferenceRepository: Send + Sync {
    async fn load(
        &self,
        profile_id: &str,
        key: PreferenceKey,
    ) -> Result<Option<Preference>, String>;
    async fn save(&self, profile_id: &str, preference: Option<&Preference>) -> Result<(), String>;
}

#[async_trait]
impl PreferenceRepository for SqliteRepository {
    async fn load(
        &self,
        profile_id: &str,
        key: PreferenceKey,
    ) -> Result<Option<Preference>, String> {
        let row = sqlx::query_as::<_, StoredPreference>(
            "SELECT key, value FROM preferences WHERE profile_id = ? AND key = ?",
        )
        .bind(profile_id)
        .bind(key.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| format!("Failed to read preference: {error}"))?;

        row.map(TryInto::try_into).transpose()
    }

    async fn save(&self, profile_id: &str, preference: Option<&Preference>) -> Result<(), String> {
        let Some(preference) = preference else {
            sqlx::query("DELETE FROM preferences WHERE profile_id = ?")
                .bind(profile_id)
                .execute(&self.pool)
                .await
                .map_err(|error| format!("Failed to delete preference: {error}"))?;
            return Ok(());
        };

        sqlx::query(
            "INSERT INTO preferences (profile_id, key, value) VALUES (?, ?, ?)
             ON CONFLICT(profile_id, key) DO UPDATE SET value = excluded.value",
        )
        .bind(profile_id)
        .bind(preference.key.as_str())
        .bind(preference.value.serialize())
        .execute(&self.pool)
        .await
        .map_err(|error| format!("Failed to save preference: {error}"))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct StoredPreference {
    key: String,
    value: String,
}

impl TryFrom<StoredPreference> for Preference {
    type Error = String;

    fn try_from(stored: StoredPreference) -> Result<Self, Self::Error> {
        let key = PreferenceKey::parse(&stored.key)?;
        let value = super::models::PreferenceValue::parse(key, &stored.value)?;
        Ok(Self { key, value })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use uuid::Uuid;

    use super::PreferenceRepository;
    use crate::{
        audio::preference::{Preference, PreferenceKey},
        database::{SqliteRepository, DATABASE_FILE_NAME},
    };

    #[test]
    fn saves_loads_and_deletes_volume_preference() {
        tauri::async_runtime::block_on(async {
            let directory =
                std::env::temp_dir().join(format!("solme-preference-{}", Uuid::new_v4()));
            let repository = SqliteRepository::open(&directory.join(DATABASE_FILE_NAME))
                .await
                .unwrap();

            repository
                .save("profile", Some(&Preference::volume(42.5)))
                .await
                .unwrap();
            assert_eq!(
                repository
                    .load("profile", PreferenceKey::Volume)
                    .await
                    .unwrap()
                    .and_then(|preference| preference.volume_value()),
                Some(42.5)
            );

            repository.save("profile", None).await.unwrap();
            assert_eq!(
                repository
                    .load("profile", PreferenceKey::Volume)
                    .await
                    .unwrap(),
                None
            );

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }
}
