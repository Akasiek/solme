#![allow(clippy::needless_pass_by_value)] // Tauri commands deserialize and own their arguments.

use std::sync::Arc;

use tauri::State;

use crate::audio::{PlayerService, PlayerStatus, QueueSnapshot};

#[tauri::command]
pub async fn player_play_album(
    album_id: String,
    start_song_id: Option<String>,
    player: State<'_, Arc<PlayerService>>,
) -> Result<(), String> {
    player.play_album(&album_id, start_song_id.as_deref()).await
}

#[tauri::command]
pub async fn player_play_albums(
    album_ids: Vec<String>,
    player: State<'_, Arc<PlayerService>>,
) -> Result<(), String> {
    player.play_albums(&album_ids).await
}

#[tauri::command]
pub async fn player_queue_album_next(
    album_id: String,
    player: State<'_, Arc<PlayerService>>,
) -> Result<(), String> {
    player.queue_album_next(&album_id).await
}

#[tauri::command]
pub async fn player_queue_album_last(
    album_id: String,
    player: State<'_, Arc<PlayerService>>,
) -> Result<(), String> {
    player.queue_album_last(&album_id).await
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
pub fn player_skip_to_queue_position(
    position: usize,
    player: State<'_, Arc<PlayerService>>,
) -> Result<(), String> {
    player.skip_to_queue_position(position)
}

#[tauri::command]
pub fn player_seek(
    position_seconds: f64,
    player: State<'_, Arc<PlayerService>>,
) -> Result<(), String> {
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

#[tauri::command]
pub fn get_player_queue(player: State<'_, Arc<PlayerService>>) -> Result<QueueSnapshot, String> {
    player.queue_snapshot()
}

#[tauri::command]
pub async fn player_move_queue_entry(
    from: usize,
    to: usize,
    expected_revision: String,
    expected_current_index: Option<usize>,
    player: State<'_, Arc<PlayerService>>,
) -> Result<(), String> {
    player
        .move_queue_entry(from, to, &expected_revision, expected_current_index)
        .await
}

#[tauri::command]
pub async fn player_queue_song(
    album_id: String,
    song_id: String,
    next: bool,
    player: State<'_, Arc<PlayerService>>,
) -> Result<(), String> {
    player.queue_song(&album_id, &song_id, next).await
}

#[tauri::command]
pub async fn player_play_queue_entry(
    entry_id: String,
    player: State<'_, Arc<PlayerService>>,
) -> Result<(), String> {
    player.play_queue_entry(&entry_id).await
}

#[tauri::command]
pub async fn player_remove_queue_entry(
    entry_id: String,
    player: State<'_, Arc<PlayerService>>,
) -> Result<(), String> {
    player.remove_queue_entry(&entry_id).await
}
