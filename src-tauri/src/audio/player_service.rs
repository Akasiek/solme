use crate::{
    library::{CachedSong, LibraryRepository},
    server::MusicServerService,
};
use std::sync::{Arc, Mutex, MutexGuard};

use super::{
    backend::AudioBackend,
    fader::FadingAudioBackend,
    models::{PlaybackState, PlayerStatus},
    preference::{PreferenceRepository, PreferenceService},
    session::PlaybackSession,
};
use crate::events::EventBus;

const PREVIOUS_SONG_RESTART_THRESHOLD_SECONDS: f64 = 5.0;

pub struct PlayerService {
    audio: Arc<dyn AudioBackend>,
    server: Arc<MusicServerService>,
    repository: Arc<dyn LibraryRepository>,
    preference: PreferenceService,
    event_bus: Arc<EventBus>,
    queue: Arc<Mutex<Vec<CachedSong>>>,
}

impl PlayerService {
    pub fn new(
        audio: Box<dyn AudioBackend>,
        server: Arc<MusicServerService>,
        repository: Arc<dyn LibraryRepository>,
        preference: Arc<dyn PreferenceRepository>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        let audio: Arc<dyn AudioBackend> = Arc::new(FadingAudioBackend::new(audio.into()));
        let preference = PreferenceService::new(Arc::clone(&server), preference);
        let queue = Arc::new(Mutex::new(Vec::new()));

        audio.set_status_change_callback(Self::status_change_callback(
            Arc::clone(&audio),
            Arc::clone(&queue),
            Arc::clone(&event_bus),
        ));

        Self {
            audio,
            server,
            repository,
            preference,
            event_bus,
            queue,
        }
    }

