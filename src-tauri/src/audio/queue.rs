use std::sync::{Arc, Mutex, MutexGuard};

use crate::library::CachedSong;

#[derive(Clone, Default)]
pub(super) struct PlayerQueue {
    songs: Arc<Mutex<Vec<CachedSong>>>,
}

impl PlayerQueue {
    fn lock(&self) -> Result<MutexGuard<'_, Vec<CachedSong>>, String> {
        self.songs
            .lock()
            .map_err(|_| "Player queue lock was poisoned".to_string())
    }

    pub fn snapshot(&self) -> Result<Vec<CachedSong>, String> {
        Ok(self.lock()?.clone())
    }

    pub fn current_and_len(
        &self,
        index: Option<usize>,
    ) -> Result<(Option<CachedSong>, usize), String> {
        let songs = self.lock()?;
        let current_song = index.and_then(|index| songs.get(index)).cloned();
        Ok((current_song, songs.len()))
    }

    pub fn len(&self) -> Result<usize, String> {
        Ok(self.lock()?.len())
    }

    pub fn replace(&self, songs: Vec<CachedSong>) -> Result<(), String> {
        *self.lock()? = songs;
        Ok(())
    }

    pub fn append(&self, songs: Vec<CachedSong>) -> Result<(), String> {
        self.lock()?.extend(songs);
        Ok(())
    }

    pub fn insert(&self, position: usize, songs: Vec<CachedSong>) -> Result<(), String> {
        let mut queue = self.lock()?;
        if position > queue.len() {
            return Err("Queue insertion position is out of bounds".to_string());
        }
        queue.splice(position..position, songs);
        Ok(())
    }

    pub fn clear(&self) -> Result<(), String> {
        self.lock()?.clear();
        Ok(())
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

    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    pub fn into_songs(self) -> Vec<CachedSong> {
        self.songs
    }
}

#[cfg(test)]
mod tests {
    use super::{PlayerQueue, PreparedQueue};
    use crate::library::CachedSong;

    #[test]
    fn rejects_misaligned_metadata_and_sources() {
        let error = PreparedQueue::new(
            vec![song("song-1"), song("song-2")],
            vec!["source-1".to_string()],
        )
        .err()
        .unwrap();

        assert_eq!(error, "Queue metadata and sources are misaligned");
    }

    #[test]
    fn keeps_current_song_and_length_in_one_snapshot() {
        let queue = PlayerQueue::default();
        queue.replace(vec![song("song-1"), song("song-2")]).unwrap();

        let (current_song, length) = queue.current_and_len(Some(1)).unwrap();

        assert_eq!(current_song.unwrap().remote_id, "song-2");
        assert_eq!(length, 2);
    }

    #[test]
    fn inserts_songs_at_requested_position() {
        let queue = PlayerQueue::default();
        queue.replace(vec![song("song-1"), song("song-4")]).unwrap();

        queue
            .insert(1, vec![song("song-2"), song("song-3")])
            .unwrap();

        assert_eq!(
            queue
                .snapshot()
                .unwrap()
                .iter()
                .map(|song| song.remote_id.as_str())
                .collect::<Vec<_>>(),
            ["song-1", "song-2", "song-3", "song-4"]
        );
    }

    fn song(id: &str) -> CachedSong {
        CachedSong {
            remote_id: id.to_string(),
            album_id: "album-1".to_string(),
            artist_id: Some("artist-1".to_string()),
            title: id.to_string(),
            artist_name: "Artist".to_string(),
            album_name: "Album".to_string(),
            artwork_path: None,
            track_number: Some(1),
            disc_number: Some(1),
            duration_seconds: 180,
        }
    }
}
