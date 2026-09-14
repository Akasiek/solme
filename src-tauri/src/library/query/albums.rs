use std::collections::{BTreeSet, HashMap};

use sqlx::{QueryBuilder, Sqlite, Transaction};

use super::super::{
    fuzzy_search,
    models::{
        AlbumPage, AlbumPageSort, AlbumSort, AlbumWithSongs, CachedAlbum, CatalogFilter, Paginated,
        Pagination,
    },
};
use crate::database::SqliteRepository;

const SQLITE_BIND_LIMIT: usize = 999;
pub(super) const ALBUM_SELECT_FROM_ACTIVE_GENERATION: &str = "
    SELECT a.remote_id, a.name, a.album_type, a.artist_name, a.artist_id, a.year,
           a.release_date, a.original_release_date, a.server_added_at, a.play_count,
           a.last_played_at, a.song_count,
           a.duration_seconds, art.local_path AS artwork_path, a.favorite, a.rating
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
    for albums in albums.chunks(SQLITE_BIND_LIMIT / 18) {
        let mut query = QueryBuilder::new(
            "INSERT INTO albums
             (profile_id, generation, remote_id, name, album_type, artist_id, artist_name,
              year, release_date, original_release_date, server_added_at, play_count,
              last_played_at, song_count, duration_seconds, cover_art_id, favorite, rating) ",
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
                .push_bind(album.play_count)
                .push_bind(&album.last_played_at)
                .push_bind(album.song_count)
                .push_bind(album.duration_seconds)
                .push_bind(&album.cover_art_id)
                .push_bind(album.favorite)
                .push_bind(album.rating);
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

pub(crate) async fn album_page(
    repo: &SqliteRepository,
    profile_id: &str,
    search: &str,
    album_types: &[String],
    filters: CatalogFilter,
    sort: AlbumPageSort,
    pagination: Pagination,
) -> Result<AlbumPage, String> {
    let mut items_query = album_page_query(
        profile_id,
        search,
        album_types,
        filters.clone(),
        sort,
        pagination,
    );
    let items = items_query
        .build_query_as::<CachedAlbum>()
        .fetch_all(&repo.pool)
        .await
        .map_err(|error| format!("Failed to read album page: {error}"))?;

    let total = count_albums(repo, profile_id, search, album_types, filters).await?;
    let album_types = available_album_types(repo, profile_id).await?;
    let genres = available_genres(repo, profile_id).await?;

    Ok(AlbumPage {
        page: Paginated { items, total },
        album_types,
        genres,
    })
}

fn album_page_query(
    profile_id: &str,
    search: &str,
    album_types: &[String],
    filters: CatalogFilter,
    sort: AlbumPageSort,
    pagination: Pagination,
) -> QueryBuilder<Sqlite> {
    let mut query = QueryBuilder::new(ALBUM_SELECT_FROM_ACTIVE_GENERATION);
    query.push_bind(profile_id.to_owned());

    push_album_page_filters(&mut query, search.trim(), album_types, filters);
    push_album_page_sort(&mut query, sort);
    push_album_page_pagination(&mut query, pagination);

    query
}

fn push_album_page_filters(
    query: &mut QueryBuilder<Sqlite>,
    search: &str,
    album_types: &[String],
    filters: CatalogFilter,
) {
    let filters = filters.normalized();
    push_album_search_filter(query, search);
    push_album_types_filter(query, album_types);
    push_annotation_filters(query, filters.clone());
    push_album_from_year_filter(query, filters.from_year);
    push_album_to_year_filter(query, filters.to_year);
    push_album_genres_filter(query, filters.genres);
    push_album_never_played_filter(query, filters.never_played);
    push_album_minimum_play_count_filter(query, filters.minimum_play_count);
}

fn push_album_search_filter(query: &mut QueryBuilder<Sqlite>, search: &str) {
    if !search.is_empty() {
        query
            .push(" AND (INSTR(LOWER(a.name), LOWER(")
            .push_bind(search.to_owned())
            .push(")) > 0 OR INSTR(LOWER(a.artist_name), LOWER(")
            .push_bind(search.to_owned())
            .push(")) > 0)");
    }
}

fn push_album_types_filter(query: &mut QueryBuilder<Sqlite>, album_types: &[String]) {
    if !album_types.is_empty() {
        query.push(" AND a.album_type IN (");
        let mut types = query.separated(", ");
        for album_type in album_types {
            types.push_bind(album_type.clone());
        }
        types.push_unseparated(")");
    }
}

fn push_album_from_year_filter(query: &mut QueryBuilder<Sqlite>, from_year: Option<i64>) {
    if let Some(from_year) = from_year {
        query.push(" AND a.year >= ").push_bind(from_year);
    }
}

fn push_album_to_year_filter(query: &mut QueryBuilder<Sqlite>, to_year: Option<i64>) {
    if let Some(to_year) = to_year {
        query.push(" AND a.year <= ").push_bind(to_year);
    }
}

fn push_album_genres_filter(query: &mut QueryBuilder<Sqlite>, selected_genres: Vec<String>) {
    if !selected_genres.is_empty() {
        query.push(
            " AND EXISTS (SELECT 1 FROM album_genres ag
             WHERE ag.profile_id = a.profile_id AND ag.generation = a.generation
               AND ag.album_id = a.remote_id AND ag.genre IN (",
        );
        let mut genres = query.separated(", ");
        for genre in selected_genres {
            genres.push_bind(genre);
        }
        genres.push_unseparated("))");
    }
}

fn push_album_never_played_filter(query: &mut QueryBuilder<Sqlite>, never_played: bool) {
    if never_played {
        query.push(" AND a.play_count = 0");
    }
}

fn push_album_minimum_play_count_filter(
    query: &mut QueryBuilder<Sqlite>,
    minimum_play_count: Option<i64>,
) {
    if let Some(minimum_play_count) = minimum_play_count {
        query
            .push(" AND a.play_count >= ")
            .push_bind(minimum_play_count);
    }
}

pub(super) fn push_annotation_filters(query: &mut QueryBuilder<Sqlite>, filters: CatalogFilter) {
    let filters = filters.normalized();
    push_favorite_filter(query, filters.favorite_only);
    push_minimum_rating_filter(query, filters.minimum_rating);
    push_unrated_filter(query, filters.unrated_only);
}

fn push_favorite_filter(query: &mut QueryBuilder<Sqlite>, favorite_only: bool) {
    if favorite_only {
        query.push(" AND a.favorite = 1");
    }
}

fn push_minimum_rating_filter(query: &mut QueryBuilder<Sqlite>, minimum_rating: Option<i64>) {
    if let Some(minimum_rating) = minimum_rating {
        query.push(" AND a.rating >= ").push_bind(minimum_rating);
    }
}

fn push_unrated_filter(query: &mut QueryBuilder<Sqlite>, unrated_only: bool) {
    if unrated_only {
        query.push(" AND a.rating IS NULL");
    }
}

fn push_album_page_sort(query: &mut QueryBuilder<Sqlite>, sort: AlbumPageSort) {
    query.push(" ").push(match sort {
        AlbumPageSort::Artist => {
            "ORDER BY a.artist_name COLLATE NOCASE, a.year, a.name COLLATE NOCASE"
        }
        AlbumPageSort::Title => {
            "ORDER BY a.name COLLATE NOCASE, a.artist_name COLLATE NOCASE, a.year"
        }
        AlbumPageSort::Newest => {
            "ORDER BY COALESCE(a.original_release_date, a.release_date, printf('%04d', a.year)) IS NULL,
                      COALESCE(a.original_release_date, a.release_date, printf('%04d', a.year)) DESC,
                      a.name COLLATE NOCASE"
        }
        AlbumPageSort::Oldest => {
            "ORDER BY COALESCE(a.original_release_date, a.release_date, printf('%04d', a.year)) IS NULL,
                      COALESCE(a.original_release_date, a.release_date, printf('%04d', a.year)),
                      a.name COLLATE NOCASE"
        }
        AlbumPageSort::RecentlyAdded => {
            "ORDER BY a.server_added_at IS NULL, a.server_added_at DESC, a.name COLLATE NOCASE"
        }
        AlbumPageSort::RecentlyPlayed => {
            "ORDER BY a.last_played_at IS NULL, a.last_played_at DESC, a.name COLLATE NOCASE"
        }
        AlbumPageSort::MostPlayed => {
            "ORDER BY a.play_count DESC, a.name COLLATE NOCASE"
        }
    });
}

fn push_album_page_pagination(query: &mut QueryBuilder<Sqlite>, pagination: Pagination) {
    let pagination = pagination.normalized();
    query.push(" LIMIT ").push_bind(pagination.limit);
    query.push(" OFFSET ").push_bind(pagination.offset);
}

async fn count_albums(
    repo: &SqliteRepository,
    profile_id: &str,
    search: &str,
    album_types: &[String],
    filters: CatalogFilter,
) -> Result<i64, String> {
    let mut query = QueryBuilder::new(
        "SELECT COUNT(*) FROM albums a
         JOIN library_sync_state s
           ON s.profile_id = a.profile_id
          AND s.active_generation = a.generation
         WHERE a.profile_id = ",
    );
    query.push_bind(profile_id);
    push_album_page_filters(&mut query, search.trim(), album_types, filters);

    query
        .build_query_scalar::<i64>()
        .fetch_one(&repo.pool)
        .await
        .map_err(|error| format!("Failed to count filtered albums: {error}"))
}

async fn available_album_types(
    repo: &SqliteRepository,
    profile_id: &str,
) -> Result<Vec<String>, String> {
    sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT a.album_type
         FROM albums a
         JOIN library_sync_state s
           ON s.profile_id = a.profile_id
          AND s.active_generation = a.generation
         WHERE a.profile_id = ? AND a.album_type IS NOT NULL AND TRIM(a.album_type) != ''
         ORDER BY a.album_type COLLATE NOCASE",
    )
    .bind(profile_id)
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read album types: {error}"))
}

async fn available_genres(
    repo: &SqliteRepository,
    profile_id: &str,
) -> Result<Vec<String>, String> {
    sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT ag.genre
         FROM album_genres ag
         JOIN library_sync_state s
           ON s.profile_id = ag.profile_id
          AND s.active_generation = ag.generation
         WHERE ag.profile_id = ? AND TRIM(ag.genre) != ''
         ORDER BY ag.genre COLLATE NOCASE",
    )
    .bind(profile_id)
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read album genres: {error}"))
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

pub(crate) async fn album(
    repo: &SqliteRepository,
    profile_id: &str,
    album_id: &str,
) -> Result<Option<CachedAlbum>, String> {
    let mut query = QueryBuilder::new(ALBUM_SELECT_FROM_ACTIVE_GENERATION);
    query
        .push_bind(profile_id)
        .push(" AND a.remote_id = ")
        .push_bind(album_id);

    query
        .build_query_as::<CachedAlbum>()
        .fetch_optional(&repo.pool)
        .await
        .map_err(|error| format!("Failed to read cached album: {error}"))
}

pub(crate) async fn album_genres(
    repo: &SqliteRepository,
    profile_id: &str,
    album_id: &str,
) -> Result<Vec<String>, String> {
    sqlx::query_scalar::<_, String>(
        "SELECT ag.genre
         FROM album_genres ag
         JOIN library_sync_state state
           ON state.profile_id = ag.profile_id
          AND state.active_generation = ag.generation
         WHERE ag.profile_id = ? AND ag.album_id = ?
         ORDER BY ag.genre COLLATE NOCASE",
    )
    .bind(profile_id)
    .bind(album_id)
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read cached album genres: {error}"))
}

pub(crate) async fn album_disc_count(
    repo: &SqliteRepository,
    profile_id: &str,
    album_id: &str,
) -> Result<i64, String> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT COALESCE(NULLIF(song.disc_number, 0), 1))
         FROM songs song
         JOIN library_sync_state state
           ON state.profile_id = song.profile_id
          AND state.active_generation = song.generation
         WHERE song.profile_id = ? AND song.album_id = ?",
    )
    .bind(profile_id)
    .bind(album_id)
    .fetch_one(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read cached album disc count: {error}"))
}

