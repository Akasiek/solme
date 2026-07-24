use std::collections::HashMap;

use sqlx::{QueryBuilder, Sqlite, Transaction};

use super::super::{
    fuzzy_search,
    models::{AlbumSort, AlbumWithSongs, CachedAlbum},
};
use crate::database::SqliteRepository;

const SQLITE_BIND_LIMIT: usize = 999;
const ALBUM_SELECT_FROM_ACTIVE_GENERATION: &str = "
    SELECT a.remote_id, a.name, a.album_type, a.artist_name, a.artist_id, a.year,
           a.release_date, a.original_release_date, a.server_added_at, a.song_count,
           a.duration_seconds, art.local_path AS artwork_path
    FROM albums a
    JOIN library_sync_state s
      ON s.profile_id = a.profile_id
     AND s.active_generation = a.generation
    LEFT JOIN artwork_cache art
      ON art.profile_id = a.profile_id
     AND art.kind = 'album'
     AND art.remote_id = a.remote_id
    WHERE a.profile_id = ";

pub(crate) async fn insert_albums(
    transaction: &mut Transaction<'_, Sqlite>,
    profile_id: &str,
    generation: &str,
    albums: &[AlbumWithSongs],
) -> Result<(), String> {
    for albums in albums.chunks(SQLITE_BIND_LIMIT / 14) {
        let mut query = QueryBuilder::new(
            "INSERT INTO albums
             (profile_id, generation, remote_id, name, album_type, artist_id, artist_name,
              year, release_date, original_release_date, server_added_at, song_count, duration_seconds, cover_art_id) ",
        );
        query.push_values(albums, |mut row, details| {
            let album = &details.album;
            row.push_bind(profile_id)
                .push_bind(generation)
                .push_bind(&album.remote_id)
                .push_bind(&album.name)
                .push_bind(&album.album_type)
                .push_bind(&album.artist_id)
                .push_bind(&album.artist_name)
                .push_bind(album.year)
                .push_bind(&album.release_date)
                .push_bind(&album.original_release_date)
                .push_bind(&album.server_added_at)
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

pub(crate) async fn albums(
    repo: &SqliteRepository,
    profile_id: &str,
    offset: i64,
    limit: i64,
    sort: AlbumSort,
) -> Result<Vec<CachedAlbum>, String> {
    let limit = limit.clamp(-1, 500);
    let offset = offset.max(0);
    let (filter, order) = album_list_filter_and_order(sort);
    let mut query = QueryBuilder::new(ALBUM_SELECT_FROM_ACTIVE_GENERATION);
    query
        .push_bind(profile_id)
        .push(filter)
        .push(" ")
        .push(order)
        .push(" LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    query
        .build_query_as::<CachedAlbum>()
        .fetch_all(&repo.pool)
        .await
        .map_err(|error| format!("Failed to read cached albums: {error}"))
}

pub(crate) async fn albums_by_ids(
    repo: &SqliteRepository,
    profile_id: &str,
    album_ids: &[String],
) -> Result<Vec<CachedAlbum>, String> {
    if album_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut query = QueryBuilder::new(ALBUM_SELECT_FROM_ACTIVE_GENERATION);
    query.push_bind(profile_id).push(" AND a.remote_id IN (");
    let mut ids = query.separated(", ");
    for album_id in album_ids {
        ids.push_bind(album_id);
    }
    ids.push_unseparated(")");

    let albums = query
        .build_query_as::<CachedAlbum>()
        .fetch_all(&repo.pool)
        .await
        .map_err(|error| format!("Failed to read cached albums by ID: {error}"))?;
    let mut albums_by_id = albums
        .into_iter()
        .map(|album| (album.remote_id.clone(), album))
        .collect::<HashMap<_, _>>();

    Ok(album_ids
        .iter()
        .filter_map(|album_id| albums_by_id.remove(album_id))
        .collect())
}

fn album_list_filter_and_order(sort: AlbumSort) -> (&'static str, &'static str) {
    match sort {
        AlbumSort::Artist => (
            "",
            "ORDER BY a.artist_name COLLATE NOCASE, a.year, a.name COLLATE NOCASE",
        ),
        AlbumSort::Random => ("", "ORDER BY RANDOM()"),
        AlbumSort::RecentlyAdded => (
            "",
            "ORDER BY a.server_added_at IS NULL,
                      a.server_added_at DESC,
                      a.artist_name COLLATE NOCASE,
                      a.name COLLATE NOCASE",
        ),
        AlbumSort::RecentlyReleased => (
            " AND COALESCE(a.original_release_date, a.release_date) IS NOT NULL",
            "ORDER BY COALESCE(a.original_release_date, a.release_date, CASE WHEN a.year IS NOT NULL THEN printf('%04d-12-31', a.year) END) IS NULL,
                      COALESCE(a.original_release_date, a.release_date, CASE WHEN a.year IS NOT NULL THEN printf('%04d-12-31', a.year) END) DESC,
                      a.artist_name COLLATE NOCASE,
                      a.name COLLATE NOCASE",
        ),
    }
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
        "SELECT a.remote_id, a.name, a.album_type, a.artist_name, a.artist_id, a.year,
                a.release_date, a.original_release_date, a.server_added_at, a.song_count,
                a.duration_seconds, artwork.local_path AS artwork_path
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
    let results = sqlx::query_as::<_, CachedAlbum>(
        "SELECT a.remote_id, a.name, a.album_type, a.artist_name, a.artist_id, a.year,
                a.release_date, a.original_release_date, a.server_added_at, a.song_count,
                a.duration_seconds, artwork.local_path AS artwork_path
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
    )
    .bind(profile_id)
    .bind(fts_query)
    .bind(limit)
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
