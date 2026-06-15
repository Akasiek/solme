use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

use futures_util::{stream, StreamExt, TryStreamExt};
use uuid::Uuid;

use crate::server::{backend::MusicServer, MusicServerService};

use super::{
    artwork::synchronize_artwork_item,
    models::{
        Album, AlbumWithSongs, CachedAlbum, CachedSong, LibrarySnapshot, LibrarySummary,
        LibrarySyncPhase, LibrarySyncStatus,
    },
    repository::LibraryRepository,
    time::now_epoch_seconds,
};

const ALBUM_CONCURRENCY: usize = 6;
const ARTWORK_CONCURRENCY: usize = 4;
const ARTWORK_MAX_AGE_SECONDS: i64 = 7 * 24 * 60 * 60;

pub struct LibrarySyncService {
    server: Arc<MusicServerService>,
    repository: Arc<dyn LibraryRepository>,
    artwork_root: PathBuf,
    running: AtomicBool,
    status: RwLock<LibrarySyncStatus>,
}

impl LibrarySyncService {
    pub fn new(
        server: Arc<MusicServerService>,
        repository: Arc<dyn LibraryRepository>,
        artwork_root: PathBuf,
    ) -> Self {
        Self {
            server,
            repository,
            artwork_root,
            running: AtomicBool::new(false),
            status: RwLock::new(LibrarySyncStatus::default()),
        }
    }

