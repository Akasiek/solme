use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use super::MpvBackend;
use crate::{
    audio::{backend::AudioBackend, playback::Audio, queue::PreparedQueue, PlaybackState},
    library::CachedSong,
};

struct SilentAudio(PathBuf);

impl SilentAudio {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("solme-queue-{}.wav", uuid::Uuid::new_v4()));
        let data_length = 8000_u32 * 90 * 2;
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"RIFF").unwrap();
        file.write_all(&(36 + data_length).to_le_bytes()).unwrap();
        file.write_all(b"WAVEfmt ").unwrap();
        file.write_all(&16_u32.to_le_bytes()).unwrap();
        file.write_all(&1_u16.to_le_bytes()).unwrap();
        file.write_all(&1_u16.to_le_bytes()).unwrap();
        file.write_all(&8000_u32.to_le_bytes()).unwrap();
        file.write_all(&16000_u32.to_le_bytes()).unwrap();
        file.write_all(&2_u16.to_le_bytes()).unwrap();
        file.write_all(&16_u16.to_le_bytes()).unwrap();
        file.write_all(b"data").unwrap();
        file.write_all(&data_length.to_le_bytes()).unwrap();
        file.write_all(&vec![0; usize::try_from(data_length).unwrap()])
            .unwrap();
        Self(path)
    }

    fn source(&self) -> String {
        self.0.to_str().unwrap().to_string()
    }
}

impl Drop for SilentAudio {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn song(title: &str) -> CachedSong {
    CachedSong {
        remote_id: "same-song".to_string(),
        album_id: "album".to_string(),
        artist_id: None,
        title: title.to_string(),
        artist_name: "Artist".to_string(),
        album_name: "Album".to_string(),
        artwork_path: None,
        track_number: None,
        disc_number: None,
        duration_seconds: 90,
        favorite: false,
        rating: None,
    }
}

fn player() -> (Arc<MpvBackend>, Audio) {
    let backend = Arc::new(MpvBackend::new().unwrap());
    backend
        .set_property("ao", "null", "disable audio output in tests")
        .unwrap();
    let playback = Audio::new(Arc::clone(&backend) as Arc<dyn AudioBackend>);
    (backend, playback)
}

#[test]
#[allow(clippy::float_cmp)]
fn native_playlist_owns_order_and_duplicate_identity_through_moves_and_restore() {
    let audio = SilentAudio::new();
    let (backend, playback) = player();
    let songs = vec![
        song("first occurrence"),
        song("second occurrence"),
        song("third occurrence"),
        song("fourth occurrence"),
    ];
    for (from, to) in [(0, 3), (3, 0), (2, 0), (2, 3)] {
        let generation = playback.begin_loading(songs.clone(), 2).unwrap();
        playback
            .load(
                generation,
                &PreparedQueue::new(songs.clone(), vec![audio.source(); 4]).unwrap(),
                2,
                Some(37.5),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while playback.queue_snapshot().unwrap().status.position_seconds != 37.5
            && Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(10));
        }
        let before = playback.queue_snapshot().unwrap();
        let active_id = before.entry_ids.get(2).unwrap().clone();
        let mut expected_songs = before.songs.clone();
        let entry = expected_songs.remove(from);
        expected_songs.insert(to, entry);
        let mut expected_ids = before.entry_ids.clone();
        let entry = expected_ids.remove(from);
        expected_ids.insert(to, entry);

        // Mutate MPV directly: application metadata must follow its IDs even
        // when no application queue mutation ran.
        backend.move_queue_entry(from, to).unwrap();
        let after = playback.queue_snapshot().unwrap();
        assert_eq!(after.songs, expected_songs);
        assert_eq!(after.entry_ids, expected_ids);
        assert_ne!(after.revision, before.revision);
        let active_index = after.status.queue_position.unwrap() - 1;
        assert_eq!(after.entry_ids.get(active_index), Some(&active_id));
        assert_eq!(after.status.current_song.unwrap().title, "third occurrence");
        assert_eq!(after.status.state, PlaybackState::Paused);
        assert_eq!(after.status.position_seconds, 37.5);
        let session = playback.session_snapshot().unwrap().unwrap();
        assert_eq!(session.queue, expected_songs);
        assert_eq!(session.active_index, active_index);
        assert_eq!(session.position_seconds, 37.5);
    }
    let session = playback.session_snapshot().unwrap().unwrap();
    let old_ids = playback.queue_snapshot().unwrap().entry_ids;
    let generation = playback
        .begin_loading(session.queue.clone(), session.active_index)
        .unwrap();
    playback
        .load(
            generation,
            &PreparedQueue::new(session.queue.clone(), vec![audio.source(); 4]).unwrap(),
            session.active_index,
            Some(session.position_seconds),
        )
        .unwrap();
    let restored = playback.queue_snapshot().unwrap();
    assert_eq!(restored.songs, session.queue);
    assert!(restored.entry_ids.iter().all(|id| !old_ids.contains(id)));
    playback.stop().unwrap();
    assert!(backend.playlist().unwrap().entries.is_empty());
    assert!(playback.queue_snapshot().unwrap().songs.is_empty());
    assert!(playback.session_snapshot().unwrap().is_none());
}

