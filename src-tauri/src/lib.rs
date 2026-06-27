use crate::commands::library::{
    get_cached_album, get_cached_albums, get_cached_songs, get_library_summary,
    get_library_sync_status, search_cached_albums, sync_library,
};
use crate::commands::player::{
    get_player_status, next, pause, play_album, previous, queue_album_at_end, queue_album_at_start,
    resume, seek, set_volume, stop,
};
use crate::commands::server::{
    connect_music_server, connect_saved_music_server, forget_saved_server_profile,
    get_saved_server_profile, ping_music_server,
};
use crate::setup::setup_app;

mod audio;
mod commands;
mod credentials;
mod database;
mod events;
mod library;
mod server;
mod setup;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(setup_app)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            play_album,
            queue_album_at_start,
            queue_album_at_end,
            pause,
            resume,
            stop,
            next,
            previous,
            seek,
            set_volume,
            get_player_status,
            connect_music_server,
            ping_music_server,
            get_saved_server_profile,
            connect_saved_music_server,
            forget_saved_server_profile,
            sync_library,
            get_library_sync_status,
            get_library_summary,
            get_cached_album,
            get_cached_albums,
            get_cached_songs,
            search_cached_albums
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