    fn lock_queue(&self) -> Result<MutexGuard<'_, Vec<CachedSong>>, String> {
        self.queue
            .lock()
            .map_err(|_| "Player queue lock was poisoned".to_string())
    }

    fn replace_queue(&self, songs: Vec<CachedSong>) -> Result<(), String> {
        let mut queue = self.lock_queue()?;
        *queue = songs;
        drop(queue);
        self.event_bus.publish_player_queue_changed()
    }

    fn append_queue(&self, songs: Vec<CachedSong>) -> Result<(), String> {
        self.lock_queue()?.extend(songs);
        self.event_bus.publish_player_queue_changed()
    }

    fn prepend_queue(&self, songs: Vec<CachedSong>) -> Result<(), String> {
        self.lock_queue()?.splice(0..0, songs);
        self.event_bus.publish_player_queue_changed()
    }

    fn clear_queue(&self) -> Result<(), String> {
        self.lock_queue()?.clear();
        self.event_bus.publish_player_queue_changed()
    }

    fn notify_status_changed(&self) -> Result<(), String> {
        let status = self.status()?;
        self.event_bus.publish_player_status(status)
    }

    fn notify_status_changed_with_position(&self, position_seconds: f64) -> Result<(), String> {
        let mut status = self.status()?;
        status.position_seconds = position_seconds.max(0.0);
        self.event_bus.publish_player_status(status)
    }

    pub fn subscribe_status(&self) -> tokio::sync::broadcast::Receiver<PlayerStatus> {
        self.event_bus.subscribe_player_status()
    }

    pub async fn play_album(
        &self,
        album_id: &str,
        start_song_id: Option<&str>,
    ) -> Result<(), String> {
        let (songs, sources) = self.album_queue_sources(album_id).await?;

        let start_index = match start_song_id {
            Some(song_id) => songs
                .iter()
                .position(|song| song.remote_id == song_id)
                .ok_or_else(|| "Selected song does not belong to this album".to_string())?,
            None => 0,
        };

        self.audio.load_queue(&sources, start_index)?;
        self.replace_queue(songs)?;
        self.notify_status_changed()
    }

    pub async fn queue_album_at_start(&self, album_id: &str) -> Result<(), String> {
        let (songs, sources) = self.album_queue_sources(album_id).await?;

        self.audio.prepend_queue(&sources)?;
        self.prepend_queue(songs)?;
        self.notify_status_changed()
    }

    pub async fn queue_album_at_end(&self, album_id: &str) -> Result<(), String> {
        let (songs, sources) = self.album_queue_sources(album_id).await?;

        self.audio.append_queue(&sources)?;
        self.append_queue(songs)?;
        self.notify_status_changed()
    }

    async fn album_queue_sources(
        &self,
        album_id: &str,
    ) -> Result<(Vec<CachedSong>, Vec<String>), String> {
        let (profile_id, server) = self.server.current_server()?;
        let songs = self.repository.songs(&profile_id, album_id).await?;
        if songs.is_empty() {
            return Err("Album has no cached songs".to_string());
        }

        let sources = Self::playback_sources(&server, &songs).await?;

        Ok((songs, sources))
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
        self.audio.stop()?;
        self.clear_queue()?;
        self.notify_status_changed()
    }

    pub fn next(&self) -> Result<(), String> {
        self.audio.next()?;
        self.notify_status_changed()
    }

    pub fn previous(&self) -> Result<(), String> {
        let position_seconds = self.audio.status().position_seconds;
        if position_seconds > PREVIOUS_SONG_RESTART_THRESHOLD_SECONDS {
            self.seek(0.0)
        } else {
            self.audio.previous()?;
            self.notify_status_changed()
        }
    }

    pub fn skip_to_queue_position(&self, queue_position: usize) -> Result<(), String> {
        self.audio.skip_to_queue_position(queue_position)?;
        self.notify_status_changed()
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
        Self::build_status(&self.audio, &self.queue)
    }

    pub fn queue(&self) -> Result<Vec<CachedSong>, String> {
        Ok(self.lock_queue()?.clone())
    }

    pub(crate) fn session_snapshot(&self) -> Result<Option<PlaybackSession>, String> {
        let audio_status = self.audio.status();
        let queue = self.lock_queue()?.clone();
        let Some(active_index) = audio_status.playlist_position else {
            return Ok(None);
        };
        if queue.is_empty() || active_index >= queue.len() {
            return Ok(None);
        }

        Ok(Some(PlaybackSession {
            queue,
            active_index,
            position_seconds: audio_status.position_seconds.max(0.0),
        }))
    }

    pub(crate) async fn restore_session(&self, session: PlaybackSession) -> Result<(), String> {
        if session.queue.is_empty() || session.active_index >= session.queue.len() {
            return Err("Stored playback session has an invalid queue".to_string());
        }

        let (_, server) = self.server.current_server()?;
        let sources = Self::playback_sources(&server, &session.queue).await?;

        self.replace_queue(session.queue)?;
        self.audio.load_queue_paused(
            &sources,
            session.active_index,
            Some(session.position_seconds),
        )?;
        self.notify_status_changed()
    }

    async fn playback_sources(
        server: &Arc<dyn crate::server::backend::MusicServer>,
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
        audio: Arc<dyn AudioBackend>,
        queue: Arc<Mutex<Vec<CachedSong>>>,
        event_bus: Arc<EventBus>,
    ) -> super::backend::AudioStatusChangeCallback {
        Arc::new(move || match Self::build_status(&audio, &queue) {
            Ok(status) => {
                if let Err(error) = event_bus.publish_player_status(status) {
                    log::error!("Failed to emit player status change: {error}");
                }
            }
            Err(error) => log::error!("Failed to build player status change: {error}"),
        })
    }

    fn build_status(
        audio: &Arc<dyn AudioBackend>,
        queue: &Arc<Mutex<Vec<CachedSong>>>,
    ) -> Result<PlayerStatus, String> {
        let audio_status = audio.status();
        let queue = queue
            .lock()
            .map_err(|_| "Player queue lock was poisoned".to_string())?;
        let current_song = audio_status
            .playlist_position
            .and_then(|position| queue.get(position))
            .cloned();
        let state = if !audio_status.playing {
            PlaybackState::Stopped
        } else if audio_status.paused {
            PlaybackState::Paused
        } else {
            PlaybackState::Playing
        };

        Ok(PlayerStatus {
            state,
            current_song,
            position_seconds: audio_status.position_seconds,
            duration_seconds: audio_status.duration_seconds,
            queue_position: audio_status.playlist_position.map(|position| position + 1),
            queue_length: queue.len(),
            volume: audio_status.volume,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use sqlx::SqlitePool;

    use super::PlayerService;
    use crate::events::EventBus;
    use crate::{
        audio::{
            backend::{AudioBackend, AudioBackendStatus},
            models::PlaybackState,
            preference::{Preference, PreferenceKey, PreferenceRepository},
            session::PlaybackSession,
        },
        credentials::CredentialStore,
        library::{
            models::{
                Album, AlbumSort, AlbumWithSongs, Artist, ArtworkCacheRecord, ArtworkCandidate,
                BinaryArtwork, CachedAlbum, CachedArtist, CachedSong, Genre, LibrarySnapshot,
                LibrarySummary,
            },
            LibraryRepository,
        },
        server::{backend::MusicServer, MusicServerService, ScrobbleEvent, ServerInfo},
    };

    #[test]
    fn plays_full_album_from_selected_song() {
        tauri::async_runtime::block_on(async {
            let server_service = test_server_service().await;

            let songs = vec![song("song-1"), song("song-2"), song("song-3")];
            let repository: Arc<dyn LibraryRepository> = Arc::new(MockRepository {
                songs: songs.clone(),
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
        });
    }

    #[test]
    fn rejects_song_outside_selected_album() {
        tauri::async_runtime::block_on(async {
            let server_service = test_server_service().await;
            let repository: Arc<dyn LibraryRepository> = Arc::new(MockRepository {
                songs: vec![song("song-1")],
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
            let repository: Arc<dyn LibraryRepository> = Arc::new(MockRepository {
                songs: songs.clone(),
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
            player.queue_album_at_end("album-2").await.unwrap();

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
    fn prepends_album_to_current_queue() {
        tauri::async_runtime::block_on(async {
            let server_service = test_server_service().await;

            let songs = vec![song("song-1"), song("song-2")];
            let repository: Arc<dyn LibraryRepository> = Arc::new(MockRepository {
                songs: songs.clone(),
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
            player.queue_album_at_start("album-2").await.unwrap();

            let audio = audio_state.lock().unwrap();
            assert_eq!(audio.start_index, 3);
            assert_eq!(
                audio.prepended_sources,
                [
                    "https://music.example.com/song-1",
                    "https://music.example.com/song-2"
                ]
            );
            drop(audio);

            let status = player.status().unwrap();
            assert_eq!(status.current_song.unwrap().remote_id, "song-2");
            assert_eq!(status.queue_position, Some(4));
            assert_eq!(status.queue_length, 4);
        });
    }

    #[test]
    fn restores_paused_playback_session() {
        tauri::async_runtime::block_on(async {
            let server_service = test_server_service().await;
            let repository: Arc<dyn LibraryRepository> =
                Arc::new(MockRepository { songs: Vec::new() });
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
    fn restores_saved_volume_preference() {
        tauri::async_runtime::block_on(async {
            let server_service = test_server_service().await;
            let repository: Arc<dyn LibraryRepository> =
                Arc::new(MockRepository { songs: Vec::new() });
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
            let repository: Arc<dyn LibraryRepository> =
                Arc::new(MockRepository { songs: Vec::new() });
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
            let server_service = test_server_service().await;
            let repository: Arc<dyn LibraryRepository> =
                Arc::new(MockRepository { songs: Vec::new() });
            let audio_state = Arc::new(Mutex::new(MockAudioState {
                playing: true,
                position_seconds: 6.0,
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

            player.previous().unwrap();

            let audio = audio_state.lock().unwrap();
            assert_eq!(audio.position_seconds, 0.0);
            assert_eq!(audio.previous_calls, 0);
        });
    }

    #[test]
    fn previous_moves_to_previous_song_before_threshold() {
        tauri::async_runtime::block_on(async {
            let server_service = test_server_service().await;
            let repository: Arc<dyn LibraryRepository> =
                Arc::new(MockRepository { songs: Vec::new() });
            let audio_state = Arc::new(Mutex::new(MockAudioState {
                playing: true,
                position_seconds: 5.0,
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

            player.previous().unwrap();

            let audio = audio_state.lock().unwrap();
            assert_eq!(audio.position_seconds, 5.0);
            assert_eq!(audio.previous_calls, 1);
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
        prepended_sources: Vec<String>,
        appended_sources: Vec<String>,
        start_index: usize,
        playing: bool,
        paused: bool,
        position_seconds: f64,
        volume: f64,
        previous_calls: usize,
        keep_stale_position_after_seek: bool,
    }

    struct MockAudioBackend {
        state: Arc<Mutex<MockAudioState>>,
    }

    impl AudioBackend for MockAudioBackend {
        fn set_status_change_callback(
            &self,
            _callback: crate::audio::backend::AudioStatusChangeCallback,
        ) {
        }

        fn load_queue(&self, sources: &[String], start_index: usize) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            state.sources = sources.to_vec();
            state.start_index = start_index;
            state.playing = true;
            state.paused = false;
            Ok(())
        }

        fn load_queue_paused(
            &self,
            sources: &[String],
            start_index: usize,
            position_seconds: Option<f64>,
        ) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            state.sources = sources.to_vec();
            state.start_index = start_index;
            state.playing = true;
            state.paused = true;
            if let Some(position_seconds) = position_seconds {
                state.position_seconds = position_seconds;
            }
            Ok(())
        }

        fn append_queue(&self, sources: &[String]) -> Result<(), String> {
            self.state
                .lock()
                .unwrap()
                .appended_sources
                .extend_from_slice(sources);
            Ok(())
        }

        fn prepend_queue(&self, sources: &[String]) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            state.prepended_sources.extend_from_slice(sources);
            state.start_index += sources.len();
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
            self.state.lock().unwrap().playing = false;
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
                playlist_position: state.playing.then_some(state.start_index),
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

        async fn albums(&self) -> Result<Vec<Album>, String> {
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

    struct MockRepository {
        songs: Vec<CachedSong>,
    }

    #[async_trait]
    impl LibraryRepository for MockRepository {
        async fn server_revision(&self, _profile_id: &str) -> Result<Option<String>, String> {
            unimplemented!()
        }

        async fn activate_snapshot(
            &self,
            _profile_id: &str,
            _generation: &str,
            _revision: Option<&str>,
            _snapshot: &LibrarySnapshot,
            _completed_at: i64,
        ) -> Result<(), String> {
            unimplemented!()
        }

        async fn summary(&self, _profile_id: &str) -> Result<LibrarySummary, String> {
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
            _album_id: &str,
        ) -> Result<Vec<CachedSong>, String> {
            Ok(self.songs.clone())
        }

        async fn artwork_is_fresh(
            &self,
            _profile_id: &str,
            _kind: &str,
            _remote_id: &str,
            _source_key: Option<&str>,
            _fresh_after: i64,
        ) -> Result<bool, String> {
            unimplemented!()
        }

        async fn artwork_candidates(
            &self,
            _profile_id: &str,
        ) -> Result<Vec<ArtworkCandidate>, String> {
            unimplemented!()
        }

        async fn save_artwork(
            &self,
            _profile_id: &str,
            _artwork: ArtworkCacheRecord,
        ) -> Result<(), String> {
            unimplemented!()
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
