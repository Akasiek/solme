use std::collections::{HashMap, HashSet};

use super::{backend::AudioPlaylist, models::PlayerStatus};
use crate::library::CachedSong;

/// Metadata and temporary load intent. Native playlist order belongs to MPV;
/// playback commands and synchronization belong to Audio.
#[derive(Default)]
pub(super) struct PlayerQueue {
    metadata: HashMap<i64, CachedSong>,
    unbound: Option<(String, CachedSong)>,
    pending: Option<PendingLoad>,
    generation: u64,
}

pub(super) struct PendingLoad {
    pub songs: Vec<CachedSong>,
    pub start_index: usize,
    pub generation: u64,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueSnapshot {
    pub songs: Vec<CachedSong>,
    pub entry_ids: Vec<String>,
    pub revision: String,
    pub status: PlayerStatus,
}

impl PlayerQueue {
    pub fn pending(&self) -> Option<&PendingLoad> {
        self.pending.as_ref()
    }

    pub fn begin_loading(
        &mut self,
        songs: Vec<CachedSong>,
        start_index: usize,
    ) -> Result<u64, String> {
        if start_index >= songs.len() {
            return Err("Queue start index is out of bounds".to_string());
        }

        self.generation += 1;
        let generation = self.generation;

        self.pending = Some(PendingLoad {
            songs,
            start_index,
            generation,
        });

        Ok(generation)
    }

    pub fn cancel_loading(&mut self, generation: u64) {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.generation == generation)
        {
            self.finish_loading();
        }
    }

    pub fn finish_loading(&mut self) {
        self.pending = None;
    }

    pub fn expect_entry(&mut self, source: &str, song: CachedSong) {
        self.unbound = Some((source.to_string(), song));
    }

    pub fn resolve(&mut self, playlist: &AudioPlaylist) -> Result<Vec<CachedSong>, String> {
        for entry in &playlist.entries {
            if self.metadata.contains_key(&entry.id) {
                continue;
            }
            let song = match self.unbound.take() {
                Some((source, song)) if source == entry.source => song,
                candidate => {
                    self.unbound = candidate;
                    return Err(format!("Unrecognized playlist entry {}", entry.id));
                }
            };
            self.metadata.insert(entry.id, song);
        }

        let live = playlist
            .entries
            .iter()
            .map(|entry| entry.id)
            .collect::<HashSet<_>>();
        self.metadata.retain(|id, _| live.contains(id));

        self.unbound = None;
        playlist
            .entries
            .iter()
            .map(|entry| {
                self.metadata
                    .get(&entry.id)
                    .cloned()
                    .ok_or_else(|| format!("Missing metadata for playlist entry {}", entry.id))
            })
            .collect()
    }
}

pub(super) struct PreparedQueue {
    songs: Vec<CachedSong>,
    sources: Vec<String>,
}

impl PreparedQueue {
    pub fn new(songs: Vec<CachedSong>, sources: Vec<String>) -> Result<Self, String> {
        if songs.is_empty() {
            return Err("Queue cannot be empty".to_string());
        }
        if songs.len() != sources.len() {
            return Err("Queue metadata and sources are misaligned".to_string());
        }
        Ok(Self { songs, sources })
    }

    pub fn selected(&self, index: usize) -> Result<(&CachedSong, &str), String> {
        self.songs
            .get(index)
            .zip(self.sources.get(index))
            .map(|(song, source)| (song, source.as_str()))
            .ok_or_else(|| "Queue start index is out of bounds".to_string())
    }

    pub fn entries(&self) -> impl Iterator<Item = (&CachedSong, &str)> {
        self.songs
            .iter()
            .zip(&self.sources)
            .map(|(song, source)| (song, source.as_str()))
    }

    pub fn into_entries(self) -> impl Iterator<Item = (CachedSong, String)> {
        self.songs.into_iter().zip(self.sources)
    }
}
