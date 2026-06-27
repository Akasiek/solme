use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use libmpv2::{events::Event, Format, Mpv, SetData};

use super::backend::{AudioBackend, AudioBackendStatus, AudioStatusChangeCallback};

pub struct MpvBackend {
    mpv: Mpv,
    status_change_callback: Arc<Mutex<Option<AudioStatusChangeCallback>>>,
}

impl MpvBackend {
    pub fn new() -> Result<Self, String> {
        set_numeric_locale()?;
        let mpv = Mpv::new().map_err(|error| format!("Failed to create mpv instance: {error}"))?;
        let status_change_callback = Arc::new(Mutex::new(None));
        Self::start_status_change_watcher(&mpv, Arc::clone(&status_change_callback))?;
        let audio = Self {
            mpv,
            status_change_callback,
        };

        audio.set_property("vo", "null", "disable video output")?;
        audio.set_property("idle", true, "enable mpv idle mode")?;

        Ok(audio)
    }

    fn start_status_change_watcher(
        mpv: &Mpv,
        callback: Arc<Mutex<Option<AudioStatusChangeCallback>>>,
    ) -> Result<(), String> {
        let mpv = mpv
            .create_client(Some("status_watcher"))
            .map_err(|error| format!("Failed to create mpv status watcher: {error}"))?;

        mpv.observe_property("playlist-pos", Format::Int64, 1)
            .map_err(|error| format!("Failed to observe mpv playlist position: {error}"))?;

        thread::spawn(move || loop {
            match mpv.wait_event(60.0) {
                Some(Ok(Event::Shutdown)) => break,
                Some(Ok(Event::StartFile | Event::FileLoaded)) => {
                    Self::notify_status_change(&callback)
                }
                Some(Ok(Event::PropertyChange {
                    name: "playlist-pos",
                    ..
                })) => Self::notify_status_change(&callback),
                Some(Ok(_)) | None => {}
                Some(Err(error)) => eprintln!("Failed to read mpv status event: {error}"),
            }
        });

        Ok(())
    }

    fn notify_status_change(callback: &Arc<Mutex<Option<AudioStatusChangeCallback>>>) {
        let callback = match callback.lock() {
            Ok(callback) => callback.clone(),
            Err(_) => {
                eprintln!("Failed to read mpv status change callback");
                return;
            }
        };

        if let Some(callback) = callback {
            callback();
        }
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

    fn load_queue_with_state(
        &self,
        sources: &[String],
        start_index: usize,
        paused: bool,
        position_seconds: Option<f64>,
    ) -> Result<(), String> {
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
        self.set_paused(paused, "set playback state after loading queue")?;

        if let Some(position_seconds) = position_seconds.filter(|p| *p > 0.0) {
            self.seek_when_file_is_loaded(position_seconds)?;
        }

        Ok(())
    }

    fn seek_when_file_is_loaded(&self, position_seconds: f64) -> Result<(), String> {
        if !self.wait_for_file_loaded(Duration::from_secs(5)) {
            return Ok(());
        }

        self.execute_command(
            "seek",
            &[&position_seconds.max(0.0).to_string(), "absolute", "exact"],
            "restore playback position",
        )
    }

    fn wait_for_file_loaded(&self, timeout: Duration) -> bool {
        let started_at = Instant::now();
        while let Some(remaining) = timeout.checked_sub(started_at.elapsed()) {
            match self.mpv.wait_event(remaining.as_secs_f64()) {
                Some(Ok(Event::FileLoaded)) => return true,
                Some(_) => continue,
                None => return false,
            }
        }

        false
    }
}

impl AudioBackend for MpvBackend {
    fn load_queue(&self, sources: &[String], start_index: usize) -> Result<(), String> {
        self.load_queue_with_state(sources, start_index, false, None)
    }

    fn load_queue_paused(
        &self,
        sources: &[String],
        start_index: usize,
        position_seconds: Option<f64>,
    ) -> Result<(), String> {
        self.load_queue_with_state(sources, start_index, true, position_seconds)
    }

    fn prepend_queue(&self, sources: &[String]) -> Result<(), String> {
        if sources.is_empty() {
            return Err("Cannot prepend an empty queue".to_string());
        }

        let queue_length = self
            .mpv
            .get_property::<i64>("playlist-count")
            .map_err(|error| format!("Failed to read playlist length: {error}"))?;
        if queue_length < 0 {
            return Err("Playlist length is invalid".to_string());
        }
        let active_index = self
            .mpv
            .get_property::<i64>("playlist-pos")
            .ok()
            .filter(|position| *position >= 0)
            .map(|position| position as usize);

        for source in sources {
            self.execute_command("loadfile", &[source, "append"], "append queue item")?;
        }
        for (target_index, _) in sources.iter().enumerate() {
            self.execute_command(
                "playlist-move",
                &[
                    &(queue_length as usize + target_index).to_string(),
                    &target_index.to_string(),
                ],
                "prepend queue item",
            )?;
        }
        if let Some(active_index) = active_index {
            self.set_property(
                "playlist-pos",
                (active_index + sources.len()) as i64,
                "restore active queue item after prepend",
            )?;
        }

        Ok(())
    }

    fn append_queue(&self, sources: &[String]) -> Result<(), String> {
        if sources.is_empty() {
            return Err("Cannot append an empty queue".to_string());
        }

        for source in sources {
            self.execute_command("loadfile", &[source, "append"], "append queue item")?;
        }
        Ok(())
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

    fn set_status_change_callback(&self, callback: AudioStatusChangeCallback) {
        match self.status_change_callback.lock() {
            Ok(mut status_change_callback) => {
                status_change_callback.replace(callback);
            }
            Err(_) => eprintln!("Failed to set mpv status change callback"),
        }
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
