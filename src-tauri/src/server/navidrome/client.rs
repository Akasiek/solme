use std::time::Duration;

use async_trait::async_trait;
use rand::random;
use reqwest::{
    header::{CONTENT_TYPE, ETAG, LAST_MODIFIED},
    Client, Response, Url,
};
use serde::de::DeserializeOwned;

use crate::library::models::{Album, AlbumWithSongs, Artist, BinaryArtwork, Genre};

use super::models::{
    AlbumDto, AlbumListPayload, AlbumPayload, ArtistInfoPayload, ArtistsPayload, GenresPayload,
    PingPayload, ScanStatusPayload, SubsonicEnvelope,
};
use crate::server::{backend::MusicServer, models::ServerInfo};

const API_VERSION: &str = "1.16.1";
const CLIENT_NAME: &str = "solme";
const ALBUM_PAGE_SIZE: u32 = 500;

pub struct NavidromeBackend {
    client: Client,
    base_url: Url,
    username: String,
    password: String,
}

impl NavidromeBackend {
    pub fn new(url: String, username: String, password: String) -> Result<Self, String> {
        let base_url =
            Url::parse(url.trim()).map_err(|error| format!("Invalid server URL: {error}"))?;

        if !matches!(base_url.scheme(), "http" | "https") {
            return Err("Server URL must use HTTP or HTTPS".to_string());
        }
        if username.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|error| format!("Failed to create HTTP client: {error}"))?;

        Ok(Self {
            client,
            base_url,
            username,
            password,
        })
    }

    fn endpoint_url(&self, endpoint: &str) -> Url {
        let mut url = self.base_url.clone();
        let base_path = url.path().trim_end_matches('/');
        url.set_path(&format!("{base_path}/rest/{endpoint}.view"));
        url.set_query(None);
        url.set_fragment(None);
        url
    }

    fn auth_query(&self) -> Vec<(&'static str, String)> {
        let salt = format!("{:016x}", random::<u64>());
        let token = format!("{:x}", md5::compute(format!("{}{}", self.password, salt)));

        vec![
            ("u", self.username.clone()),
            ("t", token),
            ("s", salt),
            ("v", API_VERSION.to_string()),
            ("c", CLIENT_NAME.to_string()),
            ("f", "json".to_string()),
        ]
    }

    async fn request_json<T>(
        &self,
        endpoint: &str,
        parameters: &[(&str, String)],
    ) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .get(self.endpoint_url(endpoint))
            .query(&self.auth_query())
            .query(parameters)
            .send()
            .await
            .map_err(|error| format!("Failed to call {endpoint}: {error}"))?
            .error_for_status()
            .map_err(|error| format!("{endpoint} returned an HTTP error: {error}"))?;

        let envelope: SubsonicEnvelope<T> = response
            .json()
            .await
            .map_err(|error| format!("{endpoint} returned an invalid response: {error}"))?;

        if envelope.subsonic_response.status != "ok" {
            let message = envelope
                .subsonic_response
                .error
                .map(|error| error.message)
                .unwrap_or_else(|| "Unknown server error".to_string());
            return Err(format!("{endpoint} failed: {message}"));
        }

        envelope
            .subsonic_response
            .payload
            .ok_or_else(|| format!("{endpoint} response is missing its payload"))
    }

    async fn request_binary(
        &self,
        url: Url,
        parameters: &[(&str, String)],
        source_key: String,
    ) -> Result<Option<BinaryArtwork>, String> {
        let mut query = self.auth_query();
        query.retain(|(key, _)| *key != "f");

        let response = self
            .client
            .get(url)
            .query(&query)
            .query(parameters)
            .send()
            .await
            .map_err(|error| format!("Failed to download artwork: {error}"))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let response = response
            .error_for_status()
            .map_err(|error| format!("Artwork request returned an HTTP error: {error}"))?;
        binary_artwork(response, source_key).await.map(Some)
    }

    fn same_origin(&self, url: &Url) -> bool {
        url.scheme() == self.base_url.scheme()
            && url.host_str() == self.base_url.host_str()
            && url.port_or_known_default() == self.base_url.port_or_known_default()
    }
}

