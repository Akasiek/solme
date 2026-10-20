use std::sync::{Arc, Mutex, MutexGuard, Weak};

use super::{
    backend::{AudioBackend, AudioPlaylist, AudioStatusChangeCallback},
    models::{PlaybackState, PlayerStatus},
    queue::{PlayerQueue, PreparedQueue, QueueSnapshot},
    session::PlaybackSession,
};
use crate::library::CachedSong;

const PREVIOUS_SONG_RESTART_THRESHOLD_SECONDS: f64 = 5.0;

struct AudioInner {
    backend: Arc<dyn AudioBackend>,
    queue: Mutex<PlayerQueue>,
    mutation: Mutex<()>,
}

/// Owns playback, its queue metadata, and command/event synchronization.
/// MPV remains the source of playlist order and active entry identity.
#[derive(Clone)]
pub(super) struct Audio {
    inner: Arc<AudioInner>,
}

pub(super) struct WeakAudio {
    inner: Weak<AudioInner>,
}

impl WeakAudio {
    pub fn upgrade(&self) -> Option<Audio> {
        Some(Audio {
            inner: self.inner.upgrade()?,
        })
    }
}

impl Audio {
    pub fn new(backend: Arc<dyn AudioBackend>) -> Self {
        Self {
            inner: Arc::new(AudioInner {
                backend,
                queue: Mutex::default(),
                mutation: Mutex::default(),
            }),
        }
    }

    pub fn downgrade(&self) -> WeakAudio {
        WeakAudio {
            inner: Arc::downgrade(&self.inner),
        }
    }

    pub fn set_status_change_callback(&self, callback: AudioStatusChangeCallback) {
        self.inner.backend.set_status_change_callback(callback);
    }

