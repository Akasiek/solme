use std::path::Path;

use async_trait::async_trait;

use crate::database::SqliteRepository;

use super::{
    models::{
        ArtworkCacheRecord, ArtworkCandidate, CachedAlbum, CachedSong, LibrarySnapshot,
        LibrarySummary,
    },
    query,
};

#[async_trait]
pub trait LibraryRepository: Send + Sync {
    async fn server_revision(&self, profile_id: &str) -> Result<Option<String>, String>;
    async fn activate_snapshot(
        &self,
        profile_id: &str,
        generation: &str,
        revision: Option<&str>,
        snapshot: &LibrarySnapshot,
        completed_at: i64,
    ) -> Result<(), String>;
    async fn summary(&self, profile_id: &str) -> Result<LibrarySummary, String>;
    async fn albums(
        &self,
        profile_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<CachedAlbum>, String>;
    async fn album(&self, profile_id: &str, album_id: &str) -> Result<Option<CachedAlbum>, String>;
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

impl SqliteRepository {}

#[async_trait]
impl LibraryRepository for SqliteRepository {
    async fn server_revision(&self, profile_id: &str) -> Result<Option<String>, String> {
        sqlx::query_scalar!(
            "SELECT server_revision FROM library_sync_state WHERE profile_id = ?",
            profile_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| format!("Failed to read library revision: {error}"))
        .map(Option::flatten)
    }

    async fn activate_snapshot(
        &self,
        profile_id: &str,
        generation: &str,
        revision: Option<&str>,
        snapshot: &LibrarySnapshot,
        completed_at: i64,
    ) -> Result<(), String> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| format!("Failed to begin library transaction: {error}"))?;

        query::insert_artists(&mut transaction, profile_id, generation, &snapshot.artists).await?;
        query::insert_genres(&mut transaction, profile_id, generation, &snapshot.genres).await?;
        query::insert_albums(&mut transaction, profile_id, generation, &snapshot.albums).await?;
        query::insert_album_genres(&mut transaction, profile_id, generation, &snapshot.albums)
            .await?;
        query::insert_album_search(&mut transaction, profile_id, generation, &snapshot.albums)
            .await?;

        let songs = snapshot
            .albums
            .iter()
            .flat_map(|details| &details.songs)
            .collect::<Vec<_>>();
        query::insert_songs(&mut transaction, profile_id, generation, &songs).await?;
        query::insert_song_genres(&mut transaction, profile_id, generation, &songs).await?;
        query::insert_song_search(&mut transaction, profile_id, generation, &songs).await?;
        let song_count = songs.len() as i64;

        sqlx::query!(
            "INSERT INTO library_sync_state
             (profile_id, active_generation, server_revision, last_success_at,
              artist_count, album_count, song_count)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(profile_id) DO UPDATE SET
               active_generation = excluded.active_generation,
               server_revision = excluded.server_revision,
               last_success_at = excluded.last_success_at,
               artist_count = excluded.artist_count,
               album_count = excluded.album_count,
               song_count = excluded.song_count",
            profile_id,
            generation,
            revision,
            completed_at,
            snapshot.artists.len() as i64,
            snapshot.albums.len() as i64,
            song_count,
        )
        .execute(&mut *transaction)
        .await
        .map_err(|error| format!("Failed to activate library generation: {error}"))?;

        transaction
            .commit()
            .await
            .map_err(|error| format!("Failed to commit library generation: {error}"))?;

        query::delete_stale_generations(self, profile_id, generation).await;

        Ok(())
    }

    async fn summary(&self, profile_id: &str) -> Result<LibrarySummary, String> {
        let summary = sqlx::query_as!(
            LibrarySummary,
            "SELECT artist_count, album_count, song_count, last_success_at
             FROM library_sync_state WHERE profile_id = ?",
            profile_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| format!("Failed to read library summary: {error}"))?;

        Ok(summary.unwrap_or(LibrarySummary {
            artist_count: 0,
            album_count: 0,
            song_count: 0,
            last_success_at: None,
        }))
    }