pub(crate) async fn album_audio_formats(
    repo: &SqliteRepository,
    profile_id: &str,
    album_id: &str,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query_as::<_, (Option<String>, Option<String>)>(
        "SELECT DISTINCT song.suffix, song.content_type
         FROM songs song
         JOIN library_sync_state state
           ON state.profile_id = song.profile_id
          AND state.active_generation = song.generation
         WHERE song.profile_id = ?
           AND song.album_id = ?
           AND COALESCE(NULLIF(song.suffix, ''), song.content_type) IS NOT NULL
         ORDER BY 1",
    )
    .bind(profile_id)
    .bind(album_id)
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read cached album audio formats: {error}"))?;

    Ok(rows
        .into_iter()
        .filter_map(|(suffix, content_type)| {
            audio_format_label(suffix.as_deref(), content_type.as_deref())
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect())
}

pub(crate) fn audio_format_label(
    suffix: Option<&str>,
    content_type: Option<&str>,
) -> Option<String> {
    suffix
        .filter(|value| !value.trim().is_empty())
        .or_else(|| content_type.and_then(|value| value.rsplit('/').next()))
        .map(|value| value.trim().to_uppercase())
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
    let mut query = QueryBuilder::new(ALBUM_SELECT_FROM_ACTIVE_GENERATION);
    query.push_bind(profile_id);

    query
        .build_query_as::<CachedAlbum>()
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
                a.release_date, a.original_release_date, a.server_added_at, a.play_count,
                a.last_played_at, a.song_count,
                a.duration_seconds, artwork.local_path AS artwork_path, a.favorite, a.rating
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

#[cfg(test)]
mod tests {
    use super::audio_format_label;

    #[test]
    fn formats_audio_format_label() {
        assert_eq!(
            audio_format_label(Some("flac"), Some("audio/flac")),
            Some("FLAC".to_string())
        );
    }
}
