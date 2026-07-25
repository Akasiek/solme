use async_trait::async_trait;

use super::{
    models::{
        AlbumSort, ArtworkCacheRecord, ArtworkCandidate, CachedAlbum, CachedSong, LibrarySnapshot,
        LibrarySummary,
    },
    query,
};
use crate::database::SqliteRepository;
use crate::library::models::CachedArtist;

#[async_trait]
pub trait LibraryStateRepository: Send + Sync {
    async fn summary(&self, profile_id: &str) -> Result<LibrarySummary, String>;
}

#[async_trait]
pub trait LibraryCatalogRepository: LibraryStateRepository {
    async fn artist(
        &self,
        profile_id: &str,
        artist_id: &str,
    ) -> Result<Option<CachedArtist>, String>;
    async fn search_artists(
        &self,
        profile_id: &str,
        query: &str,
        limit: i64,
    ) -> Result<Vec<CachedArtist>, String>;
    async fn artist_albums(
        &self,
        profile_id: &str,
        artist_id: &str,
    ) -> Result<Vec<CachedAlbum>, String>;
    async fn albums(
        &self,
        profile_id: &str,
        offset: i64,
        limit: i64,
        sort: AlbumSort,
    ) -> Result<Vec<CachedAlbum>, String>;
    async fn albums_by_ids(
        &self,
        profile_id: &str,
        album_ids: &[String],
    ) -> Result<Vec<CachedAlbum>, String>;
    async fn album(&self, profile_id: &str, album_id: &str) -> Result<Option<CachedAlbum>, String>;
    async fn album_genres(&self, profile_id: &str, album_id: &str) -> Result<Vec<String>, String>;
    async fn album_disc_count(&self, profile_id: &str, album_id: &str) -> Result<i64, String>;
    async fn album_audio_formats(
        &self,
        profile_id: &str,
        album_id: &str,
    ) -> Result<Vec<String>, String>;
    async fn search_albums(
        &self,
        profile_id: &str,
        query: &str,
        limit: i64,
    ) -> Result<Vec<CachedAlbum>, String>;
    async fn search_songs(
        &self,
        profile_id: &str,
        query: &str,
        limit: i64,
    ) -> Result<Vec<CachedSong>, String>;
    async fn songs(&self, profile_id: &str, album_id: &str) -> Result<Vec<CachedSong>, String>;
}

#[async_trait]
pub trait LibrarySnapshotRepository: LibraryStateRepository {
    async fn server_revision(&self, profile_id: &str) -> Result<Option<String>, String>;
    async fn activate_snapshot(
        &self,
        profile_id: &str,
        generation: &str,
        revision: Option<&str>,
        snapshot: &LibrarySnapshot,
        completed_at: i64,
    ) -> Result<(), String>;
}

#[async_trait]
pub trait ArtworkRepository: Send + Sync {
    async fn artwork_is_fresh(
        &self,
        profile_id: &str,
        kind: &str,
        remote_id: &str,
        source_key: Option<&str>,
        fresh_after: i64,
    ) -> Result<bool, String>;
    async fn artwork_candidates(&self, profile_id: &str) -> Result<Vec<ArtworkCandidate>, String>;
    async fn save_artwork(
        &self,
        profile_id: &str,
        artwork: ArtworkCacheRecord,
    ) -> Result<(), String>;
}

pub trait LibrarySyncRepository: LibrarySnapshotRepository + ArtworkRepository {}

impl<T> LibrarySyncRepository for T where T: LibrarySnapshotRepository + ArtworkRepository {}

#[async_trait]
impl LibraryStateRepository for SqliteRepository {
    async fn summary(&self, profile_id: &str) -> Result<LibrarySummary, String> {
        query::summary(self, profile_id).await
    }
}

#[async_trait]
impl LibrarySnapshotRepository for SqliteRepository {
    async fn server_revision(&self, profile_id: &str) -> Result<Option<String>, String> {
        query::server_revision(self, profile_id).await
    }

