use std::sync::Arc;

use tauri::State;

use crate::audio::{PlayerService, PlayerStatus};

#[tauri::command]
pub async fn player_play_album(
    album_id: String,
    start_song_id: Option<String>,
    player: State<'_, Arc<PlayerService>>,
) -> Result<(), String> {
    player.play_album(&album_id, start_song_id.as_deref()).await
}

#[tauri::command]
pub async fn player_queue_album_at_start(
    album_id: String,
    player: State<'_, Arc<PlayerService>>,
) -> Result<(), String> {
    player.queue_album_at_start(&album_id).await
}

#[tauri::command]
pub async fn player_queue_album_at_end(
    album_id: String,
    player: State<'_, Arc<PlayerService>>,
) -> Result<(), String> {
    player.queue_album_at_end(&album_id).await
}

#[tauri::command]
pub fn player_pause(player: State<'_, Arc<PlayerService>>) -> Result<(), String> {
    player.pause()
}

#[tauri::command]
pub fn player_resume(player: State<'_, Arc<PlayerService>>) -> Result<(), String> {
    player.resume()
}

#[tauri::command]
pub fn player_stop(player: State<'_, Arc<PlayerService>>) -> Result<(), String> {
    player.stop()
}

#[tauri::command]
pub fn player_next(player: State<'_, Arc<PlayerService>>) -> Result<(), String> {
    player.next()
}

#[tauri::command]
pub fn player_previous(player: State<'_, Arc<PlayerService>>) -> Result<(), String> {
    player.previous()
}

#[tauri::command]
pub fn player_seek(position_seconds: f64, player: State<'_, Arc<PlayerService>>) -> Result<(), String> {
    player.seek(position_seconds)
}

#[tauri::command]
pub fn player_set_volume(volume: f64, player: State<'_, Arc<PlayerService>>) -> Result<(), String> {
    player.set_volume(volume)
}

#[tauri::command]
pub fn get_player_status(player: State<'_, Arc<PlayerService>>) -> Result<PlayerStatus, String> {
    player.status()
}