    async fn albums(
        &self,
        profile_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<CachedAlbum>, String> {
        let limit = limit.clamp(1, 500);
        let offset = offset.max(0);
        sqlx::query_as!(
            CachedAlbum,
            "SELECT a.remote_id, a.name, a.artist_name, a.artist_id, a.year, a.song_count,
                    art.local_path AS artwork_path
             FROM albums a
             JOIN library_sync_state s
               ON s.profile_id = a.profile_id
              AND s.active_generation = a.generation
             LEFT JOIN artwork_cache art
               ON art.profile_id = a.profile_id
              AND art.kind = 'album'
              AND art.remote_id = a.remote_id
             WHERE a.profile_id = ?
             ORDER BY a.artist_name COLLATE NOCASE, a.year, a.name COLLATE NOCASE
             LIMIT ? OFFSET ?",
            profile_id,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| format!("Failed to read cached albums: {error}"))
    }

    async fn album(&self, profile_id: &str, album_id: &str) -> Result<Option<CachedAlbum>, String> {
        sqlx::query_as!(
            CachedAlbum,
            "SELECT a.remote_id, a.name, a.artist_name, a.artist_id, a.year,
                    a.song_count, artwork.local_path AS artwork_path
             FROM albums a
             JOIN library_sync_state state
               ON state.profile_id = a.profile_id
              AND state.active_generation = a.generation
             LEFT JOIN artwork_cache artwork
               ON artwork.profile_id = a.profile_id
              AND artwork.kind = 'album'
              AND artwork.remote_id = a.remote_id
             WHERE a.profile_id = ? AND a.remote_id = ?",
            profile_id,
            album_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| format!("Failed to read cached album: {error}"))
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
        sqlx::query_as!(
            CachedSong,
            "SELECT song.remote_id, song.album_id, song.artist_id, song.title, song.artist_name,
                    song.album_name, artwork.local_path AS artwork_path,
                    song.track_number, song.disc_number, song.duration_seconds
             FROM songs song
             JOIN library_sync_state state
               ON state.profile_id = song.profile_id
              AND state.active_generation = song.generation
             LEFT JOIN artwork_cache artwork
               ON artwork.profile_id = song.profile_id
              AND artwork.kind = 'album'
              AND artwork.remote_id = song.album_id
             WHERE song.profile_id = ? AND song.album_id = ?
             ORDER BY COALESCE(song.disc_number, 1),
                      COALESCE(song.track_number, 2147483647),
                      song.title COLLATE NOCASE",
            profile_id,
            album_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| format!("Failed to read cached songs: {error}"))
    }

    async fn artwork_is_fresh(
        &self,
        profile_id: &str,
        kind: &str,
        remote_id: &str,
        source_key: Option<&str>,
        fresh_after: i64,
    ) -> Result<bool, String> {
        let row = sqlx::query!(
            "SELECT local_path, source_key, downloaded_at
             FROM artwork_cache
             WHERE profile_id = ? AND kind = ? AND remote_id = ?",
            profile_id,
            kind,
            remote_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| format!("Failed to read artwork cache: {error}"))?;

        Ok(row.is_some_and(|row| {
            row.local_path
                .is_some_and(|path| Path::new(&path).is_file())
                && source_key
                    .map(|source_key| row.source_key.as_deref() == Some(source_key))
                    .unwrap_or(true)
                && row.downloaded_at.is_some_and(|time| time >= fresh_after)
        }))
    }

    async fn artwork_candidates(&self, profile_id: &str) -> Result<Vec<ArtworkCandidate>, String> {
        let album_rows = sqlx::query!(
            "SELECT a.remote_id, a.cover_art_id AS \"cover_art_id!: String\"
             FROM albums a
             JOIN library_sync_state s
               ON s.profile_id = a.profile_id
              AND s.active_generation = a.generation
             WHERE a.profile_id = ? AND a.cover_art_id IS NOT NULL",
            profile_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| format!("Failed to read album artwork candidates: {error}"))?;

        let artist_rows = sqlx::query!(
            "SELECT a.remote_id
             FROM artists a
             JOIN library_sync_state s
               ON s.profile_id = a.profile_id
              AND s.active_generation = a.generation
             WHERE a.profile_id = ?",
            profile_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| format!("Failed to read artist artwork candidates: {error}"))?;

        let mut candidates = Vec::with_capacity(album_rows.len() + artist_rows.len());
        candidates.extend(album_rows.into_iter().map(|row| ArtworkCandidate {
            kind: "album",
            remote_id: row.remote_id,
            source_id: row.cover_art_id,
        }));
        candidates.extend(artist_rows.into_iter().map(|row| {
            let remote_id = row.remote_id;
            ArtworkCandidate {
                kind: "artist",
                source_id: remote_id.clone(),
                remote_id,
            }
        }));
        Ok(candidates)
    }

    async fn save_artwork(
        &self,
        profile_id: &str,
        artwork: ArtworkCacheRecord,
    ) -> Result<(), String> {
        sqlx::query!(
            "INSERT INTO artwork_cache
             (profile_id, kind, remote_id, source_key, local_path, content_type,
              etag, last_modified, downloaded_at, last_accessed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(profile_id, kind, remote_id) DO UPDATE SET
               source_key = excluded.source_key,
               local_path = excluded.local_path,
               content_type = excluded.content_type,
               etag = excluded.etag,
               last_modified = excluded.last_modified,
               downloaded_at = excluded.downloaded_at,
               last_accessed_at = excluded.last_accessed_at",
            profile_id,
            artwork.kind,
            artwork.remote_id,
            artwork.source_key,
            artwork.local_path,
            artwork.content_type,
            artwork.etag,
            artwork.last_modified,
            artwork.downloaded_at,
            artwork.downloaded_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|error| format!("Failed to save artwork cache record: {error}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use uuid::Uuid;

    use super::LibraryRepository;
    use crate::database::{SqliteRepository, DATABASE_FILE_NAME};
    use crate::library::models::{Album, AlbumWithSongs, Artist, Genre, LibrarySnapshot, Song};

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
            assert_eq!(repository.albums("profile", 0, 50).await.unwrap().len(), 1);

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
                repository.albums("profile", 0, 500).await.unwrap().len(),
                260
            );

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
                    artist_id: Some("artist-1".to_string()),
                    artist_name: "Artist".to_string(),
                    year: Some(2026),
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
                        artist_id: Some(artist_id.clone()),
                        artist_name: format!("Artist {index}"),
                        year: Some(2026),
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
