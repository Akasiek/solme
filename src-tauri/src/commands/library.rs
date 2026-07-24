use std::sync::Arc;

use tauri::State;

use crate::library::{
    CachedAlbum, CachedAlbumDetails, CachedArtist, CachedArtistDetails, CachedSong,
    HomeAlbumSections, LibraryCatalogService, LibrarySummary, LibrarySyncService,
    LibrarySyncStatus,
};

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
    library: State<'_, Arc<LibraryCatalogService>>,
) -> Result<LibrarySummary, String> {
    library.summary().await
}

#[tauri::command]
pub async fn get_cached_artist(
    artist_id: String,
    library: State<'_, Arc<LibraryCatalogService>>,
) -> Result<Option<CachedArtistDetails>, String> {
    library.artist(&artist_id).await
}

#[tauri::command]
pub async fn get_cached_albums(
    offset: i64,
    limit: i64,
    library: State<'_, Arc<LibraryCatalogService>>,
) -> Result<Vec<CachedAlbum>, String> {
    library.albums(offset, limit).await
}

#[tauri::command]
pub async fn get_home_album_sections(
    limit: i64,
    library: State<'_, Arc<LibraryCatalogService>>,
) -> Result<HomeAlbumSections, String> {
    library.home_album_sections(limit).await
}

#[tauri::command]
pub async fn get_cached_album(
    album_id: String,
    library: State<'_, Arc<LibraryCatalogService>>,
) -> Result<Option<CachedAlbumDetails>, String> {
    library.album(&album_id).await
}

#[tauri::command]
pub async fn search_cached_albums(
    query: String,
    limit: i64,
    library: State<'_, Arc<LibraryCatalogService>>,
) -> Result<Vec<CachedAlbum>, String> {
    library.search_albums(&query, limit).await
}

#[tauri::command]
pub async fn search_cached_artists(
    query: String,
    limit: i64,
    library: State<'_, Arc<LibraryCatalogService>>,
) -> Result<Vec<CachedArtist>, String> {
    library.search_artists(&query, limit).await
}

#[tauri::command]
pub async fn search_cached_songs(
    query: String,
    limit: i64,
    library: State<'_, Arc<LibraryCatalogService>>,
) -> Result<Vec<CachedSong>, String> {
    library.search_songs(&query, limit).await
}

#[tauri::command]
pub async fn get_cached_songs(
    album_id: String,
    library: State<'_, Arc<LibraryCatalogService>>,
) -> Result<Vec<CachedSong>, String> {
    library.songs(&album_id).await
}