#[async_trait]
impl MusicServer for NavidromeBackend {
    async fn ping(&self) -> Result<ServerInfo, String> {
        let response: PingPayload = self.request_json("ping", &[]).await?;

        Ok(ServerInfo {
            server_type: response
                .server_type
                .unwrap_or_else(|| "navidrome".to_string()),
            server_version: response.server_version,
            api_version: response.version,
            username: self.username.clone(),
        })
    }

    async fn library_revision(&self) -> Result<Option<String>, String> {
        let status: ScanStatusPayload = self.request_json("getScanStatus", &[]).await?;
        Ok(status.scan_status.last_scan.map(|revision| match revision {
            serde_json::Value::String(value) => value,
            value => value.to_string(),
        }))
    }

    async fn artists(&self) -> Result<Vec<Artist>, String> {
        let payload: ArtistsPayload = self.request_json("getArtists", &[]).await?;
        Ok(payload
            .artists
            .index
            .into_iter()
            .flat_map(|index| index.artist)
            .map(|artist| Artist {
                remote_id: artist.id,
                name: artist.name,
                album_count: artist.album_count,
            })
            .collect())
    }

    async fn albums(&self) -> Result<Vec<Album>, String> {
        let mut albums = Vec::new();
        let mut offset = 0_u32;

        loop {
            let payload: AlbumListPayload = self
                .request_json(
                    "getAlbumList2",
                    &[
                        ("type", "alphabeticalByArtist".to_string()),
                        ("offset", offset.to_string()),
                        ("size", ALBUM_PAGE_SIZE.to_string()),
                    ],
                )
                .await?;
            let page = payload.album_list2.album;
            let is_last_page = page.len() < ALBUM_PAGE_SIZE as usize;
            albums.extend(page.into_iter().map(AlbumDto::into_album));

            if is_last_page {
                return Ok(albums);
            }
            offset += ALBUM_PAGE_SIZE;
        }
    }

    async fn album(&self, id: &str) -> Result<AlbumWithSongs, String> {
        let payload: AlbumPayload = self
            .request_json("getAlbum", &[("id", id.to_string())])
            .await?;
        Ok(payload.album.into_album_with_songs())
    }

    async fn genres(&self) -> Result<Vec<Genre>, String> {
        let payload: GenresPayload = self.request_json("getGenres", &[]).await?;
        Ok(payload
            .genres
            .genre
            .into_iter()
            .map(|genre| Genre {
                name: genre.value,
                song_count: genre.song_count,
                album_count: genre.album_count,
            })
            .collect())
    }

    async fn album_artwork(&self, cover_art_id: &str) -> Result<Option<BinaryArtwork>, String> {
        self.request_binary(
            self.endpoint_url("getCoverArt"),
            &[
                ("id", cover_art_id.to_string()),
                ("size", "600".to_string()),
            ],
            cover_art_id.to_string(),
        )
        .await
    }

    async fn artist_artwork(&self, artist_id: &str) -> Result<Option<BinaryArtwork>, String> {
        let payload: ArtistInfoPayload = self
            .request_json(
                "getArtistInfo2",
                &[("id", artist_id.to_string()), ("count", "0".to_string())],
            )
            .await?;
        let Some(source) = payload
            .artist_info2
            .large_image_url
            .or(payload.artist_info2.medium_image_url)
            .or(payload.artist_info2.small_image_url)
        else {
            return Ok(None);
        };
        let url =
            Url::parse(&source).map_err(|error| format!("Artist image URL is invalid: {error}"))?;
        if !self.same_origin(&url) {
            return Ok(None);
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|error| format!("Failed to download artist image: {error}"))?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let response = response
            .error_for_status()
            .map_err(|error| format!("Artist image returned an HTTP error: {error}"))?;
        binary_artwork(response, source).await.map(Some)
    }
}

