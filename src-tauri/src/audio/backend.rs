pub trait AudioBackend: Send + Sync {
    fn load_queue(&self, sources: &[String], start_index: usize) -> Result<(), String>;
    fn append_queue(&self, sources: &[String]) -> Result<(), String>;
    fn pause(&self) -> Result<(), String>;
    fn resume(&self) -> Result<(), String>;
    fn stop(&self) -> Result<(), String>;
    fn next(&self) -> Result<(), String>;
    fn previous(&self) -> Result<(), String>;
    fn seek(&self, position_seconds: f64) -> Result<(), String>;
    fn set_volume(&self, volume: f64) -> Result<(), String>;
    fn status(&self) -> AudioBackendStatus;
}

pub struct AudioBackendStatus {
    pub playing: bool,
    pub paused: bool,
    pub position_seconds: f64,
    pub duration_seconds: f64,
    pub playlist_position: Option<usize>,
    pub volume: f64,
}
