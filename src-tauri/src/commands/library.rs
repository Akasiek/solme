use std::sync::Arc;

use tauri::State;

use crate::library::{CachedAlbum, LibrarySummary, LibrarySyncService, LibrarySyncStatus};

#[tauri::command]
pub fn sync_library(
    force: bool,
    library: State<'_, Arc<LibrarySyncService>>,
) -> Result<(), String> {
    library.start(force)
}

#[tauri::command]
pub fn get_library_sync_status(
    library: State<'_, Arc<LibrarySyncService>>,
) -> Result<LibrarySyncStatus, String> {
    library.status()
}

#[tauri::command]
pub async fn get_library_summary(
    library: State<'_, Arc<LibrarySyncService>>,
) -> Result<LibrarySummary, String> {
    library.summary().await
}

#[tauri::command]
pub async fn get_cached_albums(
    offset: i64,
    limit: i64,
    library: State<'_, Arc<LibrarySyncService>>,
) -> Result<Vec<CachedAlbum>, String> {
    library.albums(offset, limit).await
}
