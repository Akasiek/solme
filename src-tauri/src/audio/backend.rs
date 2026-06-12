pub trait AudioBackend: Send + Sync {
    fn play_file(&self, path: String) -> Result<(), String>;
    fn pause(&self) -> Result<(), String>;
    fn resume(&self) -> Result<(), String>;
    fn stop(&self) -> Result<(), String>;
    fn set_volume(&self, volume: f64) -> Result<(), String>;
}
