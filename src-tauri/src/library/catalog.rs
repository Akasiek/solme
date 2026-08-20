use std::sync::Arc;

use futures_util::future::join;

use crate::server::{AlbumQuery, MusicServerService};

use super::{
    models::{
        Album, AlbumSort, CachedAlbum, CachedAlbumDetails, CachedArtist, CachedArtistDetails,
        CachedSong, HomeAlbumSections, LibraryItemAnnotation, LibraryItemKind, LibrarySummary,
    },
    repository::LibraryCatalogRepository,
};

pub struct LibraryCatalogService {
    server: Arc<MusicServerService>,
    repository: Arc<dyn LibraryCatalogRepository>,
}

impl LibraryCatalogService {
    pub fn new(
        server: Arc<MusicServerService>,
        repository: Arc<dyn LibraryCatalogRepository>,
    ) -> Self {
        Self { server, repository }
    }

    pub async fn summary(&self) -> Result<LibrarySummary, String> {
        let Some(profile_id) = self.server.cache_profile_id().await? else {
            return Ok(LibrarySummary {
                artist_count: 0,
                album_count: 0,
                song_count: 0,
                genre_count: 0,
                last_success_at: None,
            });
        };
        self.repository.summary(&profile_id).await
    }

    pub async fn set_favorite(
        &self,
        item_kind: LibraryItemKind,
        item_id: &str,
        favorite: bool,
    ) -> Result<LibraryItemAnnotation, String> {
        let item_id = validate_item_id(item_id)?;
        let (profile_id, server) = self.server.current_server()?;
        let mut annotation = self
            .repository
            .annotation(&profile_id, item_kind, item_id)
            .await?
            .ok_or_else(|| missing_item_error(item_kind, item_id))?;

        server.set_favorite(item_kind, item_id, favorite).await?;
        self.repository
            .set_favorite(&profile_id, item_kind, item_id, favorite)
            .await?;
        annotation.favorite = favorite;
        Ok(annotation)
    }

    pub async fn set_rating(
        &self,
        item_kind: LibraryItemKind,
        item_id: &str,
        rating: Option<i64>,
    ) -> Result<LibraryItemAnnotation, String> {
        let item_id = validate_item_id(item_id)?;
        if rating.is_some_and(|rating| !(1..=5).contains(&rating)) {
            return Err("Rating must be between 1 and 5".to_string());
        }
        let (profile_id, server) = self.server.current_server()?;
        let mut annotation = self
            .repository
            .annotation(&profile_id, item_kind, item_id)
            .await?
            .ok_or_else(|| missing_item_error(item_kind, item_id))?;

        server.set_rating(item_kind, item_id, rating).await?;
        self.repository
            .set_rating(&profile_id, item_kind, item_id, rating)
            .await?;
        annotation.rating = rating;
        Ok(annotation)
    }

    pub async fn albums(&self, offset: i64, limit: i64) -> Result<Vec<CachedAlbum>, String> {
        let Some(profile_id) = self.server.cache_profile_id().await? else {
            return Ok(Vec::new());
        };
        self.repository
            .albums(&profile_id, offset, limit, AlbumSort::Artist)
            .await
    }

    pub async fn home_album_sections(&self, limit: i64) -> Result<HomeAlbumSections, String> {
        let Some(profile_id) = self.server.cache_profile_id().await? else {
            return Ok(HomeAlbumSections {
                hero_random_albums: Vec::new(),
                recently_played_albums: Vec::new(),
                most_played_albums: Vec::new(),
                random_albums: Vec::new(),
                newly_added_albums: Vec::new(),
                newly_released_albums: Vec::new(),
            });
        };
        let limit = limit.clamp(1, 50);
        let (recently_played_albums, most_played_albums) = self
            .played_album_sections(&profile_id, limit as usize)
            .await?;
        let hero_random_albums = self
            .repository
            .albums(&profile_id, 0, 5, AlbumSort::Random)
            .await?;
        let random_albums = self
            .repository
            .albums(&profile_id, 0, limit, AlbumSort::Random)
            .await?;
        let newly_added_albums = self
            .repository
            .albums(&profile_id, 0, limit, AlbumSort::RecentlyAdded)
            .await?;
        let newly_released_albums = self
            .repository
            .albums(&profile_id, 0, limit, AlbumSort::RecentlyReleased)
            .await?;

        Ok(HomeAlbumSections {
            hero_random_albums,
            recently_played_albums,
            most_played_albums,
            random_albums,
            newly_added_albums,
            newly_released_albums,
        })
    }

