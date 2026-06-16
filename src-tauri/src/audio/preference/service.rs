use std::sync::Arc;

use crate::server::MusicServerService;

use super::{Preference, PreferenceKey, PreferenceRepository};

pub struct PreferenceService {
    server: Arc<MusicServerService>,
    repository: Arc<dyn PreferenceRepository>,
}

impl PreferenceService {
    pub fn new(server: Arc<MusicServerService>, repository: Arc<dyn PreferenceRepository>) -> Self {
        Self { server, repository }
    }

    pub async fn load_volume(&self) -> Result<Option<f64>, String> {
        let (profile_id, _) = self.server.current_server()?;
        let Some(preference) = self
            .repository
            .load(&profile_id, PreferenceKey::Volume)
            .await?
        else {
            return Ok(None);
        };

        Ok(preference.volume_value())
    }

    pub fn save_volume(&self, volume: f64) {
        let Ok((profile_id, _)) = self.server.current_server() else {
            return;
        };

        let repository = Arc::clone(&self.repository);
        tauri::async_runtime::spawn(async move {
            let preference = Preference::volume(volume);
            let _ = repository.save(&profile_id, Some(&preference)).await;
        });
    }
}
