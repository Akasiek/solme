use crate::audio::{MpvBackend, PlayerService};
use crate::commands::player::{pause, play_file, resume, set_volume, stop};

mod audio;
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let backend = MpvBackend::new().expect("Failed to initialize mpv backend");
    let player = PlayerService::new(Box::new(backend));

    tauri::Builder::default()
        .manage(player)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            play_file, pause, resume, stop, set_volume
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
