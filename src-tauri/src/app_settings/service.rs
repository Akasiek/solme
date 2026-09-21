use std::sync::Arc;

use super::{AppSettingKey, AppSettingUpdate, AppSettings, AppSettingsRepository};

pub struct AppSettingsService {
    repository: Arc<dyn AppSettingsRepository>,
}

impl AppSettingsService {
    pub fn new(repository: Arc<dyn AppSettingsRepository>) -> Self {
        Self { repository }
    }

    pub async fn load(&self) -> Result<AppSettings, String> {
        let mut settings = AppSettings::default();

        for stored in self.repository.load_app_settings().await? {
            if let Some(key) = AppSettingKey::parse(&stored.key) {
                settings.apply(key, &stored.value)?;
            }
        }

        Ok(settings)
    }

    pub async fn update(&self, update: AppSettingUpdate) -> Result<AppSettings, String> {
        let (key, value) = update.into_stored();
        self.repository.save_app_setting(key, &value).await?;

        self.load().await
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use async_trait::async_trait;

    use super::*;
    use crate::app_settings::repository::StoredAppSetting;

    #[derive(Default)]
    struct MockRepository {
        values: Mutex<HashMap<String, String>>,
    }

    #[async_trait]
    impl AppSettingsRepository for MockRepository {
        async fn load_app_settings(&self) -> Result<Vec<StoredAppSetting>, String> {
            let values = self.values.lock().map_err(|error| error.to_string())?;

            Ok(values
                .iter()
                .map(|(key, value)| StoredAppSetting {
                    key: key.clone(),
                    value: value.clone(),
                })
                .collect())
        }

        async fn save_app_setting(&self, key: AppSettingKey, value: &str) -> Result<(), String> {
            self.values
                .lock()
                .map_err(|error| error.to_string())?
                .insert(key.as_str().to_string(), value.to_string());
            Ok(())
        }
    }

    #[test]
    fn defaults_away_lyrics_to_enabled() {
        tauri::async_runtime::block_on(async {
            let service = AppSettingsService::new(Arc::new(MockRepository::default()));

            assert_eq!(service.load().await.unwrap(), AppSettings::default());
        });
    }

    #[test]
    fn updates_and_loads_away_lyrics_setting() {
        tauri::async_runtime::block_on(async {
            let service = AppSettingsService::new(Arc::new(MockRepository::default()));

            let settings = service
                .update(AppSettingUpdate::AwayLyricsEnabled(false))
                .await
                .unwrap();

            assert!(!settings.away_lyrics_enabled);
            assert_eq!(service.load().await.unwrap(), settings);
        });
    }
}