async fn binary_artwork(response: Response, source_key: String) -> Result<BinaryArtwork, String> {
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .split(';')
        .next()
        .unwrap_or("application/octet-stream")
        .to_string();
    if !content_type.starts_with("image/") {
        return Err(format!(
            "Artwork has unsupported content type: {content_type}"
        ));
    }
    let etag = header_value(&response, ETAG);
    let last_modified = header_value(&response, LAST_MODIFIED);
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Failed to read artwork response: {error}"))?
        .to_vec();

    Ok(BinaryArtwork {
        source_key,
        content_type,
        bytes,
        etag,
        last_modified,
    })
}

fn header_value(response: &Response, name: reqwest::header::HeaderName) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::{
        AlbumListPayload, ArtistsPayload, NavidromeBackend, PingPayload, SubsonicEnvelope,
    };

    #[test]
    fn builds_endpoint_for_server_root() {
        let backend = backend("https://music.example.com/");
        assert_eq!(
            backend.endpoint_url("ping").as_str(),
            "https://music.example.com/rest/ping.view"
        );
    }

    #[test]
    fn preserves_server_path_prefix() {
        let backend = backend("https://example.com/music");
        assert_eq!(
            backend.endpoint_url("ping").as_str(),
            "https://example.com/music/rest/ping.view"
        );
    }

    #[test]
    fn only_accepts_artist_images_from_server_origin() {
        let backend = backend("https://example.com/music");
        assert!(backend.same_origin(&"https://example.com/art.jpg".parse().unwrap()));
        assert!(!backend.same_origin(&"https://cdn.example.com/art.jpg".parse().unwrap()));
        assert!(!backend.same_origin(&"http://example.com/art.jpg".parse().unwrap()));
    }

    #[test]
    fn rejects_non_http_url() {
        let result = NavidromeBackend::new(
            "file:///music".to_string(),
            "user".to_string(),
            "password".to_string(),
        );
        assert_eq!(
            result.err().as_deref(),
            Some("Server URL must use HTTP or HTTPS")
        );
    }

    #[test]
    fn parses_artist_list_response() {
        let response: SubsonicEnvelope<ArtistsPayload> = serde_json::from_str(
            r#"{
              "subsonic-response": {
                "status": "ok",
                "version": "1.16.1",
                "artists": {
                  "index": [{
                    "name": "A",
                    "artist": [{"id": "artist-1", "name": "Artist", "albumCount": 2}]
                  }]
                }
              }
            }"#,
        )
        .unwrap();

        let artists = response.subsonic_response.payload.unwrap().artists;
        assert_eq!(artists.index[0].artist[0].id, "artist-1");
        assert_eq!(artists.index[0].artist[0].album_count, 2);
    }

    #[test]
    fn parses_ping_through_generic_envelope() {
        let response: SubsonicEnvelope<PingPayload> = serde_json::from_str(
            r#"{
              "subsonic-response": {
                "status": "ok",
                "version": "1.16.1",
                "type": "navidrome",
                "serverVersion": "0.58.0"
              }
            }"#,
        )
        .unwrap();

        let ping = response.subsonic_response.payload.unwrap();
        assert_eq!(ping.version, "1.16.1");
        assert_eq!(ping.server_type.as_deref(), Some("navidrome"));
        assert_eq!(ping.server_version.as_deref(), Some("0.58.0"));
    }

    #[test]
    fn parses_album_page_response() {
        let response: SubsonicEnvelope<AlbumListPayload> = serde_json::from_str(
            r#"{
              "subsonic-response": {
                "status": "ok",
                "version": "1.16.1",
                "albumList2": {
                  "album": [{
                    "id": "album-1",
                    "name": "Album",
                    "artistId": "artist-1",
                    "artist": "Artist",
                    "songCount": 4,
                    "duration": 900,
                    "coverArt": "cover-1"
                  }]
                }
              }
            }"#,
        )
        .unwrap();

        let albums = response
            .subsonic_response
            .payload
            .unwrap()
            .album_list2
            .album;
        assert_eq!(albums[0].id, "album-1");
        assert_eq!(albums[0].song_count, 4);
    }

    fn backend(url: &str) -> NavidromeBackend {
        NavidromeBackend::new(url.to_string(), "user".to_string(), "password".to_string()).unwrap()
    }
}
