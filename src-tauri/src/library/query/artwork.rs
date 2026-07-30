use std::path::Path;

use crate::database::SqliteRepository;

use super::super::models::{ArtworkCacheRecord, ArtworkCandidate};

#[derive(sqlx::FromRow)]
struct ArtworkCacheState {
    local_path: Option<String>,
    source_key: Option<String>,
    downloaded_at: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct AlbumArtworkCandidate {
    remote_id: String,
    cover_art_id: String,
    name: String,
}

#[derive(sqlx::FromRow)]
struct ArtistArtworkCandidate {
    remote_id: String,
    name: String,
}

pub(crate) async fn artwork_is_fresh(
    repo: &SqliteRepository,
    profile_id: &str,
    kind: &str,
    remote_id: &str,
    source_key: Option<&str>,
    fresh_after: i64,
) -> Result<bool, String> {
    let row = sqlx::query_as::<_, ArtworkCacheState>(
        "SELECT local_path, source_key, downloaded_at
         FROM artwork_cache WHERE profile_id = ? AND kind = ? AND remote_id = ?",
    )
    .bind(profile_id)
    .bind(kind)
    .bind(remote_id)
    .fetch_optional(&repo.pool)
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

pub(crate) async fn artwork_candidates(
    repo: &SqliteRepository,
    profile_id: &str,
) -> Result<Vec<ArtworkCandidate>, String> {
    let album_rows = sqlx::query_as::<_, AlbumArtworkCandidate>(
        "SELECT a.remote_id, a.cover_art_id, a.name
         FROM albums a
         JOIN library_sync_state s
           ON s.profile_id = a.profile_id
          AND s.active_generation = a.generation
         WHERE a.profile_id = ? AND a.cover_art_id IS NOT NULL",
    )
    .bind(profile_id)
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read album artwork candidates: {error}"))?;

    let artist_rows = sqlx::query_as::<_, ArtistArtworkCandidate>(
        "SELECT a.remote_id, a.name
         FROM artists a
         JOIN library_sync_state s
           ON s.profile_id = a.profile_id
          AND s.active_generation = a.generation
         WHERE a.profile_id = ?",
    )
    .bind(profile_id)
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read artist artwork candidates: {error}"))?;

    let mut candidates = Vec::with_capacity(album_rows.len() + artist_rows.len());
    candidates.extend(album_rows.into_iter().map(|row| ArtworkCandidate {
        kind: "album",
        remote_id: row.remote_id,
        source_id: row.cover_art_id,
        name: row.name,
    }));
    candidates.extend(artist_rows.into_iter().map(|row| {
        let remote_id = row.remote_id;
        ArtworkCandidate {
            kind: "artist",
            source_id: remote_id.clone(),
            remote_id,
            name: row.name,
        }
    }));
    Ok(candidates)
}

pub(crate) async fn save_artwork(
    repo: &SqliteRepository,
    profile_id: &str,
    artwork: ArtworkCacheRecord,
) -> Result<(), String> {
    sqlx::query(
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
    )
    .bind(profile_id)
    .bind(artwork.kind)
    .bind(artwork.remote_id)
    .bind(artwork.source_key)
    .bind(artwork.local_path)
    .bind(artwork.content_type)
    .bind(artwork.etag)
    .bind(artwork.last_modified)
    .bind(artwork.downloaded_at)
    .bind(artwork.downloaded_at)
    .execute(&repo.pool)
    .await
    .map_err(|error| format!("Failed to save artwork cache record: {error}"))?;
    Ok(())
}
