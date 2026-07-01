use crate::commands::library::{
    get_cached_album, get_cached_albums, get_cached_songs, get_library_summary,
    get_library_sync_status, search_cached_albums, search_cached_songs, sync_library,
};
use crate::commands::player::{
    get_player_status, player_next, player_pause, player_play_album, player_previous,
    player_queue_album_at_end, player_queue_album_at_start, player_resume, player_seek,
    player_set_volume, player_stop,
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
            player_play_album,
            player_queue_album_at_start,
            player_queue_album_at_end,
            player_pause,
            player_resume,
            player_stop,
            player_next,
            player_previous,
            player_seek,
            player_set_volume,
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
            search_cached_albums,
            search_cached_songs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
