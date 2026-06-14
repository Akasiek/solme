use libmpv2::{Mpv, SetData};

use super::backend::{AudioBackend, AudioBackendStatus};

pub struct MpvBackend {
    mpv: Mpv,
}

impl MpvBackend {
    pub fn new() -> Result<Self, String> {
        set_numeric_locale()?;
        let mpv = Mpv::new().map_err(|error| format!("Failed to create mpv instance: {error}"))?;
        let audio = Self { mpv };

        audio.set_property("vo", "null", "disable video output")?;
        audio.set_property("idle", true, "enable mpv idle mode")?;

        Ok(audio)
    }

    fn set_property<T: SetData>(
        &self,
        property: &str,
        value: T,
        action: &str,
    ) -> Result<(), String> {
        self.mpv
            .set_property(property, value)
            .map_err(|error| format!("Failed to {action}: {error}"))
    }

    fn set_paused(&self, paused: bool, action: &str) -> Result<(), String> {
        self.set_property("pause", paused, action)
    }

    fn execute_command(
        &self,
        command: &str,
        arguments: &[&str],
        action: &str,
    ) -> Result<(), String> {
        self.mpv
            .command(command, arguments)
            .map_err(|error| format!("Failed to {action}: {error}"))
    }
}

/// Configures the process-wide numeric locale required by libmpv.
///
/// libmpv expects floating-point values to use a dot as the decimal separator
/// and refuses to initialize under locales that use a different separator.
/// Only `LC_NUMERIC` is changed, and it must be set before calling `Mpv::new`.
fn set_numeric_locale() -> Result<(), String> {
    let locale = unsafe { libc::setlocale(libc::LC_NUMERIC, c"C".as_ptr()) };
    if locale.is_null() {
        return Err("Failed to set LC_NUMERIC to C for mpv".to_string());
    }
    Ok(())
}

impl AudioBackend for MpvBackend {
    fn load_queue(&self, sources: &[String], start_index: usize) -> Result<(), String> {
        if sources.is_empty() {
            return Err("Cannot play an empty queue".to_string());
        }
        if start_index >= sources.len() {
            return Err("Queue start index is out of bounds".to_string());
        }

        self.set_paused(true, "pause while loading queue")?;
        self.execute_command(
            "loadfile",
            &[&sources[0], "replace"],
            "load first queue item",
        )?;
        for source in &sources[1..] {
            self.execute_command("loadfile", &[source, "append"], "append queue item")?;
        }
        self.set_property("playlist-pos", start_index as i64, "select queue item")?;
        self.set_paused(false, "start playback")
    }

    fn pause(&self) -> Result<(), String> {
        self.set_paused(true, "pause playback")
    }

    fn resume(&self) -> Result<(), String> {
        self.set_paused(false, "resume playback")
    }

    fn stop(&self) -> Result<(), String> {
        self.execute_command("stop", &[], "stop playback")
    }

    fn next(&self) -> Result<(), String> {
        self.execute_command("playlist-next", &["force"], "play next track")
    }

    fn previous(&self) -> Result<(), String> {
        self.execute_command("playlist-prev", &["force"], "play previous track")
    }

    fn seek(&self, position_seconds: f64) -> Result<(), String> {
        self.execute_command(
            "seek",
            &[&position_seconds.max(0.0).to_string(), "absolute", "exact"],
            "seek playback",
        )
    }

    fn set_volume(&self, volume: f64) -> Result<(), String> {
        self.set_property("volume", volume.clamp(0.0, 100.0), "set volume")
    }

    fn status(&self) -> AudioBackendStatus {
        let playing = !self.mpv.get_property::<bool>("idle-active").unwrap_or(true);

        AudioBackendStatus {
            playing,
            paused: playing && self.mpv.get_property("pause").unwrap_or(false),
            position_seconds: self.mpv.get_property("time-pos").unwrap_or(0.0),
            duration_seconds: self.mpv.get_property("duration").unwrap_or(0.0),
            volume: self.mpv.get_property("volume").unwrap_or(100.0),
            playlist_position: self
                .mpv
                .get_property::<i64>("playlist-pos")
                .ok()
                .filter(|position| *position >= 0)
                .map(|position| position as usize),
        }
    }
}
