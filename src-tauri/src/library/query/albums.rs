use sqlx::{QueryBuilder, Sqlite, Transaction};

use super::super::{
    fuzzy_search,
    models::{AlbumWithSongs, CachedAlbum},
};
use crate::database::SqliteRepository;

const SQLITE_BIND_LIMIT: usize = 999;

pub(crate) async fn insert_albums(
    transaction: &mut Transaction<'_, Sqlite>,
    profile_id: &str,
    generation: &str,
    albums: &[AlbumWithSongs],
) -> Result<(), String> {
    for albums in albums.chunks(SQLITE_BIND_LIMIT / 10) {
        let mut query = QueryBuilder::new(
            "INSERT INTO albums
             (profile_id, generation, remote_id, name, artist_id, artist_name,
              year, song_count, duration_seconds, cover_art_id) ",
        );
        query.push_values(albums, |mut row, details| {
            let album = &details.album;
            row.push_bind(profile_id)
                .push_bind(generation)
                .push_bind(&album.remote_id)
                .push_bind(&album.name)
                .push_bind(&album.artist_id)
                .push_bind(&album.artist_name)
                .push_bind(album.year)
                .push_bind(album.song_count)
                .push_bind(album.duration_seconds)
                .push_bind(&album.cover_art_id);
        });
        query
            .build()
            .execute(&mut **transaction)
            .await
            .map_err(|error| format!("Failed to cache albums: {error}"))?;
    }
    Ok(())
}

pub(crate) async fn insert_album_genres(
    transaction: &mut Transaction<'_, Sqlite>,
    profile_id: &str,
    generation: &str,
    albums: &[AlbumWithSongs],
) -> Result<(), String> {
    let genres = albums
        .iter()
        .flat_map(|details| {
            details
                .album
                .genres
                .iter()
                .map(|genre| (&details.album.remote_id, genre))
        })
        .collect::<Vec<_>>();

    for genres in genres.chunks(SQLITE_BIND_LIMIT / 4) {
        let mut query = QueryBuilder::new(
            "INSERT OR IGNORE INTO album_genres
             (profile_id, generation, album_id, genre) ",
        );
        query.push_values(genres, |mut row, (album_id, genre)| {
            row.push_bind(profile_id)
                .push_bind(generation)
                .push_bind(album_id)
                .push_bind(genre);
        });
        query
            .build()
            .execute(&mut **transaction)
            .await
            .map_err(|error| format!("Failed to cache album genres: {error}"))?;
    }
    Ok(())
}

pub(crate) async fn insert_album_search(
    transaction: &mut Transaction<'_, Sqlite>,
    profile_id: &str,
    generation: &str,
    albums: &[AlbumWithSongs],
) -> Result<(), String> {
    for albums in albums.chunks(SQLITE_BIND_LIMIT / 6) {
        let mut query = QueryBuilder::new(
            "INSERT INTO album_search
             (profile_id, generation, remote_id, name, artist_name, genres) ",
        );
        query.push_values(albums, |mut row, details| {
            let album = &details.album;
            row.push_bind(profile_id)
                .push_bind(generation)
                .push_bind(&album.remote_id)
                .push_bind(&album.name)
                .push_bind(&album.artist_name)
                .push_bind(album.genres.join(" "));
        });
        query
            .build()
            .execute(&mut **transaction)
            .await
            .map_err(|error| format!("Failed to index albums for search: {error}"))?;
    }
    Ok(())
}

pub(crate) async fn fuzzy_album_candidates(
    repo: &SqliteRepository,
    profile_id: &str,
) -> Result<Vec<CachedAlbum>, String> {
    sqlx::query_as::<_, CachedAlbum>(
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
         WHERE a.profile_id = ?",
    )
    .bind(profile_id)
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read fuzzy album candidates: {error}"))
}

pub(crate) async fn search_albums(
    repo: &SqliteRepository,
    profile_id: &str,
    query: &str,
    limit: i64,
) -> Result<Vec<CachedAlbum>, String> {
    let Some(fts_query) = super::search_query(query) else {
        return Ok(Vec::new());
    };
    let limit = limit.clamp(1, 500);
    let results = sqlx::query_as!(
        CachedAlbum,
        "SELECT a.remote_id, a.name, a.artist_name, a.artist_id, a.year,
                a.song_count, artwork.local_path AS artwork_path
         FROM album_search
         JOIN library_sync_state state
           ON state.profile_id = album_search.profile_id
          AND state.active_generation = album_search.generation
         JOIN albums a
           ON a.profile_id = album_search.profile_id
          AND a.generation = album_search.generation
          AND a.remote_id = album_search.remote_id
         LEFT JOIN artwork_cache artwork
           ON artwork.profile_id = a.profile_id
          AND artwork.kind = 'album'
          AND artwork.remote_id = a.remote_id
         WHERE album_search.profile_id = ?
           AND album_search MATCH ?
         ORDER BY bm25(album_search, 0.0, 0.0, 0.0, 6.0, 4.0, 2.0),
                  a.artist_name COLLATE NOCASE,
                  a.year,
                  a.name COLLATE NOCASE
         LIMIT ?",
        profile_id,
        fts_query,
        limit,
    )
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to search cached albums: {error}"))?;

    let Some(query) = fuzzy_search::should_use_fuzzy(query, results.len(), limit) else {
        return Ok(results);
    };

    let candidates = fuzzy_album_candidates(repo, profile_id).await?;
    let fuzzy_results = fuzzy_search::rank_albums(&query, candidates, limit);
    Ok(fuzzy_search::merge_albums(results, fuzzy_results, limit))
}
