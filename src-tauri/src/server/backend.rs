use async_trait::async_trait;

use crate::library::models::{
    Album, AlbumWithSongs, Artist, BinaryArtwork, Genre, LibraryItemKind,
};

use super::models::{AlbumQuery, ScrobbleEvent, ServerInfo};

#[async_trait]
pub trait MusicServer: Send + Sync {
    async fn ping(&self) -> Result<ServerInfo, String>;
    async fn library_revision(&self) -> Result<Option<String>, String>;
    async fn artists(&self) -> Result<Vec<Artist>, String>;
    async fn albums(&self, query: AlbumQuery) -> Result<Vec<Album>, String>;
    async fn album(&self, id: &str) -> Result<AlbumWithSongs, String>;
    async fn genres(&self) -> Result<Vec<Genre>, String>;
    async fn playback_uri(&self, song_id: &str) -> Result<String, String>;
    async fn scrobble(
        &self,
        song_id: &str,
        started_at_ms: i64,
        event: ScrobbleEvent,
    ) -> Result<(), String>;
    async fn set_favorite(
        &self,
        item_kind: LibraryItemKind,
        item_id: &str,
        favorite: bool,
    ) -> Result<(), String>;
    async fn set_rating(
        &self,
        item_kind: LibraryItemKind,
        item_id: &str,
        rating: Option<i64>,
    ) -> Result<(), String>;
    async fn album_artwork(&self, cover_art_id: &str) -> Result<Option<BinaryArtwork>, String>;
    async fn artist_artwork(&self, artist_id: &str) -> Result<Option<BinaryArtwork>, String>;
}