    async fn played_album_sections(
        &self,
        profile_id: &str,
        limit: usize,
    ) -> Result<(Vec<CachedAlbum>, Vec<CachedAlbum>), String> {
        let Ok((_, server)) = self.server.current_server() else {
            return Ok((Vec::new(), Vec::new()));
        };
        let (recent, frequent) = join(
            server.albums(AlbumQuery::RecentlyPlayed { limit }),
            server.albums(AlbumQuery::MostPlayed { limit }),
        )
        .await;

        Ok((
            self.cached_server_albums(profile_id, recent).await?,
            self.cached_server_albums(profile_id, frequent).await?,
        ))
    }

    async fn cached_server_albums(
        &self,
        profile_id: &str,
        albums: Result<Vec<Album>, String>,
    ) -> Result<Vec<CachedAlbum>, String> {
        let albums = match albums {
            Ok(albums) => albums,
            Err(error) => {
                log::warn!("Failed to load a played albums section: {error}");
                return Ok(Vec::new());
            }
        };
        let album_ids = albums
            .into_iter()
            .map(|album| album.remote_id)
            .collect::<Vec<_>>();
        self.repository.albums_by_ids(profile_id, &album_ids).await
    }

    pub async fn artist(&self, artist_id: &str) -> Result<Option<CachedArtistDetails>, String> {
        let Some(profile_id) = self.server.cache_profile_id().await? else {
            return Ok(None);
        };
        let Some(artist) = self.repository.artist(&profile_id, artist_id).await? else {
            return Ok(None);
        };
        let albums = self
            .repository
            .artist_albums(&profile_id, artist_id)
            .await?;
        Ok(Some(CachedArtistDetails { artist, albums }))
    }

    pub async fn search_artists(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<CachedArtist>, String> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let Some(profile_id) = self.server.cache_profile_id().await? else {
            return Ok(Vec::new());
        };
        self.repository
            .search_artists(&profile_id, query, limit)
            .await
    }

    pub async fn album(&self, album_id: &str) -> Result<Option<CachedAlbumDetails>, String> {
        let Some(profile_id) = self.server.cache_profile_id().await? else {
            return Ok(None);
        };
        let Some(album) = self.repository.album(&profile_id, album_id).await? else {
            return Ok(None);
        };
        let genres = self.repository.album_genres(&profile_id, album_id).await?;
        let disc_count = self
            .repository
            .album_disc_count(&profile_id, album_id)
            .await?;
        let audio_formats = self
            .repository
            .album_audio_formats(&profile_id, album_id)
            .await?;
        let songs = self.repository.songs(&profile_id, album_id).await?;
        Ok(Some(CachedAlbumDetails {
            album,
            genres,
            disc_count,
            audio_formats,
            songs,
        }))
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

    pub async fn search_songs(&self, query: &str, limit: i64) -> Result<Vec<CachedSong>, String> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let Some(profile_id) = self.server.cache_profile_id().await? else {
            return Ok(Vec::new());
        };
        self.repository
            .search_songs(&profile_id, query, limit)
            .await
    }

    pub async fn songs(&self, album_id: &str) -> Result<Vec<CachedSong>, String> {
        let Some(profile_id) = self.server.cache_profile_id().await? else {
            return Ok(Vec::new());
        };
        self.repository.songs(&profile_id, album_id).await
    }
}

fn validate_item_id(item_id: &str) -> Result<&str, String> {
    let item_id = item_id.trim();
    if item_id.is_empty() {
        return Err("Library item ID cannot be empty".to_string());
    }
    Ok(item_id)
}