    pub fn start(self: &Arc<Self>, force: bool) -> Result<(), String> {
        self.server.current_server()?;
        self.begin_synchronization()?;

        let service = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            if let Err(error) = service.synchronize(force).await {
                service.update_status(|status| {
                    status.phase = LibrarySyncPhase::Failed;
                    status.last_error = Some(error);
                });
            }
            service.finish_synchronization();
        });
        Ok(())
    }

    fn begin_synchronization(&self) -> Result<(), String> {
        let was_already_running = self.running.swap(true, Ordering::SeqCst);
        if was_already_running {
            return Err("Library synchronization is already running".to_string());
        }
        Ok(())
    }

    fn finish_synchronization(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn status(&self) -> Result<LibrarySyncStatus, String> {
        self.status
            .read()
            .map(|status| status.clone())
            .map_err(|_| "Library sync status lock was poisoned".to_string())
    }

    pub async fn summary(&self) -> Result<LibrarySummary, String> {
        let Some(profile_id) = self.server.cache_profile_id().await? else {
            return Ok(LibrarySummary {
                artist_count: 0,
                album_count: 0,
                song_count: 0,
                last_success_at: None,
            });
        };
        self.repository.summary(&profile_id).await
    }

    pub async fn albums(&self, offset: i64, limit: i64) -> Result<Vec<CachedAlbum>, String> {
        let Some(profile_id) = self.server.cache_profile_id().await? else {
            return Ok(Vec::new());
        };
        self.repository.albums(&profile_id, offset, limit).await
    }

    pub async fn album(&self, album_id: &str) -> Result<Option<CachedAlbum>, String> {
        let Some(profile_id) = self.server.cache_profile_id().await? else {
            return Ok(None);
        };
        self.repository.album(&profile_id, album_id).await
    }

    pub async fn search_albums(&self, query: &str, limit: i64) -> Result<Vec<CachedAlbum>, String> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let Some(profile_id) = self.server.cache_profile_id().await? else {
            return Ok(Vec::new());
        };
        self.repository
            .search_albums(&profile_id, query, limit)
            .await
    }

    pub async fn songs(&self, album_id: &str) -> Result<Vec<CachedSong>, String> {
        let Some(profile_id) = self.server.cache_profile_id().await? else {
            return Ok(Vec::new());
        };
        self.repository.songs(&profile_id, album_id).await
    }

    async fn synchronize(&self, force: bool) -> Result<(), String> {
        self.reset_status();
        let (profile_id, server) = self.server.current_server()?;
        self.synchronize_metadata(&profile_id, Arc::clone(&server), force)
            .await?;
        self.synchronize_artwork(&profile_id, server).await?;
        self.set_phase(LibrarySyncPhase::Completed);
        Ok(())
    }

    fn reset_status(&self) {
        self.update_status(|status| {
            *status = LibrarySyncStatus {
                phase: LibrarySyncPhase::Metadata,
                ..LibrarySyncStatus::default()
            };
        });
    }

    async fn synchronize_metadata(
        &self,
        profile_id: &str,
        server: Arc<dyn MusicServer>,
        force: bool,
    ) -> Result<(), String> {
        let revision = server.library_revision().await?;
        if self
            .metadata_requires_refresh(profile_id, revision.as_deref(), force)
            .await?
        {
            self.refresh_metadata(profile_id, server, revision.as_deref())
                .await
        } else {
            self.restore_cached_metadata_status(profile_id).await
        }
    }

    async fn metadata_requires_refresh(
        &self,
        profile_id: &str,
        server_revision: Option<&str>,
        force: bool,
    ) -> Result<bool, String> {
        if force || server_revision.is_none() {
            return Ok(true);
        }

        let cached_revision = self.repository.server_revision(profile_id).await?;
        Ok(server_revision != cached_revision.as_deref())
    }

    async fn refresh_metadata(
        &self,
        profile_id: &str,
        server: Arc<dyn MusicServer>,
        revision: Option<&str>,
    ) -> Result<(), String> {
        let snapshot = self.fetch_snapshot(server).await?;
        let completed_at = now_epoch_seconds()?;

        self.set_phase(LibrarySyncPhase::Activating);
        self.repository
            .activate_snapshot(
                profile_id,
                &Uuid::new_v4().to_string(),
                revision,
                &snapshot,
                completed_at,
            )
            .await?;
        self.update_status(|status| {
            status.last_success_at = Some(completed_at);
        });
        Ok(())
    }

    async fn fetch_snapshot(
        &self,
        server: Arc<dyn MusicServer>,
    ) -> Result<LibrarySnapshot, String> {
        let (artists, genres) = futures_util::try_join!(server.artists(), server.genres())?;
        self.update_status(|status| {
            status.processed_artists = artists.len() as u64;
        });

        let album_index = server.albums().await?;
        let albums = self.fetch_albums_with_songs(server, album_index).await?;

        Ok(LibrarySnapshot {
            artists,
            albums,
            genres,
        })
    }

    async fn fetch_albums_with_songs(
        &self,
        server: Arc<dyn MusicServer>,
        album_index: Vec<Album>,
    ) -> Result<Vec<AlbumWithSongs>, String> {
        let status = &self.status;
        stream::iter(album_index)
            .map(|album| {
                let server = Arc::clone(&server);
                async move { server.album(&album.remote_id).await }
            })
            .buffer_unordered(ALBUM_CONCURRENCY)
            .map_ok(|album| {
                if let Ok(mut status) = status.write() {
                    status.processed_albums += 1;
                    status.processed_songs += album.songs.len() as u64;
                }
                album
            })
            .try_collect()
            .await
    }

    async fn restore_cached_metadata_status(&self, profile_id: &str) -> Result<(), String> {
        let summary = self.repository.summary(profile_id).await?;
        self.update_status(|status| {
            status.processed_artists = summary.artist_count as u64;
            status.processed_albums = summary.album_count as u64;
            status.processed_songs = summary.song_count as u64;
            status.last_success_at = summary.last_success_at;
        });
        Ok(())
    }

    async fn synchronize_artwork(
        &self,
        profile_id: &str,
        server: Arc<dyn MusicServer>,
    ) -> Result<(), String> {
        self.set_phase(LibrarySyncPhase::Artwork);
        let candidates = self.repository.artwork_candidates(profile_id).await?;
        self.update_status(|status| {
            status.total_artwork = candidates.len() as u64;
        });

        let fresh_after = now_epoch_seconds()? - ARTWORK_MAX_AGE_SECONDS;
        let profile_id = profile_id.to_string();
        let repository = Arc::clone(&self.repository);
        let artwork_root = self.artwork_root.clone();
        let status = &self.status;

        let mut tasks = stream::iter(candidates.into_iter())
            .map(|candidate| {
                let server = Arc::clone(&server);
                let repository = Arc::clone(&repository);
                let artwork_root = artwork_root.clone();
                let profile_id = profile_id.clone();
                async move {
                    synchronize_artwork_item(
                        &profile_id,
                        candidate,
                        server,
                        repository,
                        &artwork_root,
                        fresh_after,
                    )
                    .await
                }
            })
            .buffer_unordered(ARTWORK_CONCURRENCY);

        let mut first_error = None;
        while let Some(result) = tasks.next().await {
            if let Err(error) = result {
                first_error.get_or_insert(error);
            }
            if let Ok(mut status) = status.write() {
                status.processed_artwork += 1;
            }
        }
        if let Some(error) = first_error {
            self.update_status(|status| {
                status.last_error = Some(format!("Some artwork could not be cached: {error}"));
            });
        }
        Ok(())
    }

    fn update_status(&self, update: impl FnOnce(&mut LibrarySyncStatus)) {
        if let Ok(mut status) = self.status.write() {
            update(&mut status);
        }
    }

    fn set_phase(&self, phase: LibrarySyncPhase) {
        self.update_status(|status| {
            status.phase = phase;
        });
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        path::PathBuf,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc, Mutex,
        },
    };

    use async_trait::async_trait;
    use uuid::Uuid;

    use super::LibrarySyncService;
    use crate::{
        credentials::CredentialStore,
        library::{
            models::{
                Album, AlbumWithSongs, Artist, ArtworkCacheRecord, ArtworkCandidate, BinaryArtwork,
                CachedAlbum, CachedSong, Genre, LibrarySnapshot, LibrarySummary, LibrarySyncPhase,
                Song,
            },
            repository::LibraryRepository,
        },
        server::{backend::MusicServer, MusicServerService, ScrobbleEvent, ServerInfo},
    };

    #[test]
    fn full_sync_activates_snapshot_and_updates_status() {
        tauri::async_runtime::block_on(async {
            let server = Arc::new(MockMusicServer::new(Some("revision-2")));
            let repository = Arc::new(MockRepository::new(Some("revision-1")));
            let service = service(Arc::clone(&server), Arc::clone(&repository));

            service.synchronize(false).await.unwrap();

            assert_eq!(repository.activation_count.load(Ordering::SeqCst), 1);
            assert_eq!(server.artist_calls.load(Ordering::SeqCst), 1);
            assert_eq!(server.album_calls.load(Ordering::SeqCst), 1);
            assert_eq!(server.album_detail_calls.load(Ordering::SeqCst), 1);

            let status = service.status().unwrap();
            assert_eq!(status.phase, LibrarySyncPhase::Completed);
            assert_eq!(status.processed_artists, 1);
            assert_eq!(status.processed_albums, 1);
            assert_eq!(status.processed_songs, 1);
            assert!(status.last_success_at.is_some());
            assert!(status.last_error.is_none());
        });
    }

    #[test]
    fn unchanged_revision_skips_metadata_requests() {
        tauri::async_runtime::block_on(async {
            let server = Arc::new(MockMusicServer::new(Some("revision-1")));
            let repository = Arc::new(MockRepository::new(Some("revision-1")));
            let service = service(Arc::clone(&server), Arc::clone(&repository));

            service.synchronize(false).await.unwrap();

            assert_eq!(repository.activation_count.load(Ordering::SeqCst), 0);
            assert_eq!(server.artist_calls.load(Ordering::SeqCst), 0);
            assert_eq!(server.album_calls.load(Ordering::SeqCst), 0);
            assert_eq!(server.album_detail_calls.load(Ordering::SeqCst), 0);

            let status = service.status().unwrap();
            assert_eq!(status.phase, LibrarySyncPhase::Completed);
            assert_eq!(status.processed_artists, 4);
            assert_eq!(status.processed_albums, 5);
            assert_eq!(status.processed_songs, 6);
            assert_eq!(status.last_success_at, Some(100));
        });
    }

    #[test]
    fn forced_sync_ignores_matching_revision() {
        tauri::async_runtime::block_on(async {
            let server = Arc::new(MockMusicServer::new(Some("revision-1")));
            let repository = Arc::new(MockRepository::new(Some("revision-1")));
            let service = service(Arc::clone(&server), Arc::clone(&repository));

            service.synchronize(true).await.unwrap();

            assert_eq!(repository.activation_count.load(Ordering::SeqCst), 1);
            assert_eq!(server.artist_calls.load(Ordering::SeqCst), 1);
            assert_eq!(server.album_detail_calls.load(Ordering::SeqCst), 1);
        });
    }

    #[test]
    fn rejects_second_running_sync() {
        let server = Arc::new(MockMusicServer::new(Some("revision-1")));
        let repository = Arc::new(MockRepository::new(Some("revision-1")));
        let service = service(server, repository);
        service.running.store(true, Ordering::SeqCst);

        assert_eq!(
            service.start(false).unwrap_err(),
            "Library synchronization is already running"
        );
    }

    #[test]
    fn artwork_failure_does_not_fail_metadata_sync() {
        tauri::async_runtime::block_on(async {
            let server = Arc::new(MockMusicServer::new(Some("revision-1")));
            server.fail_artwork.store(1, Ordering::SeqCst);
            let repository = Arc::new(MockRepository::new(Some("revision-1")));
            repository
                .artwork_candidates
                .lock()
                .unwrap()
                .push(ArtworkCandidate {
                    kind: "album",
                    remote_id: "album-1".to_string(),
                    source_id: "cover-1".to_string(),
                });
            let service = service(server, repository);

            service.synchronize(false).await.unwrap();

            let status = service.status().unwrap();
            assert_eq!(status.phase, LibrarySyncPhase::Completed);
            assert_eq!(status.processed_artwork, 1);
            assert_eq!(status.total_artwork, 1);
            assert!(status
                .last_error
                .as_deref()
                .is_some_and(|error| error.contains("Some artwork could not be cached")));
        });
    }

    #[test]
    fn successful_artwork_is_written_and_recorded() {
        tauri::async_runtime::block_on(async {
            let directory = std::env::temp_dir().join(format!("solme-sync-{}", Uuid::new_v4()));
            let server = Arc::new(MockMusicServer::new(Some("revision-1")));
            let repository = Arc::new(MockRepository::new(Some("revision-1")));
            repository
                .artwork_candidates
                .lock()
                .unwrap()
                .push(ArtworkCandidate {
                    kind: "album",
                    remote_id: "album-1".to_string(),
                    source_id: "cover-1".to_string(),
                });
            let service = service_with_artwork_root(
                Arc::clone(&server),
                Arc::clone(&repository),
                directory.clone(),
            );

            service.synchronize(false).await.unwrap();

            let saved = repository.saved_artwork.lock().unwrap();
            assert_eq!(saved.len(), 1);
            assert_eq!(saved[0].source_key, "cover-1");
            assert!(PathBuf::from(&saved[0].local_path).is_file());
            drop(saved);

            std::fs::remove_dir_all(directory).unwrap();
        });
    }

    fn service(
        server: Arc<MockMusicServer>,
        repository: Arc<MockRepository>,
    ) -> Arc<LibrarySyncService> {
        let directory = std::env::temp_dir().join(format!("solme-sync-{}", Uuid::new_v4()));
        service_with_artwork_root(server, repository, directory)
    }

    fn service_with_artwork_root(
        server: Arc<MockMusicServer>,
        repository: Arc<MockRepository>,
        artwork_root: PathBuf,
    ) -> Arc<LibrarySyncService> {
        let profile_path =
            std::env::temp_dir().join(format!("solme-profile-{}.json", Uuid::new_v4()));
        let server_service = Arc::new(MusicServerService::new(
            profile_path,
            Box::new(MemoryCredentialStore),
        ));
        server_service
            .set_current_server("profile-1".to_string(), server)
            .unwrap();

        Arc::new(LibrarySyncService::new(
            server_service,
            repository,
            artwork_root,
        ))
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

    struct MockMusicServer {
        revision: Option<String>,
        artist_calls: AtomicUsize,
        album_calls: AtomicUsize,
        album_detail_calls: AtomicUsize,
        fail_artwork: AtomicUsize,
    }

    impl MockMusicServer {
        fn new(revision: Option<&str>) -> Self {
            Self {
                revision: revision.map(str::to_string),
                artist_calls: AtomicUsize::new(0),
                album_calls: AtomicUsize::new(0),
                album_detail_calls: AtomicUsize::new(0),
                fail_artwork: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl MusicServer for MockMusicServer {
        async fn ping(&self) -> Result<ServerInfo, String> {
            Ok(ServerInfo {
                server_type: "mock".to_string(),
                server_version: None,
                api_version: "1".to_string(),
                username: "user".to_string(),
            })
        }

        async fn library_revision(&self) -> Result<Option<String>, String> {
            Ok(self.revision.clone())
        }

        async fn artists(&self) -> Result<Vec<Artist>, String> {
            self.artist_calls.fetch_add(1, Ordering::SeqCst);
            Ok(vec![Artist {
                remote_id: "artist-1".to_string(),
                name: "Artist".to_string(),
                album_count: 1,
            }])
        }

        async fn albums(&self) -> Result<Vec<Album>, String> {
            self.album_calls.fetch_add(1, Ordering::SeqCst);
            Ok(vec![album()])
        }

        async fn album(&self, _id: &str) -> Result<AlbumWithSongs, String> {
            self.album_detail_calls.fetch_add(1, Ordering::SeqCst);
            Ok(AlbumWithSongs {
                album: album(),
                songs: vec![song()],
            })
        }

        async fn genres(&self) -> Result<Vec<Genre>, String> {
            Ok(vec![Genre {
                name: "Jazz".to_string(),
                song_count: 1,
                album_count: 1,
            }])
        }

        fn playback_uri(&self, song_id: &str) -> Result<String, String> {
            Ok(format!("https://music.example.com/stream/{song_id}"))
        }

        async fn scrobble(
            &self,
            _song_id: &str,
            _started_at_ms: i64,
            _event: ScrobbleEvent,
        ) -> Result<(), String> {
            Ok(())
        }

        async fn album_artwork(&self, cover_art_id: &str) -> Result<Option<BinaryArtwork>, String> {
            if self.fail_artwork.load(Ordering::SeqCst) > 0 {
                return Err("artwork failed".to_string());
            }
            Ok(Some(BinaryArtwork {
                source_key: cover_art_id.to_string(),
                content_type: "image/png".to_string(),
                bytes: vec![1, 2, 3],
                etag: Some("etag".to_string()),
                last_modified: None,
            }))
        }

        async fn artist_artwork(&self, _artist_id: &str) -> Result<Option<BinaryArtwork>, String> {
            Ok(None)
        }
    }

    struct MockRepository {
        revision: Option<String>,
        activation_count: AtomicUsize,
        artwork_candidates: Mutex<Vec<ArtworkCandidate>>,
        saved_artwork: Mutex<Vec<ArtworkCacheRecord>>,
        freshness: Mutex<HashMap<String, bool>>,
    }

    impl MockRepository {
        fn new(revision: Option<&str>) -> Self {
            Self {
                revision: revision.map(str::to_string),
                activation_count: AtomicUsize::new(0),
                artwork_candidates: Mutex::new(Vec::new()),
                saved_artwork: Mutex::new(Vec::new()),
                freshness: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl LibraryRepository for MockRepository {
        async fn server_revision(&self, _profile_id: &str) -> Result<Option<String>, String> {
            Ok(self.revision.clone())
        }

        async fn activate_snapshot(
            &self,
            _profile_id: &str,
            _generation: &str,
            _revision: Option<&str>,
            snapshot: &LibrarySnapshot,
            _completed_at: i64,
        ) -> Result<(), String> {
            assert_eq!(snapshot.artists.len(), 1);
            assert_eq!(snapshot.albums.len(), 1);
            assert_eq!(snapshot.genres.len(), 1);
            self.activation_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn summary(&self, _profile_id: &str) -> Result<LibrarySummary, String> {
            Ok(LibrarySummary {
                artist_count: 4,
                album_count: 5,
                song_count: 6,
                last_success_at: Some(100),
            })
        }

        async fn albums(
            &self,
            _profile_id: &str,
            _offset: i64,
            _limit: i64,
        ) -> Result<Vec<CachedAlbum>, String> {
            Ok(Vec::new())
        }

        async fn album(
            &self,
            _profile_id: &str,
            _album_id: &str,
        ) -> Result<Option<CachedAlbum>, String> {
            Ok(None)
        }

        async fn search_albums(
            &self,
            _profile_id: &str,
            _query: &str,
            _limit: i64,
        ) -> Result<Vec<CachedAlbum>, String> {
            Ok(Vec::new())
        }

        async fn songs(
            &self,
            _profile_id: &str,
            _album_id: &str,
        ) -> Result<Vec<CachedSong>, String> {
            Ok(Vec::new())
        }

        async fn artwork_is_fresh(
            &self,
            _profile_id: &str,
            kind: &str,
            remote_id: &str,
            _source_key: Option<&str>,
            _fresh_after: i64,
        ) -> Result<bool, String> {
            Ok(*self
                .freshness
                .lock()
                .unwrap()
                .get(&format!("{kind}:{remote_id}"))
                .unwrap_or(&false))
        }

        async fn artwork_candidates(
            &self,
            _profile_id: &str,
        ) -> Result<Vec<ArtworkCandidate>, String> {
            Ok(self.artwork_candidates.lock().unwrap().clone())
        }

        async fn save_artwork(
            &self,
            _profile_id: &str,
            artwork: ArtworkCacheRecord,
        ) -> Result<(), String> {
            self.saved_artwork.lock().unwrap().push(artwork);
            Ok(())
        }
    }

    fn album() -> Album {
        Album {
            remote_id: "album-1".to_string(),
            name: "Album".to_string(),
            artist_id: Some("artist-1".to_string()),
            artist_name: "Artist".to_string(),
            year: Some(2026),
            song_count: 1,
            duration_seconds: 180,
            cover_art_id: Some("cover-1".to_string()),
            genres: vec!["Jazz".to_string()],
        }
    }

    fn song() -> Song {
        Song {
            remote_id: "song-1".to_string(),
            album_id: "album-1".to_string(),
            artist_id: Some("artist-1".to_string()),
            title: "Song".to_string(),
            artist_name: "Artist".to_string(),
            album_name: "Album".to_string(),
            track_number: Some(1),
            disc_number: Some(1),
            year: Some(2026),
            duration_seconds: 180,
            suffix: Some("opus".to_string()),
            content_type: Some("audio/ogg".to_string()),
            cover_art_id: Some("cover-1".to_string()),
            genres: vec!["Jazz".to_string()],
        }
    }
}
