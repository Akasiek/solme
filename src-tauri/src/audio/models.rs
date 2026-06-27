use serde::Serialize;

use crate::library::CachedSong;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStatus {
    pub state: PlaybackState,
    pub current_song: Option<CachedSong>,
    pub position_seconds: f64,
    pub duration_seconds: f64,
    pub queue_position: Option<usize>,
    pub queue_length: usize,
    pub volume: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}
