use std::sync::Arc;

use futures_util::future::join;

use crate::server::{AlbumQuery, MusicServerService};

use super::{
    models::{
        Album, AlbumSort, CachedAlbum, CachedAlbumDetails, CachedArtist, CachedArtistDetails,
        CachedSong, HomeAlbumSections, LibrarySummary,
    },
    repository::LibraryRepository,
};

pub struct LibraryCatalogService {
    server: Arc<MusicServerService>,
    repository: Arc<dyn LibraryRepository>,
}

impl LibraryCatalogService {
    pub fn new(server: Arc<MusicServerService>, repository: Arc<dyn LibraryRepository>) -> Self {
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
