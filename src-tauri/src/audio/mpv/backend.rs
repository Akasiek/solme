use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use rsmpv::{Client, Event, Mpv, Node, SetProperty};

use super::{playlist::parse_playlist, status_watcher::StatusWatcher};
use crate::audio::backend::{
    AudioBackend, AudioBackendStatus, AudioPlaylist, AudioStatusChangeCallback,
};

const DEMUXER_READAHEAD_SECS: f64 = 30.0;
const DEMUXER_MAX_BYTES: i64 = 256 * 1024 * 1024;
const STREAM_BUFFER_SIZE: i64 = 4 * 1024 * 1024;

pub struct MpvBackend {
    mpv: Arc<Mpv>,
    _watcher: StatusWatcher,
    status_change_callback: Arc<Mutex<Option<AudioStatusChangeCallback>>>,
}

impl MpvBackend {
    pub fn new() -> Result<Self, String> {
        set_numeric_locale()?;
        let mpv = Arc::new(
            Mpv::new().map_err(|error| format!("Failed to create mpv instance: {error}"))?,
        );
        let status_change_callback = Arc::new(Mutex::new(None));
        let watcher = StatusWatcher::start(Arc::clone(&mpv), Arc::clone(&status_change_callback))?;
        let audio = Self {
            mpv,
            _watcher: watcher,
            status_change_callback,
        };

        audio.set_property("vo", "null", "disable video output")?;
        audio.set_property("idle", true, "enable mpv idle mode")?;
        audio.set_property("gapless-audio", "yes", "enable gapless audio")?;
        audio.set_property("cache", "yes", "enable mpv stream cache")?;
        audio.set_property("prefetch-playlist", true, "prefetch next playlist entry")?;
        audio.set_property(
            "stream-buffer-size",
            STREAM_BUFFER_SIZE,
            "increase stream buffer",
        )?;
        audio.set_property(
            "demuxer-readahead-secs",
            DEMUXER_READAHEAD_SECS,
            "increase audio readahead",
        )?;
        audio.set_property(
            "demuxer-max-bytes",
            DEMUXER_MAX_BYTES,
            "increase demuxer cache",
        )?;

        Ok(audio)
    }

    fn set_property<T: SetProperty>(
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
        let arguments = std::iter::once(command)
            .chain(arguments.iter().copied())
            .collect::<Vec<_>>();
        self.mpv
            .command(&arguments)
            .map_err(|error| format!("Failed to {action}: {error}"))
    }

    fn wait_for_file_loaded(
        client: &mut Client<'_>,
        expected_entry_id: i64,
        timeout: Duration,
    ) -> bool {
        let started_at = Instant::now();
        let mut loading_entry_id = None;
        while let Some(remaining) = timeout.checked_sub(started_at.elapsed()) {
            match client.wait_event(remaining.as_secs_f64()) {
                Some(Event::StartFile { playlist_entry_id }) => {
                    loading_entry_id = Some(playlist_entry_id);
                }
                Some(Event::FileLoaded) if loading_entry_id == Some(expected_entry_id) => {
                    return true;
                }
                Some(Event::Shutdown) => return false,
                _ => {}
            }
        }
        false
    }
}

impl AudioBackend for MpvBackend {
    fn playlist(&self) -> Result<AudioPlaylist, String> {
        let node = self
            .mpv
            .get_property::<Node>("playlist")
            .map_err(|error| format!("Failed to read MPV playlist: {error}"))?;
        parse_playlist(node)
    }

    fn load_source(&self, source: &str, paused_position: Option<f64>) -> Result<(), String> {
        // Subscribe before loadfile so START_FILE cannot be missed.
        let mut load_events = self
            .mpv
            .create_client(None)
            .map_err(|error| format!("Failed to create MPV loading client: {error}"))?;
        self.set_paused(true, "pause while loading queue entry")?;
        self.execute_command(
            "loadfile",
            &[source, "replace"],
            "load selected queue entry",
        )?;
        let entry_id = self
            .playlist()?
            .entries
            .first()
            .map(|entry| entry.id)
            .ok_or_else(|| "Selected queue entry is missing".to_string())?;
        self.set_paused(
            paused_position.is_some(),
            "set playback state after loading entry",
        )?;

        if !Self::wait_for_file_loaded(&mut load_events, entry_id, Duration::from_secs(5)) {
            return Err(
                "The selected queue entry could not be loaded within 5 seconds".to_string(),
            );
        }

        if let Some(position) = paused_position.filter(|position| *position > 0.0) {
            self.seek(position)?;
        }

        Ok(())
    }

    fn insert_source(&self, source: &str, position: usize) -> Result<(), String> {
        let length = self.playlist()?.entries.len();
        if position > length {
            return Err("Queue insertion position is out of bounds".to_string());
        }

        self.append_source(source)?;
        if position < length {
            self.execute_command(
                "playlist-move",
                &[&length.to_string(), &position.to_string()],
                "insert queue entry",
            )?;
        }

        Ok(())
    }

    fn append_source(&self, source: &str) -> Result<(), String> {
        self.execute_command("loadfile", &[source, "append"], "append queue entry")
    }

    fn move_queue_entry(&self, from: usize, to: usize) -> Result<(), String> {
        // MPV inserts before the destination entry in the original playlist.
        let destination = if from < to { to + 1 } else { to };
        self.execute_command(
            "playlist-move",
            &[&from.to_string(), &destination.to_string()],
            "move queue entry",
        )
    }

    fn pause_immediately(&self) -> Result<(), String> {
        self.set_paused(true, "pause playback immediately")
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

    fn skip_to_queue_position(&self, index: usize) -> Result<(), String> {
        let index = i64::try_from(index).map_err(|_| "Queue position is too large".to_string())?;
        self.set_property("playlist-pos", index, "skip to track")
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
        }
    }

    fn set_status_change_callback(&self, callback: AudioStatusChangeCallback) {
        match self.status_change_callback.lock() {
            Ok(mut status_change_callback) => {
                status_change_callback.replace(callback);
            }
            Err(_) => log::error!("Failed to set mpv status change callback"),
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