fn missing_item_error(item_kind: LibraryItemKind, item_id: &str) -> String {
    format!(
        "Cached {} with ID {item_id} was not found",
        item_kind.label()
    )
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        },
    };

    use async_trait::async_trait;
    use sqlx::SqlitePool;
    use uuid::Uuid;

    use super::LibraryCatalogService;
    use crate::{
        credentials::CredentialStore,
        database::{SqliteRepository, DATABASE_FILE_NAME},
        library::{
            models::{
                Album, AlbumWithSongs, Artist, BinaryArtwork, Genre, LibraryItemKind,
                LibrarySnapshot, Song,
            },
            repository::{LibraryCatalogRepository, LibrarySnapshotRepository},
        },
        server::{backend::MusicServer, AlbumQuery, MusicServerService, ScrobbleEvent, ServerInfo},
    };

    #[test]
    fn updates_cache_only_after_remote_annotation_succeeds() {
        tauri::async_runtime::block_on(async {
            let directory = std::env::temp_dir().join(format!("solme-catalog-{}", Uuid::new_v4()));
            let repository = Arc::new(
                SqliteRepository::open(&directory.join(DATABASE_FILE_NAME))
                    .await
                    .unwrap(),
            );
            repository
                .activate_snapshot("profile", "generation-1", None, &snapshot(), 123)
                .await
                .unwrap();

            let remote = Arc::new(MockMusicServer::default());
            let server = Arc::new(MusicServerService::new(
                SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
                Box::new(MemoryCredentialStore),
            ));
            server
                .set_current_server("profile".to_string(), remote.clone())
                .unwrap();
            let catalog = LibraryCatalogService::new(server, repository.clone());

            let annotation = catalog
                .set_favorite(LibraryItemKind::Album, "album-1", true)
                .await
                .unwrap();
            assert!(annotation.favorite);
            assert_eq!(remote.calls.lock().unwrap().as_slice(), ["favorite"]);
            assert!(
                repository
                    .annotation("profile", LibraryItemKind::Album, "album-1")
                    .await
                    .unwrap()
                    .unwrap()
                    .favorite
            );

            remote.fail.store(true, Ordering::SeqCst);
            assert!(catalog
                .set_rating(LibraryItemKind::Album, "album-1", Some(4))
                .await
                .is_err());
            assert_eq!(
                repository
                    .annotation("profile", LibraryItemKind::Album, "album-1")
                    .await
                    .unwrap()
                    .unwrap()
                    .rating,
                None
            );

            let call_count = remote.calls.lock().unwrap().len();
            assert!(catalog
                .set_rating(LibraryItemKind::Album, "album-1", Some(6))
                .await
                .is_err());
            assert_eq!(remote.calls.lock().unwrap().len(), call_count);

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[derive(Default)]
    struct MockMusicServer {
        fail: AtomicBool,
        calls: Mutex<Vec<&'static str>>,
    }

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
        async fn playback_uri(&self, _song_id: &str) -> Result<String, String> {
            unimplemented!()
        }
        async fn scrobble(
            &self,
            _song_id: &str,
            _started_at_ms: i64,
            _event: ScrobbleEvent,
        ) -> Result<(), String> {
            unimplemented!()
        }
        async fn set_favorite(
            &self,
            _item_kind: LibraryItemKind,
            _item_id: &str,
            _favorite: bool,
        ) -> Result<(), String> {
            self.calls.lock().unwrap().push("favorite");
            self.result()
        }
        async fn set_rating(
            &self,
            _item_kind: LibraryItemKind,
            _item_id: &str,
            _rating: Option<i64>,
        ) -> Result<(), String> {
            self.calls.lock().unwrap().push("rating");
            self.result()
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

    impl MockMusicServer {
        fn result(&self) -> Result<(), String> {
            if self.fail.load(Ordering::SeqCst) {
                Err("remote annotation failed".to_string())
            } else {
                Ok(())
            }
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

    fn snapshot() -> LibrarySnapshot {
        LibrarySnapshot {
            artists: vec![Artist {
                remote_id: "artist-1".to_string(),
                name: "Artist".to_string(),
                album_count: 1,
                cover_art_id: None,
                favorite: false,
                rating: None,
            }],
            albums: vec![AlbumWithSongs {
                album: Album {
                    remote_id: "album-1".to_string(),
                    name: "Album".to_string(),
                    album_type: None,
                    artist_id: Some("artist-1".to_string()),
                    artist_name: "Artist".to_string(),
                    year: None,
                    release_date: None,
                    original_release_date: None,
                    server_added_at: None,
                    song_count: 1,
                    duration_seconds: 180,
                    cover_art_id: None,
                    genres: Vec::new(),
                    favorite: false,
                    rating: None,
                },
                songs: vec![Song {
                    remote_id: "song-1".to_string(),
                    album_id: "album-1".to_string(),
                    artist_id: Some("artist-1".to_string()),
                    title: "Song".to_string(),
                    artist_name: "Artist".to_string(),
                    album_name: "Album".to_string(),
                    track_number: Some(1),
                    disc_number: Some(1),
                    year: None,
                    duration_seconds: 180,
                    suffix: None,
                    content_type: None,
                    bit_rate: None,
                    bit_depth: None,
                    sample_rate: None,
                    cover_art_id: None,
                    genres: Vec::new(),
                    favorite: false,
                    rating: None,
                }],
            }],
            genres: Vec::new(),
        }
    }
}