#[test]
fn native_insertion_preserves_selection_and_cancelled_loading_cannot_replace_queue() {
    let audio = SilentAudio::new();
    let (backend, playback) = player();
    let songs = vec![song("first"), song("second")];
    let generation = playback.begin_loading(songs.clone(), 1).unwrap();
    playback
        .load(
            generation,
            &PreparedQueue::new(songs, vec![audio.source(); 2]).unwrap(),
            1,
            Some(0.0),
        )
        .unwrap();
    let before = playback.queue_snapshot().unwrap();
    playback
        .enqueue(
            PreparedQueue::new(
                vec![song("next one"), song("next two")],
                vec![audio.source(); 2],
            )
            .unwrap(),
            true,
        )
        .unwrap();
    let after = playback.queue_snapshot().unwrap();
    assert_eq!(
        after
            .songs
            .iter()
            .map(|song| song.title.as_str())
            .collect::<Vec<_>>(),
        ["first", "second", "next one", "next two"]
    );
    assert_eq!(after.status.queue_position, Some(2));
    assert_eq!(after.entry_ids.get(1), before.entry_ids.get(1));
    assert_eq!(backend.playlist().unwrap().entries.len(), 4);
    let generation = playback.begin_loading(vec![song("cancelled")], 0).unwrap();
    assert!(playback.session_snapshot().unwrap().is_none());
    playback.stop().unwrap();
    assert!(playback
        .load(
            generation,
            &PreparedQueue::new(vec![song("cancelled")], vec![audio.source()]).unwrap(),
            0,
            None
        )
        .is_err());
    assert!(playback.queue_snapshot().unwrap().songs.is_empty());
}

#[test]
fn native_events_preserve_idle_queue_and_clear_it_when_playback_finishes() {
    use std::sync::Mutex;
    let audio = SilentAudio::new();
    let (backend, playback) = player();
    let weak = playback.downgrade();
    let errors = Arc::new(Mutex::new(Vec::new()));
    let callback_errors = Arc::clone(&errors);
    backend.set_status_change_callback(Arc::new(move |change| {
        if let Some(playback) = weak.upgrade() {
            if let Err(error) = playback.backend_changed(change) {
                callback_errors.lock().unwrap().push(error);
            }
        }
    }));
    playback
        .enqueue(
            PreparedQueue::new(vec![song("queued while idle")], vec![audio.source()]).unwrap(),
            false,
        )
        .unwrap();
    // Let the real event thread process the playlist invalidation.
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(playback.queue_snapshot().unwrap().songs.len(), 1);
    assert_eq!(
        playback.queue_snapshot().unwrap().status.state,
        PlaybackState::Stopped
    );

    let songs = vec![song("first"), song("last")];
    let generation = playback.begin_loading(songs.clone(), 1).unwrap();
    playback
        .load(
            generation,
            &PreparedQueue::new(songs, vec![audio.source(); 2]).unwrap(),
            1,
            Some(37.5),
        )
        .unwrap();
    // The end of the native playlist produces the same idle transition as EOF.
    backend.next().unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while !playback.queue_snapshot().unwrap().songs.is_empty() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(playback.queue_snapshot().unwrap().songs.is_empty());
    assert!(backend.playlist().unwrap().entries.is_empty());
    assert!(errors.lock().unwrap().is_empty());
}

