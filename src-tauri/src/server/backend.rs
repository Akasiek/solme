use async_trait::async_trait;

use crate::library::models::{Album, AlbumWithSongs, Artist, BinaryArtwork, Genre};

use super::models::ServerInfo;

#[async_trait]
pub trait MusicServer: Send + Sync {
    async fn ping(&self) -> Result<ServerInfo, String>;
    async fn library_revision(&self) -> Result<Option<String>, String>;
    async fn artists(&self) -> Result<Vec<Artist>, String>;
    async fn albums(&self) -> Result<Vec<Album>, String>;
    async fn album(&self, id: &str) -> Result<AlbumWithSongs, String>;
    async fn genres(&self) -> Result<Vec<Genre>, String>;
    async fn album_artwork(&self, cover_art_id: &str) -> Result<Option<BinaryArtwork>, String>;
    async fn artist_artwork(&self, artist_id: &str) -> Result<Option<BinaryArtwork>, String>;
}
