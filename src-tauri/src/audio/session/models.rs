use crate::library::CachedSong;

#[derive(Clone, Debug, PartialEq)]
pub struct PlaybackSession {
    pub queue: Vec<CachedSong>,
    pub active_index: usize,
    pub position_seconds: f64,
}