#[test]
fn dropping_backend_releases_watcher_and_native_core() {
    let backend = MpvBackend::new().unwrap();
    let core = Arc::downgrade(&backend.mpv);
    drop(backend);

    let deadline = Instant::now() + Duration::from_secs(2);
    while core.strong_count() != 0 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(
        core.strong_count(),
        0,
        "status watcher retained the MPV core"
    );
}

#[test]
#[allow(clippy::float_cmp)]
fn safe_wrapper_preserves_volume_pause_and_seek_commands() {
    let audio = SilentAudio::new();
    let (backend, playback) = player();
    let songs = vec![song("commands")];
    let generation = playback.begin_loading(songs.clone(), 0).unwrap();
    playback
        .load(
            generation,
            &PreparedQueue::new(songs, vec![audio.source()]).unwrap(),
            0,
            Some(0.0),
        )
        .unwrap();

    playback.set_volume(42.0).unwrap();
    assert_eq!(backend.status().volume, 42.0);
    playback.resume().unwrap();
    assert!(!backend.status().paused);
    playback.pause().unwrap();
    assert!(backend.status().paused);
    playback.seek(12.5).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    // MPV's audio clock can differ by one output buffer after resume/pause.
    while (backend.status().position_seconds - 12.5).abs() > 0.25 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!((backend.status().position_seconds - 12.5).abs() <= 0.25);
}

#[test]
#[allow(clippy::float_cmp)]
fn removing_native_entries_preserves_duplicates_and_advances_current_song() {
    let audio = SilentAudio::new();
    let (backend, playback) = player();
    let songs = vec![song("duplicate one"), song("duplicate two"), song("next")];
    let generation = playback.begin_loading(songs.clone(), 1).unwrap();
    playback
        .load(
            generation,
            &PreparedQueue::new(songs, vec![audio.source(); 3]).unwrap(),
            1,
            Some(12.5),
        )
        .unwrap();
    let before = playback.queue_snapshot().unwrap();
    let removed = before.entry_ids.first().unwrap();
    playback.remove_entry(removed).unwrap();
    let after = playback.queue_snapshot().unwrap();
    assert_eq!(after.songs.len(), 2);
    assert_eq!(after.status.current_song.unwrap().title, "duplicate two");
    assert_eq!(after.entry_ids.first(), before.entry_ids.get(1));
    assert!(playback.remove_entry(removed).is_err());
    assert_eq!(backend.playlist().unwrap().entries.len(), 2);

    playback
        .remove_entry(after.entry_ids.first().unwrap())
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let snapshot = playback.queue_snapshot().unwrap();
        if snapshot
            .status
            .current_song
            .as_ref()
            .is_some_and(|song| song.title == "next")
        {
            assert_eq!(snapshot.songs.len(), 1);
            break;
        }
        assert!(
            Instant::now() < deadline,
            "MPV did not advance after removing current entry"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    let remaining = playback.queue_snapshot().unwrap().entry_ids.remove(0);
    playback.remove_entry(&remaining).unwrap();
    assert!(playback.queue_snapshot().unwrap().songs.is_empty());
    assert!(backend.playlist().unwrap().entries.is_empty());
}
