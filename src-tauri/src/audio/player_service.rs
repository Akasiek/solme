use super::backend::AudioBackend;

pub struct PlayerService {
    backend: Box<dyn AudioBackend>,
}

impl PlayerService {
    pub fn new(backend: Box<dyn AudioBackend>) -> Self {
        Self { backend }
    }

    pub fn play_file(&self, path: String) -> Result<(), String> {
        self.backend.play_file(path)
    }

    pub fn pause(&self) -> Result<(), String> {
        self.backend.pause()
    }

    pub fn resume(&self) -> Result<(), String> {
        self.backend.resume()
    }

    pub fn stop(&self) -> Result<(), String> {
        self.backend.stop()
    }

    pub fn set_volume(&self, volume: f64) -> Result<(), String> {
        self.backend.set_volume(volume)
    }
}
