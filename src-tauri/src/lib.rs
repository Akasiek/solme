use std::sync::Arc;

use crate::audio::{MpvBackend, PlayerService};
use crate::commands::library::{
    get_cached_albums, get_library_summary, get_library_sync_status, sync_library,
};
use crate::commands::player::{pause, play_file, resume, set_volume, stop};
use crate::commands::server::{
    connect_music_server, connect_saved_music_server, forget_saved_server_profile,
    get_saved_server_profile, ping_music_server,
};
use crate::credentials::SystemCredentialStore;
use crate::library::{LibrarySyncService, SqliteLibraryRepository};
use crate::server::MusicServerService;
use tauri::Manager;

mod audio;
mod commands;
mod credentials;
mod library;
mod server;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let backend = MpvBackend::new().expect("Failed to initialize mpv backend");
    let player = PlayerService::new(Box::new(backend));

    tauri::Builder::default()
        .manage(player)
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            let data_dir = app.path().app_data_dir()?;
            let cache_dir = app.path().app_cache_dir()?;
            let server = Arc::new(MusicServerService::new(
                config_dir.join("server-profile.json"),
                Box::new(SystemCredentialStore::new().map_err(std::io::Error::other)?),
            ));
            let repository = Arc::new(
                tauri::async_runtime::block_on(SqliteLibraryRepository::open(
                    &data_dir.join("library.sqlite"),
                ))
                .map_err(std::io::Error::other)?,
            );
            let library = Arc::new(LibrarySyncService::new(
                Arc::clone(&server),
                repository,
                cache_dir.join("artwork"),
            ));

            app.manage(Arc::clone(&server));
            app.manage(Arc::clone(&library));
            tauri::async_runtime::spawn(async move {
                if server.connect_saved().await.is_ok() {
                    let _ = library.start(false);
                }
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            play_file,
            pause,
            resume,
            stop,
            set_volume,
            connect_music_server,
            ping_music_server,
            get_saved_server_profile,
            connect_saved_music_server,
            forget_saved_server_profile,
            sync_library,
            get_library_sync_status,
            get_library_summary,
            get_cached_albums
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
