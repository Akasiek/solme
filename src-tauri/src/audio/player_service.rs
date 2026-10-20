use crate::{
    library::{CachedSong, LibraryCatalogRepository},
    server::MusicServerService,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{
    backend::AudioBackend,
    fader::FadingAudioBackend,
    models::PlayerStatus,
    playback::{Audio, WeakAudio},
    preference::{PreferenceRepository, PreferenceService},
    queue::{PreparedQueue, QueueSnapshot},
    session::PlaybackSession,
};
use crate::events::EventBus;

type MusicServerBackend = dyn crate::server::backend::MusicServer;

pub struct PlayerService {
    server: Arc<MusicServerService>,
    repository: Arc<dyn LibraryCatalogRepository>,
    preference: PreferenceService,
    event_bus: Arc<EventBus>,
    audio: Audio,
    queue_load_lock: Mutex<()>,
}

impl PlayerService {
    pub fn new(
        audio: Box<dyn AudioBackend>,
        server: Arc<MusicServerService>,
        repository: Arc<dyn LibraryCatalogRepository>,
        preference: Arc<dyn PreferenceRepository>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        let backend: Arc<dyn AudioBackend> = Arc::new(FadingAudioBackend::new(audio.into()));
        let preference = PreferenceService::new(Arc::clone(&server), preference);
        let audio = Audio::new(backend);
        audio.set_status_change_callback(Self::status_change_callback(
            audio.downgrade(),
            Arc::clone(&event_bus),
        ));

        Self {
            server,
            repository,
            preference,
            event_bus,
            audio,
            queue_load_lock: Mutex::new(()),
        }
    }

    fn notify_queue_changed(&self) {
        if let Err(error) = self.event_bus.publish_player_queue_changed() {
            log::error!("Failed to emit player queue change: {error}");
        }
    }

    fn notify_status_changed(&self) -> Result<(), String> {
        let status = self.status()?;
        if let Err(error) = self.event_bus.publish_player_status(status) {
            log::error!("Failed to emit player status change: {error}");
        }
        Ok(())
    }

    fn notify_status_changed_with_position(&self, position_seconds: f64) -> Result<(), String> {
        let mut status = self.status()?;
        status.position_seconds = position_seconds.max(0.0);
        if let Err(error) = self.event_bus.publish_player_status(status) {
            log::error!("Failed to emit player status change: {error}");
        }
        Ok(())
    }

    fn finish_queue_change(&self, result: Result<(), String>) -> Result<(), String> {
        // A failed native command can still leave a partial playlist. Always
        // publish the actual view, including on the error path.
        self.notify_queue_changed();
        let status = self.notify_status_changed();
        result.and(status)
    }

    pub fn subscribe_status(&self) -> tokio::sync::broadcast::Receiver<PlayerStatus> {
        self.event_bus.subscribe_player_status()
    }

    pub async fn play_album(
        &self,
        album_id: &str,
        start_song_id: Option<&str>,
    ) -> Result<(), String> {
        let (server, songs) = self.album_songs(album_id).await?;
        let start_index = Self::album_start_index(&songs, start_song_id)?;
        self.play_songs(&server, songs, start_index).await
    }

    pub async fn play_albums(&self, album_ids: &[String]) -> Result<(), String> {
        let (profile_id, server) = self.server.current_server()?;
        let mut songs = Vec::new();
        for album_id in album_ids {
            songs.extend(self.repository.songs(&profile_id, album_id).await?);
        }
        if songs.is_empty() {
            return Err("No cached songs in selected albums".to_string());
        }
        self.play_songs(&server, songs, 0).await
    }

    async fn play_songs(
        &self,
        server: &Arc<MusicServerBackend>,
        songs: Vec<CachedSong>,
        start_index: usize,
    ) -> Result<(), String> {
        let _queue_load = self.queue_load_lock.lock().await;
        let generation = self.audio.begin_loading(songs.clone(), start_index)?;
        self.notify_queue_changed();
        self.notify_status_changed()?;
        let result = match Self::playback_sources(server, &songs).await {
            Ok(sources) => self.audio.load(
                generation,
                &PreparedQueue::new(songs, sources)?,
                start_index,
                None,
            ),
            Err(error) => {
                self.audio.cancel_loading(generation)?;
                Err(error)
            }
        };
        self.finish_queue_change(result)
    }

    async fn album_songs(
        &self,
        album_id: &str,
    ) -> Result<(Arc<MusicServerBackend>, Vec<CachedSong>), String> {
        let (profile_id, server) = self.server.current_server()?;
        let songs = self.repository.songs(&profile_id, album_id).await?;
        if songs.is_empty() {
            return Err("Album has no cached songs".to_string());
        }
        Ok((server, songs))
    }

    fn album_start_index(
        songs: &[CachedSong],
        start_song_id: Option<&str>,
    ) -> Result<usize, String> {
        match start_song_id {
            Some(song_id) => songs
                .iter()
                .position(|song| song.remote_id == song_id)
                .ok_or_else(|| "Selected song does not belong to this album".to_string()),
            None => Ok(0),
        }
    }

    pub async fn queue_album_next(&self, album_id: &str) -> Result<(), String> {
        let _queue_load = self.queue_load_lock.lock().await;
        let prepared = self.prepare_album_queue(album_id).await?;
        self.finish_queue_change(self.audio.enqueue(prepared, true))
    }

    pub async fn queue_album_last(&self, album_id: &str) -> Result<(), String> {
        let _queue_load = self.queue_load_lock.lock().await;
        let prepared = self.prepare_album_queue(album_id).await?;
        self.finish_queue_change(self.audio.enqueue(prepared, false))
    }

    async fn prepare_album_queue(&self, album_id: &str) -> Result<PreparedQueue, String> {
        let (profile_id, server) = self.server.current_server()?;
        let songs = self.repository.songs(&profile_id, album_id).await?;
        if songs.is_empty() {
            return Err("Album has no cached songs".to_string());
        }

        let sources = Self::playback_sources(&server, &songs).await?;
        PreparedQueue::new(songs, sources)
    }

    pub fn pause(&self) -> Result<(), String> {
        self.audio.pause()?;
        self.notify_status_changed()
    }

    pub fn resume(&self) -> Result<(), String> {
        self.audio.resume()?;
        self.notify_status_changed()
    }

    pub fn stop(&self) -> Result<(), String> {
        self.finish_queue_change(self.audio.stop())
    }

    pub fn next(&self) -> Result<(), String> {
        self.audio.next()?;
        self.notify_status_changed()
    }

    pub fn previous(&self) -> Result<(), String> {
        if self.audio.previous()? {
            self.notify_status_changed_with_position(0.0)
        } else {
            self.notify_status_changed()
        }
    }

    pub fn skip_to_queue_position(&self, queue_position: usize) -> Result<(), String> {
        self.audio.skip(queue_position)?;
        self.notify_status_changed()
    }

    pub async fn move_queue_entry(
        &self,
        from: usize,
        to: usize,
        expected_revision: &str,
        expected_current_index: Option<usize>,
    ) -> Result<(), String> {
        let _queue_load = self.queue_load_lock.lock().await;
        self.finish_queue_change(self.audio.move_entry(
            from,
            to,
            expected_revision,
            expected_current_index,
        ))
    }

    pub fn seek(&self, position_seconds: f64) -> Result<(), String> {
        self.audio.seek(position_seconds)?;
        self.notify_status_changed_with_position(position_seconds)
    }

    pub fn set_volume(&self, volume: f64) -> Result<(), String> {
        let volume = volume.clamp(0.0, 100.0);
        self.audio.set_volume(volume)?;
        self.preference.save_volume(volume);
        self.notify_status_changed()
    }

    pub fn status(&self) -> Result<PlayerStatus, String> {
        self.audio.status()
    }

    pub fn queue_snapshot(&self) -> Result<QueueSnapshot, String> {
        self.audio.queue_snapshot()
    }

    #[cfg(test)]
    pub fn queue(&self) -> Result<Vec<CachedSong>, String> {
        Ok(self.audio.queue_snapshot()?.songs)
    }

    pub(crate) fn session_snapshot(&self) -> Result<Option<PlaybackSession>, String> {
        self.audio.session_snapshot()
    }

    pub(crate) async fn restore_session(&self, session: PlaybackSession) -> Result<(), String> {
        let _queue_load = self.queue_load_lock.lock().await;
        if session.queue.is_empty() || session.active_index >= session.queue.len() {
            return Err("Stored playback session has an invalid queue".to_string());
        }
        let (_, server) = self.server.current_server()?;
        let sources = Self::playback_sources(&server, &session.queue).await?;
        let generation = self
            .audio
            .begin_loading(session.queue.clone(), session.active_index)?;
        let prepared = PreparedQueue::new(session.queue, sources)?;
        self.finish_queue_change(self.audio.load(
            generation,
            &prepared,
            session.active_index,
            Some(session.position_seconds),
        ))
    }

    async fn playback_sources(
        server: &Arc<MusicServerBackend>,
        songs: &[CachedSong],
    ) -> Result<Vec<String>, String> {
        let mut sources = Vec::with_capacity(songs.len());
        for song in songs {
            sources.push(server.playback_uri(&song.remote_id).await?);
        }
        Ok(sources)
    }

    pub async fn restore_preferences(&self) -> Result<(), String> {
        let Some(volume) = self.preference.load_volume().await? else {
            return Ok(());
        };

        self.audio.set_volume(volume)?;
        self.notify_status_changed()
    }

    fn status_change_callback(
        audio: WeakAudio,
        event_bus: Arc<EventBus>,
    ) -> super::backend::AudioStatusChangeCallback {
        Arc::new(move |change| {
            let Some(audio) = audio.upgrade() else {
                return;
            };
            match audio.backend_changed(change) {
                Ok(Some(snapshot)) => {
                    if let Err(error) = event_bus.publish_player_queue_changed() {
                        log::error!("Failed to emit player queue change: {error}");
                    }
                    if let Err(error) = event_bus.publish_player_status(snapshot.status) {
                        log::error!("Failed to emit player status change: {error}");
                    }
                }
                Ok(None) => {}
                Err(error) => log::error!("Failed to read player state change: {error}"),
            }
        })
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::unimplemented, clippy::unused_async)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{mpsc, Arc, Condvar, Mutex},
        time::Duration,
    };

    use async_trait::async_trait;
    use sqlx::SqlitePool;

    use super::PlayerService;
    use crate::events::EventBus;
    use crate::{
        audio::{
            backend::{AudioBackend, AudioBackendStatus, AudioPlaylist, AudioPlaylistEntry},
            models::PlaybackState,
            preference::{Preference, PreferenceKey, PreferenceRepository},
            session::PlaybackSession,
        },
        credentials::CredentialStore,
        library::{
            models::{
                Album, AlbumSort, AlbumWithSongs, Artist, BinaryArtwork, CachedAlbum, CachedArtist,
                CachedSong, Genre, LibraryItemAnnotation, LibraryItemKind, LibrarySummary,
            },
            LibraryCatalogRepository, LibraryStateRepository,
        },
        server::{backend::MusicServer, AlbumQuery, MusicServerService, ScrobbleEvent, ServerInfo},
    };

    #[test]
    fn plays_full_album_from_selected_song() {
        tauri::async_runtime::block_on(async {
            let server_service = test_server_service().await;

            let songs = vec![song("song-1"), song("song-2"), song("song-3")];
            let repository: Arc<dyn LibraryCatalogRepository> = Arc::new(MockRepository {
                songs: songs.clone(),
                ..Default::default()
            });
            let audio_state = Arc::new(Mutex::new(MockAudioState::default()));
            let player = PlayerService::new(
                Box::new(MockAudioBackend {
                    state: Arc::clone(&audio_state),
                }),
                server_service,
                repository,
                Arc::new(MockPreferenceRepository::default()),
                noop_event_bus(),
            );

            player.play_album("album-1", Some("song-2")).await.unwrap();

            let audio = audio_state.lock().unwrap();
            assert_eq!(audio.start_index, 1);
            assert_eq!(
                audio.sources,
                [
                    "https://music.example.com/song-1",
                    "https://music.example.com/song-2",
                    "https://music.example.com/song-3"
                ]
            );
            drop(audio);

            let status = player.status().unwrap();
            assert_eq!(status.state, PlaybackState::Playing);
            assert_eq!(
                status.current_song.map(|song| song.remote_id).as_deref(),
                Some("song-2")
            );
            assert_eq!(status.queue_position, Some(2));
            assert_eq!(status.queue_length, 3);

            let queue = player.queue().unwrap();
            assert_eq!(
                queue
                    .iter()
                    .map(|song| song.remote_id.as_str())
                    .collect::<Vec<_>>(),
                ["song-1", "song-2", "song-3"]
            );

            assert_eq!(
                player.skip_to_queue_position(3).unwrap_err(),
                "Queue position is out of bounds"
            );
            player.skip_to_queue_position(2).unwrap();
            let status = player.status().unwrap();
            assert_eq!(status.current_song.unwrap().remote_id, "song-3");
            assert_eq!(status.queue_position, Some(3));
        });
    }

    #[test]
    fn plays_albums_in_requested_order_and_skips_empty_albums() {
        tauri::async_runtime::block_on(async {
            let server_service = test_server_service().await;
            let mut newest_song = song("newest-song");
            newest_song.album_id = "newest".to_string();
            let mut oldest_song = song("oldest-song");
            oldest_song.album_id = "oldest".to_string();
            let repository: Arc<dyn LibraryCatalogRepository> = Arc::new(MockRepository {
                songs_by_album: Some(HashMap::from([
                    ("newest".to_string(), vec![newest_song]),
                    ("oldest".to_string(), vec![oldest_song]),
                ])),
                ..Default::default()
            });
            let audio_state = Arc::new(Mutex::new(MockAudioState::default()));
            let player = PlayerService::new(
                Box::new(MockAudioBackend {
                    state: Arc::clone(&audio_state),
                }),
                server_service,
                repository,
                Arc::new(MockPreferenceRepository::default()),
                noop_event_bus(),
            );

            player.play_albums(&["oldest".to_string()]).await.unwrap();
            player
                .play_albums(&[
                    "empty".to_string(),
                    "newest".to_string(),
                    "oldest".to_string(),
                ])
                .await
                .unwrap();

            let queue = player.queue().unwrap();
            assert_eq!(
                queue
                    .iter()
                    .map(|song| song.remote_id.as_str())
                    .collect::<Vec<_>>(),
                ["newest-song", "oldest-song"]
            );
            assert_eq!(player.status().unwrap().queue_position, Some(1));
            assert_eq!(
                audio_state.lock().unwrap().sources,
                [
                    "https://music.example.com/newest-song",
                    "https://music.example.com/oldest-song"
                ]
            );

            assert_eq!(
                player.play_albums(&[]).await.unwrap_err(),
                "No cached songs in selected albums"
            );
            assert_eq!(player.queue().unwrap(), queue);
        });
    }

    #[test]
    fn exposes_queue_while_first_audio_source_is_loading() {
        let server_service = tauri::async_runtime::block_on(async { test_server_service().await });
        let songs = vec![song("song-1"), song("song-2"), song("song-3")];
        let repository: Arc<dyn LibraryCatalogRepository> = Arc::new(MockRepository {
            songs: songs.clone(),
            ..Default::default()
        });
        let (load_started_tx, load_started_rx) = mpsc::sync_channel(0);
        let load_release = Arc::new((Mutex::new(false), Condvar::new()));
        let audio_state = Arc::new(Mutex::new(MockAudioState {
            playing: true,
            load_started: Some(load_started_tx),
            load_release: Some(Arc::clone(&load_release)),
            ..Default::default()
        }));
        let player = Arc::new(PlayerService::new(
            Box::new(MockAudioBackend {
                state: Arc::clone(&audio_state),
            }),
            server_service,
            repository,
            Arc::new(MockPreferenceRepository::default()),
            noop_event_bus(),
        ));
        let mut status_receiver = player.subscribe_status();

        let playing_player = Arc::clone(&player);
        let play_thread = std::thread::spawn(move || {
            tauri::async_runtime::block_on(async {
                playing_player.play_album("album-1", None).await
            })
        });

        load_started_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("audio loading did not start");

        assert_eq!(player.queue().unwrap(), songs);
        let loading_status = status_receiver.try_recv().unwrap();
        assert_eq!(loading_status.state, PlaybackState::Loading);
        assert_eq!(loading_status.current_song.unwrap().remote_id, "song-1");
        assert_eq!(loading_status.queue_position, Some(1));
        assert!(audio_state.lock().unwrap().paused);

        let (released, release_signal) = &*load_release;
        *released.lock().unwrap() = true;
        release_signal.notify_one();

        play_thread.join().unwrap().unwrap();
    }

    #[test]
    fn serializes_single_and_multiple_album_playback() {
        let server_service = tauri::async_runtime::block_on(async { test_server_service().await });
        let first_song = song("first-song");
        let second_song = song("second-song");
        let repository: Arc<dyn LibraryCatalogRepository> = Arc::new(MockRepository {
            songs_by_album: Some(HashMap::from([
                ("first".to_string(), vec![first_song.clone()]),
                ("second".to_string(), vec![second_song.clone()]),
            ])),
            ..Default::default()
        });
        let (load_started_tx, load_started_rx) = mpsc::sync_channel(0);
        let load_release = Arc::new((Mutex::new(false), Condvar::new()));
        let audio_state = Arc::new(Mutex::new(MockAudioState {
            load_started: Some(load_started_tx),
            load_release: Some(Arc::clone(&load_release)),
            ..Default::default()
        }));
        let player = Arc::new(PlayerService::new(
            Box::new(MockAudioBackend {
                state: Arc::clone(&audio_state),
            }),
            server_service,
            repository,
            Arc::new(MockPreferenceRepository::default()),
            noop_event_bus(),
        ));

        let first_player = Arc::clone(&player);
        let first_thread = std::thread::spawn(move || {
            tauri::async_runtime::block_on(async { first_player.play_album("first", None).await })
        });
        load_started_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("first audio load did not start");
        let lock_held_during_load = player.queue_load_lock.try_lock().is_err();

        let (second_started_tx, second_started_rx) = mpsc::sync_channel(0);
        let second_player = Arc::clone(&player);
        let second_thread = std::thread::spawn(move || {
            second_started_tx.send(()).unwrap();
            tauri::async_runtime::block_on(async {
                second_player.play_albums(&["second".to_string()]).await
            })
        });
        second_started_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("second playback request did not start");
        std::thread::sleep(Duration::from_millis(100));
        let queue_during_first_load = player.queue().unwrap();
        let sources_during_first_load = audio_state.lock().unwrap().sources.clone();

        let (released, release_signal) = &*load_release;
        *released.lock().unwrap() = true;
        release_signal.notify_all();
        first_thread.join().unwrap().unwrap();
        second_thread.join().unwrap().unwrap();

        assert!(lock_held_during_load);
        assert_eq!(queue_during_first_load, vec![first_song]);
        assert_eq!(
            sources_during_first_load,
            ["https://music.example.com/first-song"]
        );
        assert_eq!(player.queue().unwrap(), vec![second_song]);
        assert_eq!(
            audio_state.lock().unwrap().sources,
            ["https://music.example.com/second-song"]
        );
    }

    #[test]
    fn rejects_song_outside_selected_album() {
        tauri::async_runtime::block_on(async {
            let server_service = test_server_service().await;
            let repository: Arc<dyn LibraryCatalogRepository> = Arc::new(MockRepository {
                songs: vec![song("song-1")],
                ..Default::default()
            });
            let player = PlayerService::new(
                Box::new(MockAudioBackend {
                    state: Arc::new(Mutex::new(MockAudioState::default())),
                }),
                server_service,
                repository,
                Arc::new(MockPreferenceRepository::default()),
                noop_event_bus(),
            );

            assert_eq!(
                player
                    .play_album("album-1", Some("other-song"))
                    .await
                    .unwrap_err(),
                "Selected song does not belong to this album"
            );
        });
    }

    #[test]
    fn appends_album_to_current_queue() {
        tauri::async_runtime::block_on(async {
            let server_service = test_server_service().await;

            let songs = vec![song("song-1"), song("song-2")];
            let repository: Arc<dyn LibraryCatalogRepository> = Arc::new(MockRepository {
                songs: songs.clone(),
                ..Default::default()
            });
            let audio_state = Arc::new(Mutex::new(MockAudioState::default()));
            let player = PlayerService::new(
                Box::new(MockAudioBackend {
                    state: Arc::clone(&audio_state),
                }),
                server_service,
                repository,
                Arc::new(MockPreferenceRepository::default()),
                noop_event_bus(),
            );

            player.play_album("album-1", Some("song-2")).await.unwrap();
            player.queue_album_last("album-2").await.unwrap();

            let audio = audio_state.lock().unwrap();
            assert_eq!(audio.start_index, 1);
            assert_eq!(
                audio.appended_sources,
                [
                    "https://music.example.com/song-1",
                    "https://music.example.com/song-2"
                ]
            );
            drop(audio);

            let status = player.status().unwrap();
            assert_eq!(status.current_song.unwrap().remote_id, "song-2");
            assert_eq!(status.queue_position, Some(2));
            assert_eq!(status.queue_length, 4);
        });
    }

    #[test]
    fn queues_album_after_current_song() {
        tauri::async_runtime::block_on(async {
            let server_service = test_server_service().await;

            let songs = vec![song("song-1"), song("song-2")];
            let repository: Arc<dyn LibraryCatalogRepository> = Arc::new(MockRepository {
                songs: songs.clone(),
                ..Default::default()
            });
            let audio_state = Arc::new(Mutex::new(MockAudioState::default()));
            let player = PlayerService::new(
                Box::new(MockAudioBackend {
                    state: Arc::clone(&audio_state),
                }),
                server_service,
                repository,
                Arc::new(MockPreferenceRepository::default()),
                noop_event_bus(),
            );

            player.play_album("album-1", Some("song-1")).await.unwrap();
            player.queue_album_next("album-2").await.unwrap();

            let audio = audio_state.lock().unwrap();
            assert_eq!(audio.start_index, 0);
            assert_eq!(audio.insertion_position, Some(1));
            assert_eq!(
                audio.inserted_sources,
                [
                    "https://music.example.com/song-1",
                    "https://music.example.com/song-2"
                ]
            );
            drop(audio);

            let status = player.status().unwrap();
            assert_eq!(status.current_song.unwrap().remote_id, "song-1");
            assert_eq!(status.queue_position, Some(1));
            assert_eq!(status.queue_length, 4);
        });
    }

    #[test]
    fn restores_paused_playback_session() {
        tauri::async_runtime::block_on(async {
            let server_service = test_server_service().await;
            let repository: Arc<dyn LibraryCatalogRepository> = Arc::new(MockRepository::default());
            let audio_state = Arc::new(Mutex::new(MockAudioState::default()));
            let player = PlayerService::new(
                Box::new(MockAudioBackend {
                    state: Arc::clone(&audio_state),
                }),
                server_service,
                repository,
                Arc::new(MockPreferenceRepository::default()),
                noop_event_bus(),
            );
            let queue = vec![song("song-1"), song("song-2"), song("song-3")];

            player
                .restore_session(PlaybackSession {
                    queue,
                    active_index: 1,
                    position_seconds: 37.5,
                })
                .await
                .unwrap();

            let audio = audio_state.lock().unwrap();
            assert_eq!(audio.start_index, 1);
            assert_eq!(audio.position_seconds, 37.5);
            assert!(audio.paused);
            assert_eq!(
                audio.sources,
                [
                    "https://music.example.com/song-1",
                    "https://music.example.com/song-2",
                    "https://music.example.com/song-3"
                ]
            );
            drop(audio);

            let status = player.status().unwrap();
            assert_eq!(status.state, PlaybackState::Paused);
            assert_eq!(status.current_song.unwrap().remote_id, "song-2");
            assert_eq!(status.queue_length, 3);
        });
    }

    #[test]
    fn moves_queue_entries_preserving_active_occurrence_progress_and_session() {
        tauri::async_runtime::block_on(async {
            for paused in [true, false] {
                for (from, to, expected_active) in [
                    (2, 0, 0),
                    (2, 3, 3),
                    (3, 0, 3),
                    (0, 3, 1),
                    (0, 1, 2),
                    (2, 2, 2),
                ] {
                    let (player, audio_state) = queue_move_player().await;
                    if !paused {
                        player.resume().unwrap();
                    }
                    let snapshot = player.queue_snapshot().unwrap();
                    let mut expected = snapshot.songs.clone();
                    let entry = expected.remove(from);
                    expected.insert(to, entry);

                    player
                        .move_queue_entry(from, to, &snapshot.revision, Some(2))
                        .await
                        .unwrap();

                    let status = player.status().unwrap();
                    assert_eq!(status.queue_position, Some(expected_active + 1));
                    assert_eq!(status.current_song.unwrap().remote_id, "song-1");
                    assert_eq!(status.position_seconds, 37.5);
                    assert_eq!(
                        status.state,
                        if paused {
                            PlaybackState::Paused
                        } else {
                            PlaybackState::Playing
                        }
                    );
                    let session = player.session_snapshot().unwrap().unwrap();
                    assert_eq!(session.queue, expected);
                    assert_eq!(session.active_index, expected_active);
                    assert_eq!(session.position_seconds, 37.5);
                    let audio = audio_state.lock().unwrap();
                    assert_eq!(audio.start_index, expected_active);
                    assert_eq!(
                        audio.sources,
                        expected
                            .iter()
                            .map(|song| format!("https://music.example.com/{}", song.remote_id))
                            .collect::<Vec<_>>()
                    );
                }
            }
        });
    }

    #[test]
    fn rejects_stale_or_invalid_queue_moves_and_leaves_failed_moves_unchanged() {
        tauri::async_runtime::block_on(async {
            for reason in ["revision", "current", "source", "target", "backend"] {
                let (player, audio_state) = queue_move_player().await;
                let snapshot = player.queue_snapshot().unwrap();
                let (mut from, mut to) = (0, 3);
                match reason {
                    "revision" => player
                        .move_queue_entry(0, 1, &snapshot.revision, Some(2))
                        .await
                        .unwrap(),
                    "current" => audio_state.lock().unwrap().start_index = 1,
                    "source" => from = 4,
                    "target" => to = 4,
                    "backend" => {
                        audio_state.lock().unwrap().move_error = Some("Move failed".to_string());
                    }
                    _ => unreachable!(),
                }
                let before = player.queue_snapshot().unwrap();
                let revision = before.revision;
                let sources = audio_state.lock().unwrap().sources.clone();
                assert!(
                    player
                        .move_queue_entry(from, to, &snapshot.revision, Some(2))
                        .await
                        .is_err(),
                    "{reason}"
                );
                assert_eq!(player.queue().unwrap(), before.songs, "{reason}");
                assert_eq!(
                    player.queue_snapshot().unwrap().revision,
                    revision,
                    "{reason}"
                );
                assert_eq!(audio_state.lock().unwrap().sources, sources, "{reason}");
            }
        });
    }

    async fn queue_move_player() -> (PlayerService, Arc<Mutex<MockAudioState>>) {
        let audio_state = Arc::new(Mutex::new(MockAudioState::default()));
        let player = PlayerService::new(
            Box::new(MockAudioBackend {
                state: Arc::clone(&audio_state),
            }),
            test_server_service().await,
            Arc::new(MockRepository {
                songs: vec![song("new-1"), song("new-2"), song("new-3")],
                ..Default::default()
            }),
            Arc::new(MockPreferenceRepository::default()),
            noop_event_bus(),
        );
        player
            .restore_session(PlaybackSession {
                queue: vec![
                    song("song-1"),
                    song("song-2"),
                    song("song-1"),
                    song("song-3"),
                ],
                active_index: 2,
                position_seconds: 37.5,
            })
            .await
            .unwrap();
        (player, audio_state)
    }

    #[test]
    fn publishes_actual_partial_playlist_when_appending_or_inserting_fails() {
        tauri::async_runtime::block_on(async {
            for insert in [true, false] {
                let (player, state) = queue_move_player().await;
                let before = player.queue_snapshot().unwrap();
                {
                    let mut audio = state.lock().unwrap();
                    if insert {
                        audio.insert_error_after_append =
                            Some("Insert failed after append".to_string());
                    } else {
                        audio.append_failure_after = Some(audio.appended_sources.len() + 1);
                    }
                }
                let songs = [song("new-1"), song("new-2"), song("new-3")];
                let mut statuses = player.subscribe_status();
                let result = if insert {
                    player.queue_album_next("new-album").await
                } else {
                    player.queue_album_last("new-album").await
                };
                assert!(result.is_err());
                let snapshot = player.queue_snapshot().unwrap();
                let mut expected = before.songs;
                // The failed insert left its accepted entry at the end in MPV.
                expected.push(songs.first().unwrap().clone());
                assert_eq!(snapshot.songs, expected);
                assert_ne!(snapshot.revision, before.revision);
                assert_eq!(snapshot.status.current_song.unwrap().remote_id, "song-1");
                assert_eq!(statuses.recv().await.unwrap().queue_length, 5);
                assert_eq!(player.session_snapshot().unwrap().unwrap().queue, expected);
            }
        });
    }

    #[test]
    fn retries_metadata_binding_after_a_native_playlist_read_fails() {
        tauri::async_runtime::block_on(async {
            let (player, state) = queue_move_player().await;
            {
                let mut state = state.lock().unwrap();
                state.read_failure_after_append = Some(state.appended_sources.len() + 1);
            }
            assert!(player.queue_album_last("new-album").await.is_err());
            let snapshot = player.queue_snapshot().unwrap();
            assert_eq!(snapshot.songs.last().unwrap().remote_id, "new-1");
            assert_eq!(snapshot.songs.len(), 5);
            assert_eq!(snapshot.entry_ids.len(), 5);
        });
    }

    #[test]
    fn reflects_the_native_prefix_after_queue_replacement_fails() {
        tauri::async_runtime::block_on(async {
            let (player, state) = queue_move_player().await;
            let previous_ids = player.queue_snapshot().unwrap().entry_ids;
            {
                let mut state = state.lock().unwrap();
                state.append_failure_after = Some(state.appended_sources.len() + 1);
            }
            let songs = [song("new-1"), song("new-2"), song("new-3")];
            assert!(player.play_album("new-album", None).await.is_err());
            let snapshot = player.queue_snapshot().unwrap();
            assert_eq!(
                snapshot.songs,
                songs.into_iter().take(2).collect::<Vec<_>>()
            );
            assert!(snapshot
                .entry_ids
                .iter()
                .all(|id| !previous_ids.contains(id)));
            assert_ne!(snapshot.status.state, PlaybackState::Loading);
            assert_eq!(
                player.session_snapshot().unwrap().unwrap().queue,
                snapshot.songs
            );
        });
    }

    #[test]
    fn restores_saved_volume_preference() {
        tauri::async_runtime::block_on(async {
            let server_service = test_server_service().await;
            let repository: Arc<dyn LibraryCatalogRepository> = Arc::new(MockRepository::default());
            let preferences = Arc::new(MockPreferenceRepository {
                preference: Mutex::new(Some(Preference::volume(37.0))),
            });
            let audio_state = Arc::new(Mutex::new(MockAudioState::default()));
            let player = PlayerService::new(
                Box::new(MockAudioBackend {
                    state: Arc::clone(&audio_state),
                }),
                server_service,
                repository,
                preferences,
                noop_event_bus(),
            );

            player.restore_preferences().await.unwrap();

            assert_eq!(audio_state.lock().unwrap().volume, 37.0);
            assert_eq!(player.status().unwrap().volume, 37.0);
        });
    }

    #[test]
    fn seek_event_reports_requested_position_when_backend_status_is_stale() {
        tauri::async_runtime::block_on(async {
            let server_service = test_server_service().await;
            let repository: Arc<dyn LibraryCatalogRepository> = Arc::new(MockRepository::default());
            let audio_state = Arc::new(Mutex::new(MockAudioState {
                playing: true,
                position_seconds: 37.5,
                keep_stale_position_after_seek: true,
                ..Default::default()
            }));
            let player = PlayerService::new(
                Box::new(MockAudioBackend {
                    state: Arc::clone(&audio_state),
                }),
                server_service,
                repository,
                Arc::new(MockPreferenceRepository::default()),
                noop_event_bus(),
            );
            let mut status_receiver = player.subscribe_status();

            player.seek(0.0).unwrap();

            let status = status_receiver.recv().await.unwrap();
            assert_eq!(status.position_seconds, 0.0);
            assert_eq!(audio_state.lock().unwrap().position_seconds, 37.5);
        });
    }

    #[test]
    fn previous_restarts_current_song_after_threshold() {
        tauri::async_runtime::block_on(async {
            let (player, audio_state) = queue_move_player().await;
            audio_state.lock().unwrap().position_seconds = 6.0;
            player.previous().unwrap();
            let audio = audio_state.lock().unwrap();
            assert_eq!(audio.position_seconds, 0.0);
            assert_eq!(audio.previous_calls, 0);
        });
    }

    #[test]
    fn previous_moves_to_previous_song_before_threshold() {
        tauri::async_runtime::block_on(async {
            let (player, audio_state) = queue_move_player().await;
            audio_state.lock().unwrap().position_seconds = 5.0;
            player.previous().unwrap();
            let audio = audio_state.lock().unwrap();
            assert_eq!(audio.position_seconds, 5.0);
            assert_eq!(audio.previous_calls, 1);
        });
    }

    #[test]
    fn previous_restarts_first_song_before_threshold() {
        tauri::async_runtime::block_on(async {
            let (player, audio_state) = queue_move_player().await;
            {
                let mut audio = audio_state.lock().unwrap();
                audio.start_index = 0;
                audio.position_seconds = 2.0;
            }
            player.previous().unwrap();
            let audio = audio_state.lock().unwrap();
            assert_eq!(audio.position_seconds, 0.0);
            assert_eq!(audio.previous_calls, 0);
        });
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
            favorite: false,
            rating: None,
        }
    }

    async fn test_server_service() -> Arc<MusicServerService> {
        let database = SqlitePool::connect_lazy("sqlite::memory:").unwrap();
        let server_service = Arc::new(MusicServerService::new(
            database,
            Box::new(MemoryCredentialStore),
        ));
        server_service
            .set_current_server("profile".to_string(), Arc::new(MockMusicServer))
            .unwrap();
        server_service
    }

    fn noop_event_bus() -> Arc<EventBus> {
        Arc::new(EventBus::disabled())
    }

    #[derive(Default)]
    struct MockAudioState {
        sources: Vec<String>,
        ids: Vec<i64>,
        next_id: i64,
        append_failure_after: Option<usize>,
        read_failure_after_append: Option<usize>,
        playlist_read_failures: usize,
        insert_error_after_append: Option<String>,
        inserted_sources: Vec<String>,
        insertion_position: Option<usize>,
        appended_sources: Vec<String>,
        start_index: usize,
        playing: bool,
        paused: bool,
        position_seconds: f64,
        volume: f64,
        previous_calls: usize,
        keep_stale_position_after_seek: bool,
        move_error: Option<String>,
        load_started: Option<mpsc::SyncSender<()>>,
        load_release: Option<Arc<(Mutex<bool>, Condvar)>>,
    }

    struct MockAudioBackend {
        state: Arc<Mutex<MockAudioState>>,
    }

    impl AudioBackend for MockAudioBackend {
        fn playlist(&self) -> Result<AudioPlaylist, String> {
            let mut state = self.state.lock().unwrap();
            if state.playlist_read_failures > 0 {
                state.playlist_read_failures -= 1;
                return Err("Playlist read failed".to_string());
            }
            Ok(AudioPlaylist {
                entries: state
                    .sources
                    .iter()
                    .zip(&state.ids)
                    .enumerate()
                    .map(|(index, (source, id))| AudioPlaylistEntry {
                        id: *id,
                        source: source.clone(),
                        current: state.playing && index == state.start_index,
                        playing: state.playing && index == state.start_index,
                    })
                    .collect(),
            })
        }
        fn set_status_change_callback(
            &self,
            _callback: crate::audio::backend::AudioStatusChangeCallback,
        ) {
        }

        fn load_source(&self, source: &str, paused_position: Option<f64>) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            state.sources = vec![source.to_string()];
            state.next_id += 1;
            state.ids = vec![state.next_id];
            state.start_index = 0;
            let load_started = state.load_started.take();
            let load_release = state.load_release.clone();
            drop(state);
            if let Some(load_started) = load_started {
                load_started.send(()).unwrap();
            }
            if let Some(load_release) = load_release {
                let (released, release_signal) = &*load_release;
                let mut released = released.lock().unwrap();
                while !*released {
                    released = release_signal.wait(released).unwrap();
                }
            }
            let mut state = self.state.lock().unwrap();
            state.playing = true;
            state.paused = paused_position.is_some();
            state.position_seconds = paused_position.unwrap_or(0.0);
            Ok(())
        }

        fn append_source(&self, source: &str) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            if state.append_failure_after == Some(state.appended_sources.len()) {
                return Err("Append failed".to_string());
            }
            state.appended_sources.push(source.to_string());
            if state.read_failure_after_append == Some(state.appended_sources.len()) {
                state.playlist_read_failures = 1;
            }
            state.next_id += 1;
            let id = state.next_id;
            state.sources.push(source.to_string());
            state.ids.push(id);
            Ok(())
        }

        fn insert_source(&self, source: &str, position: usize) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            state.inserted_sources.push(source.to_string());
            state.insertion_position.get_or_insert(position);
            state.next_id += 1;
            let id = state.next_id;
            if let Some(error) = state.insert_error_after_append.clone() {
                state.sources.push(source.to_string());
                state.ids.push(id);
                return Err(error);
            }
            state.sources.insert(position, source.to_string());
            state.ids.insert(position, id);
            if position <= state.start_index {
                state.start_index += 1;
            }
            Ok(())
        }

        fn move_queue_entry(&self, from: usize, to: usize) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            if let Some(error) = &state.move_error {
                return Err(error.clone());
            }
            let source = state.sources.remove(from);
            state.sources.insert(to, source);
            let id = state.ids.remove(from);
            state.ids.insert(to, id);
            let current = state.start_index;
            state.start_index = if current == from {
                to
            } else if from < current && current <= to {
                current - 1
            } else if to <= current && current < from {
                current + 1
            } else {
                current
            };
            Ok(())
        }

        fn pause_immediately(&self) -> Result<(), String> {
            self.state.lock().unwrap().paused = true;
            Ok(())
        }

        fn pause(&self) -> Result<(), String> {
            self.state.lock().unwrap().paused = true;
            Ok(())
        }

        fn resume(&self) -> Result<(), String> {
            self.state.lock().unwrap().paused = false;
            Ok(())
        }

        fn stop(&self) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            state.playing = false;
            state.sources.clear();
            state.ids.clear();
            Ok(())
        }

        fn next(&self) -> Result<(), String> {
            Ok(())
        }

        fn previous(&self) -> Result<(), String> {
            self.state.lock().unwrap().previous_calls += 1;
            Ok(())
        }

        fn skip_to_queue_position(&self, index: usize) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            state.start_index = index;
            state.position_seconds = 0.0;
            Ok(())
        }

        fn seek(&self, position_seconds: f64) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            if !state.keep_stale_position_after_seek {
                state.position_seconds = position_seconds;
            }
            Ok(())
        }

        fn set_volume(&self, volume: f64) -> Result<(), String> {
            self.state.lock().unwrap().volume = volume;
            Ok(())
        }

        fn status(&self) -> AudioBackendStatus {
            let state = self.state.lock().unwrap();
            AudioBackendStatus {
                playing: state.playing,
                paused: state.paused,
                position_seconds: state.position_seconds,
                duration_seconds: 180.0,
                volume: state.volume,
            }
        }
    }

    #[derive(Default)]
    struct MockPreferenceRepository {
        preference: Mutex<Option<Preference>>,
    }

    #[async_trait]
    impl PreferenceRepository for MockPreferenceRepository {
        async fn load(
            &self,
            _profile_id: &str,
            _key: PreferenceKey,
        ) -> Result<Option<Preference>, String> {
            Ok(self.preference.lock().unwrap().clone())
        }

        async fn save(
            &self,
            _profile_id: &str,
            preference: Option<&Preference>,
        ) -> Result<(), String> {
            *self.preference.lock().unwrap() = preference.cloned();
            Ok(())
        }
    }

    struct MockMusicServer;

    #[async_trait]
    impl MusicServer for MockMusicServer {
        async fn ping(&self) -> Result<ServerInfo, String> {
            unimplemented!()
        }

        async fn library_revision(&self) -> Result<Option<String>, String> {
            unimplemented!()
        }

        async fn artists(&self) -> Result<Vec<Artist>, String> {
            unimplemented!()
        }

        async fn albums(&self, _query: AlbumQuery) -> Result<Vec<Album>, String> {
            unimplemented!()
        }

        async fn album(&self, _id: &str) -> Result<AlbumWithSongs, String> {
            unimplemented!()
        }

        async fn genres(&self) -> Result<Vec<Genre>, String> {
            unimplemented!()
        }

        async fn playback_uri(&self, song_id: &str) -> Result<String, String> {
            Ok(format!("https://music.example.com/{song_id}"))
        }

        async fn scrobble(
            &self,
            _song_id: &str,
            _started_at_ms: i64,
            _event: ScrobbleEvent,
        ) -> Result<(), String> {
            Ok(())
        }

        async fn set_favorite(
            &self,
            _item_kind: LibraryItemKind,
            _item_id: &str,
            _favorite: bool,
        ) -> Result<(), String> {
            Ok(())
        }

        async fn set_rating(
            &self,
            _item_kind: LibraryItemKind,
            _item_id: &str,
            _rating: Option<i64>,
        ) -> Result<(), String> {
            Ok(())
        }

        async fn album_artwork(
            &self,
            _cover_art_id: &str,
        ) -> Result<Option<BinaryArtwork>, String> {
            unimplemented!()
        }

        async fn artist_artwork(&self, _artist_id: &str) -> Result<Option<BinaryArtwork>, String> {
            unimplemented!()
        }
    }

    #[derive(Default)]
    struct MockRepository {
        songs: Vec<CachedSong>,
        songs_by_album: Option<HashMap<String, Vec<CachedSong>>>,
    }

    #[async_trait]
    impl LibraryStateRepository for MockRepository {
        async fn summary(&self, _profile_id: &str) -> Result<LibrarySummary, String> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl LibraryCatalogRepository for MockRepository {
        async fn annotation(
            &self,
            _profile_id: &str,
            _item_kind: LibraryItemKind,
            _item_id: &str,
        ) -> Result<Option<LibraryItemAnnotation>, String> {
            unimplemented!()
        }

        async fn set_favorite(
            &self,
            _profile_id: &str,
            _item_kind: LibraryItemKind,
            _item_id: &str,
            _favorite: bool,
        ) -> Result<(), String> {
            unimplemented!()
        }

        async fn set_rating(
            &self,
            _profile_id: &str,
            _item_kind: LibraryItemKind,
            _item_id: &str,
            _rating: Option<i64>,
        ) -> Result<(), String> {
            unimplemented!()
        }

        async fn artist(
            &self,
            _profile_id: &str,
            _artist_id: &str,
        ) -> Result<Option<CachedArtist>, String> {
            unimplemented!()
        }

        async fn search_artists(
            &self,
            _profile_id: &str,
            _query: &str,
            _limit: i64,
        ) -> Result<Vec<CachedArtist>, String> {
            Ok(Vec::new())
        }

        async fn artist_albums(
            &self,
            _profile_id: &str,
            _artist_id: &str,
        ) -> Result<Vec<CachedAlbum>, String> {
            unimplemented!()
        }

        async fn albums(
            &self,
            _profile_id: &str,
            _offset: i64,
            _limit: i64,
            _sort: AlbumSort,
        ) -> Result<Vec<CachedAlbum>, String> {
            unimplemented!()
        }

        async fn album(
            &self,
            _profile_id: &str,
            _album_id: &str,
        ) -> Result<Option<CachedAlbum>, String> {
            unimplemented!()
        }

        async fn album_genres(
            &self,
            _profile_id: &str,
            _album_id: &str,
        ) -> Result<Vec<String>, String> {
            unimplemented!()
        }

        async fn album_disc_count(
            &self,
            _profile_id: &str,
            _album_id: &str,
        ) -> Result<i64, String> {
            unimplemented!()
        }

        async fn album_audio_formats(
            &self,
            _profile_id: &str,
            _album_id: &str,
        ) -> Result<Vec<String>, String> {
            unimplemented!()
        }

        async fn search_albums(
            &self,
            _profile_id: &str,
            _query: &str,
            _limit: i64,
        ) -> Result<Vec<CachedAlbum>, String> {
            unimplemented!()
        }

        async fn search_songs(
            &self,
            _profile_id: &str,
            _query: &str,
            _limit: i64,
        ) -> Result<Vec<CachedSong>, String> {
            unimplemented!()
        }

        async fn songs(
            &self,
            _profile_id: &str,
            album_id: &str,
        ) -> Result<Vec<CachedSong>, String> {
            Ok(self.songs_by_album.as_ref().map_or_else(
                || self.songs.clone(),
                |songs_by_album| songs_by_album.get(album_id).cloned().unwrap_or_default(),
            ))
        }
    }

    struct MemoryCredentialStore;

    impl CredentialStore for MemoryCredentialStore {
        fn save(&self, _id: &str, _password: &str) -> Result<(), String> {
            Ok(())
        }

        fn load(&self, _id: &str) -> Result<String, String> {
            Ok("password".to_string())
        }

        fn delete(&self, _id: &str) -> Result<(), String> {
            Ok(())
        }
    }
}
