use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AudioStatusChange {
    Changed,
    BecameIdle { last_entry_id: Option<i64> },
}

pub type AudioStatusChangeCallback = Arc<dyn Fn(AudioStatusChange) + Send + Sync>;

pub trait AudioBackend: Send + Sync {
    fn playlist(&self) -> Result<AudioPlaylist, String>;
    fn load_source(&self, source: &str, paused_position: Option<f64>) -> Result<(), String>;
    fn insert_source(&self, source: &str, position: usize) -> Result<(), String>;
    fn append_source(&self, source: &str) -> Result<(), String>;
    /// Move an entry to its final zero-based index without restarting playback.
    fn move_queue_entry(&self, from: usize, to: usize) -> Result<(), String>;
    fn remove_queue_entry(&self, index: usize) -> Result<(), String>;
    fn pause_immediately(&self) -> Result<(), String>;
    fn pause(&self) -> Result<(), String>;
    fn resume(&self) -> Result<(), String>;
    fn stop(&self) -> Result<(), String>;
    fn next(&self) -> Result<(), String>;
    fn previous(&self) -> Result<(), String>;
    fn skip_to_queue_position(&self, queue_position: usize) -> Result<(), String>;
    fn seek(&self, position_seconds: f64) -> Result<(), String>;
    fn set_volume(&self, volume: f64) -> Result<(), String>;
    fn status(&self) -> AudioBackendStatus;
    /// Registers a callback fired when backend playback state may have changed.
    ///
    /// Backends should use this for changes that happen outside explicit
    /// service calls, such as advancing to the next playlist item. The
    /// audio module reads the current playlist and joins IDs with queue metadata.
    /// `BecameIdle` distinguishes completed playback from a queue built while idle.
    fn set_status_change_callback(&self, callback: AudioStatusChangeCallback);
}

pub struct AudioBackendStatus {
    pub playing: bool,
    pub paused: bool,
    pub position_seconds: f64,
    pub duration_seconds: f64,
    pub volume: f64,
}

#[derive(Clone, Debug, Default)]
pub struct AudioPlaylist {
    pub entries: Vec<AudioPlaylistEntry>,
}

impl AudioPlaylist {
    pub fn current_index(&self) -> Option<usize> {
        self.entries
            .iter()
            .position(|entry| entry.playing)
            .or_else(|| self.entries.iter().position(|entry| entry.current))
    }
}

#[derive(Clone, Debug)]
pub struct AudioPlaylistEntry {
    pub id: i64,
    pub source: String,
    pub current: bool,
    pub playing: bool,
}
