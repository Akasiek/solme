use serde::Deserialize;

use crate::library::models::{Album, AlbumWithSongs, Song};

#[derive(Deserialize)]
pub(super) struct SubsonicEnvelope<T> {
    #[serde(rename = "subsonic-response")]
    pub subsonic_response: SubsonicResponse<T>,
}

#[derive(Deserialize)]
pub(super) struct SubsonicResponse<T> {
    pub status: String,
    pub error: Option<SubsonicError>,
    #[serde(flatten)]
    pub payload: Option<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PingPayload {
    pub version: String,
    #[serde(rename = "type")]
    pub server_type: Option<String>,
    pub server_version: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct SubsonicError {
    pub message: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ScanStatusPayload {
    pub scan_status: ScanStatus,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ScanStatus {
    pub last_scan: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub(super) struct ArtistsPayload {
    pub artists: ArtistsDto,
}

#[derive(Deserialize)]
pub(super) struct ArtistsDto {
    #[serde(default)]
    pub index: Vec<ArtistIndexDto>,
}

#[derive(Deserialize)]
pub(super) struct ArtistIndexDto {
    #[serde(default)]
    pub artist: Vec<ArtistDto>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ArtistDto {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub album_count: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AlbumListPayload {
    pub album_list2: AlbumListDto,
}

#[derive(Deserialize)]
pub(super) struct AlbumListDto {
    #[serde(default)]
    pub album: Vec<AlbumDto>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AlbumDto {
    pub id: String,
    pub name: String,
    pub artist_id: Option<String>,
    #[serde(default)]
    pub artist: String,
    pub year: Option<i64>,
    #[serde(default)]
    pub song_count: i64,
    #[serde(default)]
    pub duration: i64,
    pub cover_art: Option<String>,
    pub genre: Option<String>,
    #[serde(default)]
    genres: Vec<NamedGenreDto>,
    #[serde(default)]
    song: Vec<SongDto>,
}

impl AlbumDto {
    pub fn into_album(self) -> Album {
        let AlbumDto {
            id,
            name,
            artist_id,
            artist,
            year,
            song_count,
            duration,
            cover_art,
            genre,
            genres,
            ..
        } = self;

        Album {
            remote_id: id,
            name,
            artist_id,
            artist_name: artist,
            year,
            song_count,
            duration_seconds: duration,
            cover_art_id: cover_art,
            genres: collect_genres(genre, genres),
        }
    }

    pub fn into_album_with_songs(self) -> AlbumWithSongs {
        let AlbumDto {
            id,
            name,
            artist_id,
            artist,
            year,
            song_count,
            duration,
            cover_art,
            genre,
            genres,
            song,
        } = self;
        let songs = song
            .into_iter()
            .map(|song| song.into_song(&id, &name, &artist))
            .collect();

        AlbumWithSongs {
            album: Album {
                remote_id: id,
                name,
                artist_id,
                artist_name: artist,
                year,
                song_count,
                duration_seconds: duration,
                cover_art_id: cover_art,
                genres: collect_genres(genre, genres),
            },
            songs,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct AlbumPayload {
    pub album: AlbumDto,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SongDto {
    id: String,
    album_id: Option<String>,
    artist_id: Option<String>,
    title: String,
    artist: Option<String>,
    album: Option<String>,
    track: Option<i64>,
    disc_number: Option<i64>,
    year: Option<i64>,
    #[serde(default)]
    duration: i64,
    suffix: Option<String>,
    content_type: Option<String>,
    cover_art: Option<String>,
    genre: Option<String>,
    #[serde(default)]
    genres: Vec<NamedGenreDto>,
}

impl SongDto {
    fn into_song(
        self,
        fallback_album_id: &str,
        fallback_album: &str,
        fallback_artist: &str,
    ) -> Song {
        Song {
            remote_id: self.id,
            album_id: self
                .album_id
                .unwrap_or_else(|| fallback_album_id.to_string()),
            artist_id: self.artist_id,
            title: self.title,
            artist_name: self.artist.unwrap_or_else(|| fallback_artist.to_string()),
            album_name: self.album.unwrap_or_else(|| fallback_album.to_string()),
            track_number: self.track,
            disc_number: self.disc_number,
            year: self.year,
            duration_seconds: self.duration,
            suffix: self.suffix,
            content_type: self.content_type,
            cover_art_id: self.cover_art,
            genres: collect_genres(self.genre, self.genres),
        }
    }
}

#[derive(Deserialize)]
struct NamedGenreDto {
    name: String,
}

fn collect_genres(primary: Option<String>, genres: Vec<NamedGenreDto>) -> Vec<String> {
    let mut values = genres
        .into_iter()
        .map(|genre| genre.name)
        .collect::<Vec<_>>();
    if let Some(primary) = primary {
        if !values.iter().any(|genre| genre == &primary) {
            values.push(primary);
        }
    }
    values
}

#[derive(Deserialize)]
pub(super) struct GenresPayload {
    pub genres: GenresDto,
}

#[derive(Deserialize)]
pub(super) struct GenresDto {
    #[serde(default)]
    pub genre: Vec<GenreDto>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GenreDto {
    pub value: String,
    #[serde(default)]
    pub song_count: i64,
    #[serde(default)]
    pub album_count: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ArtistInfoPayload {
    pub artist_info2: ArtistInfoDto,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ArtistInfoDto {
    pub small_image_url: Option<String>,
    pub medium_image_url: Option<String>,
    pub large_image_url: Option<String>,
}