    async fn activate_snapshot(
        &self,
        profile_id: &str,
        generation: &str,
        revision: Option<&str>,
        snapshot: &LibrarySnapshot,
        completed_at: i64,
    ) -> Result<(), String> {
        query::activate_snapshot(
            self,
            profile_id,
            generation,
            revision,
            snapshot,
            completed_at,
        )
        .await
    }
}

#[async_trait]
impl LibraryCatalogRepository for SqliteRepository {
    async fn artist(
        &self,
        profile_id: &str,
        artist_id: &str,
    ) -> Result<Option<CachedArtist>, String> {
        query::artist(self, profile_id, artist_id).await
    }

    async fn search_artists(
        &self,
        profile_id: &str,
        query: &str,
        limit: i64,
    ) -> Result<Vec<CachedArtist>, String> {
        query::search_artists(self, profile_id, query, limit).await
    }

    async fn artist_albums(
        &self,
        profile_id: &str,
        artist_id: &str,
    ) -> Result<Vec<CachedAlbum>, String> {
        query::artist_albums(self, profile_id, artist_id).await
    }

    async fn albums(
        &self,
        profile_id: &str,
        offset: i64,
        limit: i64,
        sort: AlbumSort,
    ) -> Result<Vec<CachedAlbum>, String> {
        query::albums(self, profile_id, offset, limit, sort).await
    }

    async fn albums_by_ids(
        &self,
        profile_id: &str,
        album_ids: &[String],
    ) -> Result<Vec<CachedAlbum>, String> {
        query::albums_by_ids(self, profile_id, album_ids).await
    }

    async fn album(&self, profile_id: &str, album_id: &str) -> Result<Option<CachedAlbum>, String> {
        query::album(self, profile_id, album_id).await
    }

    async fn album_genres(&self, profile_id: &str, album_id: &str) -> Result<Vec<String>, String> {
        query::album_genres(self, profile_id, album_id).await
    }

    async fn album_disc_count(&self, profile_id: &str, album_id: &str) -> Result<i64, String> {
        query::album_disc_count(self, profile_id, album_id).await
    }

    async fn album_audio_formats(
        &self,
        profile_id: &str,
        album_id: &str,
    ) -> Result<Vec<String>, String> {
        query::album_audio_formats(self, profile_id, album_id).await
    }

    async fn search_albums(
        &self,
        profile_id: &str,
        query: &str,
        limit: i64,
    ) -> Result<Vec<CachedAlbum>, String> {
        query::search_albums(self, profile_id, query, limit).await
    }

    async fn search_songs(
        &self,
        profile_id: &str,
        query: &str,
        limit: i64,
    ) -> Result<Vec<CachedSong>, String> {
        query::search_songs(self, profile_id, query, limit).await
    }

    async fn songs(&self, profile_id: &str, album_id: &str) -> Result<Vec<CachedSong>, String> {
        query::songs(self, profile_id, album_id).await
    }
}

#[async_trait]
impl ArtworkRepository for SqliteRepository {
    async fn artwork_is_fresh(
        &self,
        profile_id: &str,
        kind: &str,
        remote_id: &str,
        source_key: Option<&str>,
        fresh_after: i64,
    ) -> Result<bool, String> {
        query::artwork_is_fresh(self, profile_id, kind, remote_id, source_key, fresh_after).await
    }

    async fn artwork_candidates(&self, profile_id: &str) -> Result<Vec<ArtworkCandidate>, String> {
        query::artwork_candidates(self, profile_id).await
    }

