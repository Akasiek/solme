use tauri::State;

use crate::audio::PlayerService;

#[tauri::command]
pub fn play_file(path: String, player: State<PlayerService>) -> Result<(), String> {
    player.play_file(path)
}

#[tauri::command]
pub fn pause(player: State<PlayerService>) -> Result<(), String> {
    player.pause()
}

#[tauri::command]
pub fn resume(player: State<PlayerService>) -> Result<(), String> {
    player.resume()
}

#[tauri::command]
pub fn stop(player: State<PlayerService>) -> Result<(), String> {
    player.stop()
}

#[tauri::command]
pub fn set_volume(volume: f64, player: State<PlayerService>) -> Result<(), String> {
    player.set_volume(volume)
}
