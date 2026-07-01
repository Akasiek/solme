use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Artist {
    pub remote_id: String,
    pub name: String,
    pub album_count: i64,
}

#[derive(Clone)]
pub struct Album {
    pub remote_id: String,
    pub name: String,
    pub artist_id: Option<String>,
    pub artist_name: String,
    pub year: Option<i64>,
    pub song_count: i64,
    pub duration_seconds: i64,
    pub cover_art_id: Option<String>,
    pub genres: Vec<String>,
}

#[derive(Clone)]
pub struct Song {
    pub remote_id: String,
    pub album_id: String,
    pub artist_id: Option<String>,
    pub title: String,
    pub artist_name: String,
    pub album_name: String,
    pub track_number: Option<i64>,
    pub disc_number: Option<i64>,
    pub year: Option<i64>,
    pub duration_seconds: i64,
    pub suffix: Option<String>,
    pub content_type: Option<String>,
    pub cover_art_id: Option<String>,
    pub genres: Vec<String>,
}

#[derive(Clone)]
pub struct AlbumWithSongs {
    pub album: Album,
    pub songs: Vec<Song>,
}

#[derive(Clone)]
pub struct Genre {
    pub name: String,
    pub song_count: i64,
    pub album_count: i64,
}

pub struct LibrarySnapshot {
    pub artists: Vec<Artist>,
    pub albums: Vec<AlbumWithSongs>,
    pub genres: Vec<Genre>,
}

#[derive(Clone)]
pub struct BinaryArtwork {
    pub source_key: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Clone)]
pub struct ArtworkCandidate {
    pub kind: &'static str,
    pub remote_id: String,
    pub source_id: String,
}

pub struct ArtworkCacheRecord {
    pub kind: &'static str,
    pub remote_id: String,
    pub source_key: String,
    pub local_path: String,
    pub content_type: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub downloaded_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySummary {
    pub artist_count: i64,
    pub album_count: i64,
    pub song_count: i64,
    pub last_success_at: Option<i64>,
}

#[derive(Clone, Debug, sqlx::FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedAlbum {
    pub remote_id: String,
    pub name: String,
    pub artist_name: String,
    pub artist_id: Option<String>,
    pub year: Option<i64>,
    pub song_count: i64,
    pub artwork_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, sqlx::FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedSong {
    pub remote_id: String,
    pub album_id: String,
    pub artist_id: Option<String>,
    pub title: String,
    pub artist_name: String,
    pub album_name: String,
    pub artwork_path: Option<String>,
    pub track_number: Option<i64>,
    pub disc_number: Option<i64>,
    pub duration_seconds: i64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LibrarySyncPhase {
    #[default]
    Idle,
    Metadata,
    Activating,
    Artwork,
    Completed,
    Failed,
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySyncStatus {
    pub phase: LibrarySyncPhase,
    pub processed_artists: u64,
    pub processed_albums: u64,
    pub processed_songs: u64,
    pub processed_artwork: u64,
    pub total_artwork: u64,
    pub last_success_at: Option<i64>,
    pub last_error: Option<String>,
}