    async fn save_artwork(
        &self,
        profile_id: &str,
        artwork: ArtworkCacheRecord,
    ) -> Result<(), String> {
        query::save_artwork(self, profile_id, artwork).await
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use uuid::Uuid;

    use super::{
        ArtworkRepository, LibraryCatalogRepository, LibrarySnapshotRepository,
        LibraryStateRepository,
    };
    use crate::database::{SqliteRepository, DATABASE_FILE_NAME};
    use crate::library::models::{
        Album, AlbumSort, AlbumWithSongs, Artist, ArtworkCacheRecord, Genre, LibrarySnapshot, Song,
    };
    #[test]
    fn activates_complete_generation() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            repository
                .activate_snapshot(
                    "profile",
                    "generation-1",
                    Some("revision-1"),
                    &snapshot(false),
                    123,
                )
                .await
                .unwrap();

            let summary = repository.summary("profile").await.unwrap();
            assert_eq!(summary.artist_count, 1);
            assert_eq!(summary.album_count, 1);
            assert_eq!(summary.song_count, 1);
            assert_eq!(summary.last_success_at, Some(123));
            assert_eq!(
                repository.server_revision("profile").await.unwrap(),
                Some("revision-1".to_string())
            );
            let albums = repository
                .albums("profile", 0, 50, AlbumSort::Artist)
                .await
                .unwrap();
            assert_eq!(albums.len(), 1);
            assert_eq!(
                albums[0].original_release_date.as_deref(),
                Some("2025-12-31")
            );
            assert_eq!(
                repository.album_genres("profile", "album-1").await.unwrap(),
                vec!["Jazz".to_string()]
            );

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn returns_cached_albums_in_requested_id_order() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            repository
                .activate_snapshot("profile", "generation-1", None, &large_snapshot(3), 123)
                .await
                .unwrap();

            let requested_ids = vec![
                "album-2".to_string(),
                "missing-album".to_string(),
                "album-0".to_string(),
            ];
            let albums = repository
                .albums_by_ids("profile", &requested_ids)
                .await
                .unwrap();
            let returned_ids = albums
                .iter()
                .map(|album| album.remote_id.as_str())
                .collect::<Vec<_>>();

            assert_eq!(returned_ids, vec!["album-2", "album-0"]);

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn returns_cached_artist_from_active_generation() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;

            repository
                .activate_snapshot("profile", "generation-1", None, &snapshot(false), 123)
                .await
                .unwrap();

            let artist = repository
                .artist("profile", "artist-1")
                .await
                .unwrap()
                .expect("artist should be cached");
            assert_eq!(artist.remote_id, "artist-1");
            assert_eq!(artist.name, "Artist");
            assert_eq!(artist.album_count, 1);
            assert_eq!(artist.artwork_path, None);

            assert!(repository
                .artist("profile", "missing-artist")
                .await
                .unwrap()
                .is_none());
            assert!(repository
                .artist("other-profile", "artist-1")
                .await
                .unwrap()
                .is_none());

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn searches_artists_by_name_with_prefix_and_fuzzy_fallback() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            let mut snapshot = snapshot(false);
            snapshot.artists[0].name = "Nirvana".to_string();

            repository
                .activate_snapshot("profile", "generation-1", None, &snapshot, 123)
                .await
                .unwrap();

            let by_prefix = repository
                .search_artists("profile", "nir", 20)
                .await
                .unwrap();
            let by_typo = repository
                .search_artists("profile", "nibana", 20)
                .await
                .unwrap();

            assert_eq!(by_prefix.len(), 1);
            assert_eq!(by_prefix[0].remote_id, "artist-1");
            assert_eq!(by_typo.len(), 1);
            assert_eq!(by_typo[0].remote_id, "artist-1");

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn artist_search_uses_only_the_active_generation() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            let mut old_snapshot = snapshot(false);
            old_snapshot.artists[0].name = "Old Artist".to_string();
            repository
                .activate_snapshot("profile", "generation-1", None, &old_snapshot, 123)
                .await
                .unwrap();

            let mut new_snapshot = snapshot(false);
            new_snapshot.artists[0].name = "New Artist".to_string();
            repository
                .activate_snapshot("profile", "generation-2", None, &new_snapshot, 124)
                .await
                .unwrap();

            assert!(repository
                .search_artists("profile", "old", 20)
                .await
                .unwrap()
                .is_empty());
            assert_eq!(
                repository
                    .search_artists("profile", "new", 20)
                    .await
                    .unwrap()
                    .len(),
                1
            );

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn returns_cached_artist_with_artwork_path() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;

            repository
                .activate_snapshot("profile", "generation-1", None, &snapshot(false), 123)
                .await
                .unwrap();
            repository
                .save_artwork(
                    "profile",
                    ArtworkCacheRecord {
                        kind: "artist",
                        remote_id: "artist-1".to_string(),
                        source_key: "artist-1".to_string(),
                        local_path: "/tmp/artist-1.webp".to_string(),
                        content_type: "image/webp".to_string(),
                        etag: None,
                        last_modified: None,
                        downloaded_at: 123,
                    },
                )
                .await
                .unwrap();

            let artist = repository
                .artist("profile", "artist-1")
                .await
                .unwrap()
                .expect("artist should be cached");
            assert_eq!(artist.artwork_path.as_deref(), Some("/tmp/artist-1.webp"));

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn returns_cached_artist_albums_from_active_generation() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            let mut snapshot = snapshot(false);
            snapshot.albums[0].album.album_type = Some("Live".to_string());
            snapshot.artists.push(Artist {
                remote_id: "artist-2".to_string(),
                name: "Other Artist".to_string(),
                album_count: 1,
            });
            snapshot.albums.push(AlbumWithSongs {
                album: Album {
                    remote_id: "album-2".to_string(),
                    name: "Other Album".to_string(),
                    album_type: None,
                    artist_id: Some("artist-2".to_string()),
                    artist_name: "Other Artist".to_string(),
                    year: Some(2025),
                    release_date: Some("2025-01-01".to_string()),
                    original_release_date: None,
                    server_added_at: Some("2026-01-03T00:00:00Z".to_string()),
                    song_count: 0,
                    duration_seconds: 0,
                    cover_art_id: Some("cover-2".to_string()),
                    genres: vec!["Rock".to_string()],
                },
                songs: Vec::new(),
            });

            repository
                .activate_snapshot("profile", "generation-1", None, &snapshot, 123)
                .await
                .unwrap();

            let albums = repository
                .artist_albums("profile", "artist-1")
                .await
                .unwrap();
            assert_eq!(albums.len(), 1);
            assert_eq!(albums[0].remote_id, "album-1");
            assert_eq!(albums[0].album_type.as_deref(), Some("Live"));

            let missing = repository
                .artist_albums("profile", "missing-artist")
                .await
                .unwrap();
            assert!(missing.is_empty());

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn cached_artist_uses_latest_active_generation() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;

            repository
                .activate_snapshot("profile", "generation-1", None, &snapshot(false), 123)
                .await
                .unwrap();

            let mut next_snapshot = snapshot(false);
            next_snapshot.artists[0].name = "Updated Artist".to_string();
            next_snapshot.artists[0].album_count = 2;
            next_snapshot.albums[0].album.artist_name = "Updated Artist".to_string();
            next_snapshot.albums[0].songs[0].artist_name = "Updated Artist".to_string();
            repository
                .activate_snapshot("profile", "generation-2", None, &next_snapshot, 456)
                .await
                .unwrap();

            let artist = repository
                .artist("profile", "artist-1")
                .await
                .unwrap()
                .expect("artist should be cached");
            assert_eq!(artist.name, "Updated Artist");
            assert_eq!(artist.album_count, 2);

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn failed_generation_keeps_previous_cache_active() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            repository
                .activate_snapshot(
                    "profile",
                    "generation-1",
                    Some("revision-1"),
                    &snapshot(false),
                    123,
                )
                .await
                .unwrap();

            let result = repository
                .activate_snapshot(
                    "profile",
                    "generation-2",
                    Some("revision-2"),
                    &snapshot(true),
                    456,
                )
                .await;
            assert!(result.is_err());

            let summary = repository.summary("profile").await.unwrap();
            assert_eq!(summary.album_count, 1);
            assert_eq!(summary.song_count, 1);
            assert_eq!(summary.last_success_at, Some(123));
            assert_eq!(
                repository.server_revision("profile").await.unwrap(),
                Some("revision-1".to_string())
            );

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn inserts_snapshot_in_multiple_batches() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            let snapshot = large_snapshot(260);

            repository
                .activate_snapshot("profile", "generation-1", None, &snapshot, 123)
                .await
                .unwrap();

            let summary = repository.summary("profile").await.unwrap();
            assert_eq!(summary.artist_count, 260);
            assert_eq!(summary.album_count, 260);
            assert_eq!(summary.song_count, 260);
            assert_eq!(
                repository
                    .albums("profile", 0, 500, AlbumSort::Artist)
                    .await
                    .unwrap()
                    .len(),
                260
            );

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn recently_released_sort_prefers_original_release_date_before_release_date() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            let mut snapshot = snapshot(false);
            snapshot.albums = vec![
                album_with_dates("album-1", "Release-only album", None, Some("2024-01-01")),
                album_with_dates(
                    "album-2",
                    "Original-date album",
                    Some("2023-01-01"),
                    Some("2025-01-01"),
                ),
                album_with_dates(
                    "album-3",
                    "Old original-date album",
                    Some("2020-01-01"),
                    Some("2026-01-01"),
                ),
            ];

            repository
                .activate_snapshot("profile", "generation-1", None, &snapshot, 123)
                .await
                .unwrap();

            let albums = repository
                .albums("profile", 0, 50, AlbumSort::RecentlyReleased)
                .await
                .unwrap();
            let ids = albums
                .iter()
                .map(|album| album.remote_id.as_str())
                .collect::<Vec<_>>();
            assert_eq!(ids, vec!["album-1", "album-2", "album-3"]);

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn returns_cached_songs_in_disc_and_track_order() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            let mut snapshot = snapshot(false);
            let album = &mut snapshot.albums[0];
            album.songs = vec![
                song("song-3", Some(1), Some(2)),
                song("song-1", Some(1), Some(1)),
                song("song-4", Some(2), Some(2)),
                song("song-2", Some(2), Some(1)),
            ];
            album.album.song_count = album.songs.len() as i64;

            repository
                .activate_snapshot("profile", "generation-1", None, &snapshot, 123)
                .await
                .unwrap();

            let songs = repository.songs("profile", "album-1").await.unwrap();
            let ids = songs
                .into_iter()
                .map(|song| song.remote_id)
                .collect::<Vec<_>>();
            assert_eq!(ids, ["song-1", "song-2", "song-3", "song-4"]);

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn searches_active_albums_by_album_or_artist_name() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            let mut snapshot = snapshot(false);
            snapshot.albums[0].album.name = "Kind of Blue".to_string();
            snapshot.albums[0].album.artist_name = "Miles Davis".to_string();
            snapshot.albums[0].album.genres = vec!["Modal Jazz".to_string()];

            repository
                .activate_snapshot("profile", "generation-1", None, &snapshot, 123)
                .await
                .unwrap();

            let by_album = repository
                .search_albums("profile", "kind", 20)
                .await
                .unwrap();
            let by_artist = repository
                .search_albums("profile", "MILES", 20)
                .await
                .unwrap();
            let by_genre = repository
                .search_albums("profile", "modal", 20)
                .await
                .unwrap();
            let by_prefix = repository
                .search_albums("profile", "mil", 20)
                .await
                .unwrap();
            assert_eq!(by_album.len(), 1);
            assert_eq!(by_artist.len(), 1);
            assert_eq!(by_genre.len(), 1);
            assert_eq!(by_prefix.len(), 1);
            assert_eq!(by_album[0].remote_id, "album-1");
            assert_eq!(
                repository
                    .album("profile", "album-1")
                    .await
                    .unwrap()
                    .map(|album| album.name)
                    .as_deref(),
                Some("Kind of Blue")
            );
            assert_eq!(
                repository.album_genres("profile", "album-1").await.unwrap(),
                vec!["Modal Jazz".to_string()]
            );

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn album_search_uses_fuzzy_fallback_for_typos() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            let mut snapshot = snapshot(false);
            snapshot.albums[0].album.name = "Nevermind".to_string();
            snapshot.albums[0].album.artist_name = "Nirvana".to_string();

            repository
                .activate_snapshot("profile", "generation-1", None, &snapshot, 123)
                .await
                .unwrap();

            let by_artist_typo = repository
                .search_albums("profile", "nibana", 20)
                .await
                .unwrap();

            assert_eq!(by_artist_typo.len(), 1);
            assert_eq!(by_artist_typo[0].remote_id, "album-1");

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn searches_active_songs_by_title_artist_album_or_genre() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            let mut snapshot = snapshot(false);
            snapshot.albums[0].album.name = "Kind of Blue".to_string();
            snapshot.albums[0].album.artist_name = "Miles Davis".to_string();
            snapshot.albums[0].songs = vec![
                Song {
                    remote_id: "song-1".to_string(),
                    album_id: "album-1".to_string(),
                    artist_id: Some("artist-1".to_string()),
                    title: "So What".to_string(),
                    artist_name: "Miles Davis".to_string(),
                    album_name: "Kind of Blue".to_string(),
                    track_number: Some(1),
                    disc_number: Some(1),
                    year: Some(1959),
                    duration_seconds: 545,
                    suffix: None,
                    content_type: None,
                    bit_rate: None,
                    bit_depth: None,
                    sample_rate: None,
                    cover_art_id: Some("cover-1".to_string()),
                    genres: vec!["Modal Jazz".to_string()],
                },
                Song {
                    remote_id: "song-2".to_string(),
                    album_id: "album-1".to_string(),
                    artist_id: Some("artist-1".to_string()),
                    title: "Freddie Freeloader".to_string(),
                    artist_name: "Miles Davis".to_string(),
                    album_name: "Kind of Blue".to_string(),
                    track_number: Some(2),
                    disc_number: Some(1),
                    year: Some(1959),
                    duration_seconds: 589,
                    suffix: None,
                    content_type: None,
                    bit_rate: None,
                    bit_depth: None,
                    sample_rate: None,
                    cover_art_id: Some("cover-1".to_string()),
                    genres: vec!["Blues".to_string()],
                },
            ];
            snapshot.albums[0].album.song_count = snapshot.albums[0].songs.len() as i64;

            repository
                .activate_snapshot("profile", "generation-1", None, &snapshot, 123)
                .await
                .unwrap();

            let by_title = repository
                .search_songs("profile", "fredd", 20)
                .await
                .unwrap();
            let by_artist = repository
                .search_songs("profile", "miles", 20)
                .await
                .unwrap();
            let by_album = repository
                .search_songs("profile", "blue", 20)
                .await
                .unwrap();
            let by_genre = repository
                .search_songs("profile", "modal", 20)
                .await
                .unwrap();

            assert_eq!(by_title.len(), 1);
            assert_eq!(by_title[0].remote_id, "song-2");
            assert_eq!(by_artist.len(), 2);
            assert_eq!(by_album.len(), 2);
            assert_eq!(by_genre.len(), 1);
            assert_eq!(by_genre[0].remote_id, "song-1");

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn song_search_uses_fuzzy_fallback_for_typos() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            let mut snapshot = snapshot(false);
            snapshot.albums[0].album.name = "Nevermind".to_string();
            snapshot.albums[0].album.artist_name = "Nirvana".to_string();
            snapshot.albums[0].songs[0].title = "Smells Like Teen Spirit".to_string();
            snapshot.albums[0].songs[0].artist_name = "Nirvana".to_string();
            snapshot.albums[0].songs[0].album_name = "Nevermind".to_string();

            repository
                .activate_snapshot("profile", "generation-1", None, &snapshot, 123)
                .await
                .unwrap();

            let by_title_typo = repository
                .search_songs("profile", "smels", 20)
                .await
                .unwrap();

            assert_eq!(by_title_typo.len(), 1);
            assert_eq!(by_title_typo[0].remote_id, "song-1");

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn searches_only_active_generation() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            let mut old_snapshot = snapshot(false);
            old_snapshot.albums[0].album.name = "Old Album".to_string();
            old_snapshot.albums[0].songs[0].title = "Old Song".to_string();

            repository
                .activate_snapshot("profile", "generation-1", None, &old_snapshot, 123)
                .await
                .unwrap();

            let mut new_snapshot = snapshot(false);
            new_snapshot.albums[0].album.name = "New Album".to_string();
            new_snapshot.albums[0].songs[0].title = "New Song".to_string();

            repository
                .activate_snapshot("profile", "generation-2", None, &new_snapshot, 124)
                .await
                .unwrap();

            assert!(repository
                .search_albums("profile", "old", 20)
                .await
                .unwrap()
                .is_empty());
            assert!(repository
                .search_songs("profile", "old", 20)
                .await
                .unwrap()
                .is_empty());
            assert_eq!(
                repository
                    .search_albums("profile", "new", 20)
                    .await
                    .unwrap()
                    .len(),
                1
            );
            assert_eq!(
                repository
                    .search_songs("profile", "new", 20)
                    .await
                    .unwrap()
                    .len(),
                1
            );

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn rejects_snapshot_with_missing_album_artist_reference() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            let mut snapshot = snapshot(false);
            snapshot.artists.clear();

            let result = repository
                .activate_snapshot("profile", "generation-1", None, &snapshot, 123)
                .await;

            assert!(result.is_err());

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn allows_song_artist_without_matching_artist_row() {
        tauri::async_runtime::block_on(async {
            let (repository, directory) = repository().await;
            let mut snapshot = snapshot(false);
            snapshot.albums[0].songs[0].artist_id = Some("guest-artist".to_string());

            repository
                .activate_snapshot("profile", "generation-1", None, &snapshot, 123)
                .await
                .unwrap();

            let songs = repository.songs("profile", "album-1").await.unwrap();
            assert_eq!(songs.len(), 1);
            assert_eq!(songs[0].artist_name, "Artist");

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    async fn repository() -> (SqliteRepository, std::path::PathBuf) {
        let directory = std::env::temp_dir().join(format!("solme-library-{}", Uuid::new_v4()));
        let repository = SqliteRepository::open(&directory.join(DATABASE_FILE_NAME))
            .await
            .unwrap();
        (repository, directory)
    }

    fn snapshot(duplicate_song: bool) -> LibrarySnapshot {
        let song = song("song-1", Some(1), Some(1));
        let mut songs = vec![song.clone()];
        if duplicate_song {
            songs.push(song);
        }

        LibrarySnapshot {
            artists: vec![Artist {
                remote_id: "artist-1".to_string(),
                name: "Artist".to_string(),
                album_count: 1,
            }],
            albums: vec![AlbumWithSongs {
                album: Album {
                    remote_id: "album-1".to_string(),
                    name: "Album".to_string(),
                    album_type: None,
                    artist_id: Some("artist-1".to_string()),
                    artist_name: "Artist".to_string(),
                    year: Some(2026),
                    release_date: Some("2026-01-01".to_string()),
                    original_release_date: Some("2025-12-31".to_string()),
                    server_added_at: Some("2026-01-02T00:00:00Z".to_string()),
                    song_count: songs.len() as i64,
                    duration_seconds: 180,
                    cover_art_id: Some("cover-1".to_string()),
                    genres: vec!["Jazz".to_string()],
                },
                songs,
            }],
            genres: vec![Genre {
                name: "Jazz".to_string(),
                song_count: 1,
                album_count: 1,
            }],
        }
    }

    fn album_with_dates(
        remote_id: &str,
        name: &str,
        original_release_date: Option<&str>,
        release_date: Option<&str>,
    ) -> AlbumWithSongs {
        AlbumWithSongs {
            album: Album {
                remote_id: remote_id.to_string(),
                name: name.to_string(),
                album_type: None,
                artist_id: Some("artist-1".to_string()),
                artist_name: "Artist".to_string(),
                year: Some(2026),
                release_date: release_date.map(str::to_string),
                original_release_date: original_release_date.map(str::to_string),
                server_added_at: Some("2026-01-02T00:00:00Z".to_string()),
                song_count: 1,
                duration_seconds: 180,
                cover_art_id: Some(format!("cover-{remote_id}")),
                genres: vec!["Jazz".to_string()],
            },
            songs: vec![Song {
                remote_id: format!("song-{remote_id}"),
                album_id: remote_id.to_string(),
                artist_id: Some("artist-1".to_string()),
                title: format!("Song {remote_id}"),
                artist_name: "Artist".to_string(),
                album_name: name.to_string(),
                track_number: Some(1),
                disc_number: Some(1),
                year: Some(2026),
                duration_seconds: 180,
                suffix: Some("opus".to_string()),
                content_type: Some("audio/ogg".to_string()),
                bit_rate: Some(256),
                bit_depth: Some(24),
                sample_rate: Some(48000),
                cover_art_id: Some(format!("cover-{remote_id}")),
                genres: vec!["Jazz".to_string()],
            }],
        }
    }

    fn song(remote_id: &str, track_number: Option<i64>, disc_number: Option<i64>) -> Song {
        Song {
            remote_id: remote_id.to_string(),
            album_id: "album-1".to_string(),
            artist_id: Some("artist-1".to_string()),
            title: remote_id.to_string(),
            artist_name: "Artist".to_string(),
            album_name: "Album".to_string(),
            track_number,
            disc_number,
            year: Some(2026),
            duration_seconds: 180,
            suffix: Some("opus".to_string()),
            content_type: Some("audio/ogg".to_string()),
            bit_rate: Some(256),
            bit_depth: Some(24),
            sample_rate: Some(48000),
            cover_art_id: Some("cover-1".to_string()),
            genres: vec!["Jazz".to_string()],
        }
    }

    fn large_snapshot(count: usize) -> LibrarySnapshot {
        let artists = (0..count)
            .map(|index| Artist {
                remote_id: format!("artist-{index}"),
                name: format!("Artist {index}"),
                album_count: 1,
            })
            .collect();
        let albums = (0..count)
            .map(|index| {
                let artist_id = format!("artist-{index}");
                let album_id = format!("album-{index}");
                AlbumWithSongs {
                    album: Album {
                        remote_id: album_id.clone(),
                        name: format!("Album {index}"),
                        album_type: None,
                        artist_id: Some(artist_id.clone()),
                        artist_name: format!("Artist {index}"),
                        year: Some(2026),
                        release_date: Some("2026-01-01".to_string()),
                        original_release_date: Some("2025-12-31".to_string()),
                        server_added_at: Some("2026-01-02T00:00:00Z".to_string()),
                        song_count: 1,
                        duration_seconds: 180,
                        cover_art_id: Some(format!("cover-{index}")),
                        genres: vec!["Jazz".to_string()],
                    },
                    songs: vec![Song {
                        remote_id: format!("song-{index}"),
                        album_id,
                        artist_id: Some(artist_id),
                        title: format!("Song {index}"),
                        artist_name: format!("Artist {index}"),
                        album_name: format!("Album {index}"),
                        track_number: Some(1),
                        disc_number: Some(1),
                        year: Some(2026),
                        duration_seconds: 180,
                        suffix: Some("opus".to_string()),
                        content_type: Some("audio/ogg".to_string()),
                        bit_rate: Some(256),
                        bit_depth: Some(24),
                        sample_rate: Some(48000),
                        cover_art_id: Some(format!("cover-{index}")),
                        genres: vec!["Jazz".to_string()],
                    }],
                }
            })
            .collect();

        LibrarySnapshot {
            artists,
            albums,
            genres: vec![Genre {
                name: "Jazz".to_string(),
                song_count: count as i64,
                album_count: count as i64,
            }],
        }
    }
}