    fn queue(&self) -> Result<MutexGuard<'_, PlayerQueue>, String> {
        self.inner
            .queue
            .lock()
            .map_err(|_| "Player queue lock was poisoned".to_string())
    }

    pub fn status(&self) -> Result<PlayerStatus, String> {
        Ok(self.queue_snapshot()?.status)
    }

    fn mutation(&self) -> Result<MutexGuard<'_, ()>, String> {
        self.inner
            .mutation
            .lock()
            .map_err(|_| "Player command lock was poisoned".to_string())
    }

    pub fn queue_snapshot(&self) -> Result<QueueSnapshot, String> {
        let mut queue = self.queue()?;
        if let Some(pending) = queue.pending() {
            let status = PlayerStatus {
                state: PlaybackState::Loading,
                current_song: pending.songs.get(pending.start_index).cloned(),
                position_seconds: 0.0,
                duration_seconds: 0.0,
                queue_position: Some(pending.start_index + 1),
                queue_length: pending.songs.len(),
                volume: self.inner.backend.status().volume,
            };
            return Ok(QueueSnapshot {
                songs: pending.songs.clone(),
                entry_ids: (0..pending.songs.len())
                    .map(|index| format!("loading:{}:{index}", pending.generation))
                    .collect(),
                revision: format!("loading:{}", pending.generation),
                status,
            });
        }
        self.native_snapshot(&mut queue)
    }

    fn native_snapshot(&self, queue: &mut PlayerQueue) -> Result<QueueSnapshot, String> {
        let (playlist, audio) = self.read_backend()?;
        let songs = queue.resolve(&playlist)?;
        let entry_ids = playlist
            .entries
            .iter()
            .map(|entry| entry.id.to_string())
            .collect::<Vec<_>>();

        // Exact content token, derived from MPV. No independent order or revision
        // counter needs updating when the backend changes its playlist.
        let revision = format!("mpv:{}", entry_ids.join(","));
        let current_index = playlist.current_index();
        let state = if !audio.playing {
            PlaybackState::Stopped
        } else if audio.paused {
            PlaybackState::Paused
        } else {
            PlaybackState::Playing
        };

        let status = PlayerStatus {
            state,
            current_song: current_index.and_then(|index| songs.get(index)).cloned(),
            position_seconds: audio.position_seconds,
            duration_seconds: audio.duration_seconds,
            queue_position: current_index.map(|index| index + 1),
            queue_length: songs.len(),
            volume: audio.volume,
        };

        Ok(QueueSnapshot {
            songs,
            entry_ids,
            revision,
            status,
        })
    }

    fn read_backend(&self) -> Result<(AudioPlaylist, super::backend::AudioBackendStatus), String> {
        // MPV can advance independently of application locks. Confirm that the
        // position/duration read belongs to the same entry and playlist view.
        for _ in 0..3 {
            let playlist = self.inner.backend.playlist()?;
            let status = self.inner.backend.status();
            let confirmed = self.inner.backend.playlist()?;
            if playlist
                .entries
                .iter()
                .map(|entry| entry.id)
                .eq(confirmed.entries.iter().map(|entry| entry.id))
                && playlist.current_index() == confirmed.current_index()
            {
                return Ok((playlist, status));
            }
        }
        Err("The playlist changed while reading player queue. Please try again.".to_string())
    }

    pub fn begin_loading(&self, songs: Vec<CachedSong>, start_index: usize) -> Result<u64, String> {
        if start_index >= songs.len() {
            return Err("Queue start index is out of bounds".to_string());
        }
        let _mutation = self.mutation()?;
        let mut queue = self.queue()?;
        // Reconcile any earlier partial operation before replacing its candidates.
        self.native_snapshot(&mut queue)?;
        self.inner.backend.pause_immediately()?;
        queue.begin_loading(songs, start_index)
    }

    pub fn cancel_loading(&self, generation: u64) -> Result<(), String> {
        self.queue()?.cancel_loading(generation);
        Ok(())
    }

    pub fn load(
        &self,
        generation: u64,
        prepared: &PreparedQueue,
        start_index: usize,
        paused_position: Option<f64>,
    ) -> Result<(), String> {
        let _mutation = self.mutation()?;
        let (selected_song, selected_source) = prepared.selected(start_index)?;

        {
            let mut queue = self.queue()?;
            if queue
                .pending()
                .is_none_or(|pending| pending.generation != generation)
            {
                return Err("Queue loading was cancelled".to_string());
            }
            queue.expect_entry(selected_source, selected_song.clone());
        }

        // Open the selected entry first. Register each native ID before adding
        // another occurrence, so identical URLs can carry different metadata.
        // Loading preview reads remain available while this source opens.
        let result = self
            .inner
            .backend
            .load_source(selected_source, paused_position);
        let mut queue = self.queue()?;
        let result = (|| {
            let reconciled = self.native_snapshot(&mut queue);
            result.and(reconciled.map(|_| ()))?;
            for (index, (song, source)) in prepared.entries().enumerate() {
                if index == start_index {
                    continue;
                }
                self.add_entry(
                    &mut queue,
                    song.clone(),
                    source,
                    (index < start_index).then_some(index),
                )?;
            }
            Ok(())
        })();
        queue.finish_loading();
        result
    }

    fn add_entry(
        &self,
        queue: &mut PlayerQueue,
        song: CachedSong,
        source: &str,
        position: Option<usize>,
    ) -> Result<(), String> {
        queue.expect_entry(source, song);
        let result = if let Some(position) = position {
            self.inner.backend.insert_source(source, position)
        } else {
            self.inner.backend.append_source(source)
        };

        // A failed insert can still leave an appended entry in MPV. Register
        // it and expose the actual native order even when the command failed.
        let reconciled = self.native_snapshot(queue);
        result.and(reconciled.map(|_| ()))
    }

    pub fn enqueue(&self, prepared: PreparedQueue, next: bool) -> Result<(), String> {
        let _mutation = self.mutation()?;
        let mut queue = self.queue()?;
        if queue.pending().is_some() {
            return Err("The queue is still loading".to_string());
        }
        let snapshot = self.native_snapshot(&mut queue)?;
        let position = snapshot.status.queue_position.unwrap_or(0);
        for (offset, (song, source)) in prepared.into_entries().enumerate() {
            self.add_entry(&mut queue, song, &source, next.then_some(position + offset))?;
        }
        Ok(())
    }

    pub fn move_entry(
        &self,
        from: usize,
        to: usize,
        expected_revision: &str,
        expected_current_index: Option<usize>,
    ) -> Result<(), String> {
        let _mutation = self.mutation()?;
        let mut queue = self.queue()?;
        if queue.pending().is_some() {
            return Err("The queue is still loading".to_string());
        }
        let snapshot = self.native_snapshot(&mut queue)?;
        if snapshot.revision != expected_revision
            || snapshot.status.queue_position.map(|position| position - 1) != expected_current_index
        {
            return Err("The queue or current song changed. Please try again.".to_string());
        }
        if from >= snapshot.songs.len() || to >= snapshot.songs.len() {
            return Err("Queue position is out of bounds".to_string());
        }
        if from != to {
            self.inner.backend.move_queue_entry(from, to)?;
        }
        Ok(())
    }

    fn control(
        &self,
        command: impl FnOnce(&dyn AudioBackend) -> Result<(), String>,
    ) -> Result<(), String> {
        let _mutation = self.mutation()?;
        command(&*self.inner.backend)
    }

    pub fn pause(&self) -> Result<(), String> {
        self.control(|audio| audio.pause())
    }
    pub fn resume(&self) -> Result<(), String> {
        self.control(|audio| audio.resume())
    }
    pub fn next(&self) -> Result<(), String> {
        self.control(|audio| audio.next())
    }
    pub fn seek(&self, position: f64) -> Result<(), String> {
        self.control(|audio| audio.seek(position))
    }
    pub fn set_volume(&self, volume: f64) -> Result<(), String> {
        self.control(|audio| audio.set_volume(volume))
    }

    pub fn stop(&self) -> Result<(), String> {
        let _mutation = self.mutation()?;
        let mut queue = self.queue()?;
        self.inner.backend.stop()?;
        queue.finish_loading();
        self.native_snapshot(&mut queue)?;
        Ok(())
    }

    pub fn skip(&self, index: usize) -> Result<(), String> {
        let _mutation = self.mutation()?;
        let mut queue = self.queue()?;
        if queue.pending().is_some() {
            return Err("The queue is still loading".to_string());
        }
        if index >= self.native_snapshot(&mut queue)?.songs.len() {
            return Err("Queue position is out of bounds".to_string());
        }
        self.inner.backend.skip_to_queue_position(index)
    }

    pub fn previous(&self) -> Result<bool, String> {
        let _mutation = self.mutation()?;
        let mut queue = self.queue()?;
        let snapshot = self.native_snapshot(&mut queue)?;
        let restart = snapshot.status.queue_position == Some(1)
            || snapshot.status.position_seconds > PREVIOUS_SONG_RESTART_THRESHOLD_SECONDS;
        if restart {
            self.inner.backend.seek(0.0)?;
        } else {
            self.inner.backend.previous()?;
        }
        Ok(restart)
    }

    pub fn session_snapshot(&self) -> Result<Option<PlaybackSession>, String> {
        let snapshot = self.queue_snapshot()?;
        if snapshot.status.state == PlaybackState::Loading {
            return Ok(None);
        }
        Ok(snapshot
            .status
            .queue_position
            .map(|position| PlaybackSession {
                queue: snapshot.songs,
                active_index: position - 1,
                position_seconds: snapshot.status.position_seconds.max(0.0),
            }))
    }

    pub fn backend_changed(
        &self,
        change: super::backend::AudioStatusChange,
    ) -> Result<Option<QueueSnapshot>, String> {
        // This callback runs on the adapter's event thread, independently of
        // the native core. Serialize it with commands before reading their result.
        let _mutation = self.mutation()?;
        let mut queue = self.queue()?;
        if queue.pending().is_some() {
            return Ok(None);
        }
        let mut snapshot = self.native_snapshot(&mut queue)?;
        let finished_entry = match change {
            super::backend::AudioStatusChange::BecameIdle {
                last_entry_id: Some(id),
            } => Some(id.to_string()),
            _ => None,
        };
        if finished_entry.is_some_and(|id| snapshot.entry_ids.contains(&id))
            && snapshot.status.state == PlaybackState::Stopped
            && snapshot.status.queue_position.is_none()
        {
            self.inner.backend.stop()?;
            snapshot = self.native_snapshot(&mut queue)?;
        }
        Ok(Some(snapshot))
    }
}
