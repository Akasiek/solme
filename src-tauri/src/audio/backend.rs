use std::sync::Arc;

pub type AudioStatusChangeCallback = Arc<dyn Fn() + Send + Sync>;

pub trait AudioBackend: Send + Sync {
    fn load_queue(&self, sources: &[String], start_index: usize) -> Result<(), String>;
    fn load_queue_paused(
        &self,
        sources: &[String],
        start_index: usize,
        position_seconds: Option<f64>,
    ) -> Result<(), String>;
    fn prepend_queue(&self, sources: &[String]) -> Result<(), String>;
    fn append_queue(&self, sources: &[String]) -> Result<(), String>;
    fn pause(&self) -> Result<(), String>;
    fn resume(&self) -> Result<(), String>;
    fn stop(&self) -> Result<(), String>;
    fn next(&self) -> Result<(), String>;
    fn previous(&self) -> Result<(), String>;
    fn seek(&self, position_seconds: f64) -> Result<(), String>;
    fn set_volume(&self, volume: f64) -> Result<(), String>;
    fn status(&self) -> AudioBackendStatus;
    /// Registers a callback fired when backend playback state may have changed.
    ///
    /// Backends should use this for changes that happen outside explicit
    /// service calls, such as advancing to the next playlist item. The
    /// callback is payload-free because `PlayerService` owns the cached queue
    /// and builds the full `PlayerStatus`.
    fn set_status_change_callback(&self, callback: AudioStatusChangeCallback);
}

pub struct AudioBackendStatus {
    pub playing: bool,
    pub paused: bool,
    pub position_seconds: f64,
    pub duration_seconds: f64,
    pub playlist_position: Option<usize>,
    pub volume: f64,
}
