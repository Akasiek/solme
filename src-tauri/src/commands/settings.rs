use std::sync::Arc;

use tauri::State;

use crate::app_settings::{AppSettingUpdate, AppSettings, AppSettingsService};

#[tauri::command]
pub async fn get_app_settings(
    settings: State<'_, Arc<AppSettingsService>>,
) -> Result<AppSettings, String> {
    settings.load().await
}

#[tauri::command]
pub async fn update_app_setting(
    update: AppSettingUpdate,
    settings: State<'_, Arc<AppSettingsService>>,
) -> Result<AppSettings, String> {
    settings.update(update).await
}
