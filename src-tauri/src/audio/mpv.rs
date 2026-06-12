use std::path::Path;

use libmpv2::Mpv;

use super::backend::AudioBackend;

pub struct MpvBackend {
    mpv: Mpv,
}

impl MpvBackend {
    pub fn new() -> Result<Self, String> {
        let mpv = Mpv::new().map_err(|error| format!("Failed to create mpv instance: {error}"))?;

        mpv.set_property("vo", "null")
            .map_err(|error| format!("Failed to disable video output: {error}"))?;
        mpv.set_property("idle", true)
            .map_err(|error| format!("Failed to enable mpv idle mode: {error}"))?;

        Ok(Self { mpv })
    }
}

impl AudioBackend for MpvBackend {
    fn play_file(&self, path: String) -> Result<(), String> {
        if !Path::new(&path).is_file() {
            return Err(format!("Audio file does not exist: {path}"));
        }

        self.mpv
            .command("loadfile", &[&path, "replace"])
            .map_err(|error| format!("Failed to play audio file: {error}"))
    }

    fn pause(&self) -> Result<(), String> {
        self.mpv
            .set_property("pause", true)
            .map_err(|error| format!("Failed to pause playback: {error}"))
    }

    fn resume(&self) -> Result<(), String> {
        self.mpv
            .set_property("pause", false)
            .map_err(|error| format!("Failed to resume playback: {error}"))
    }

    fn stop(&self) -> Result<(), String> {
        self.mpv
            .command("stop", &[])
            .map_err(|error| format!("Failed to stop playback: {error}"))
    }

    fn set_volume(&self, volume: f64) -> Result<(), String> {
        self.mpv
            .set_property("volume", volume.clamp(0.0, 100.0))
            .map_err(|error| format!("Failed to set volume: {error}"))
    }
}
