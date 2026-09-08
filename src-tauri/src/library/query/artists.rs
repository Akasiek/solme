use super::super::{
    fuzzy_search,
    models::{Artist, Genre},
};
use crate::database::SqliteRepository;
use crate::library::models::{ArtistPageSort, CachedAlbum, CachedArtist, Paginated, Pagination};
use sqlx::{QueryBuilder, Sqlite, Transaction};

const SQLITE_BIND_LIMIT: usize = 999;

const ARTIST_SELECT_FROM_ACTIVE_GENERATION: &str = "
    SELECT a.remote_id, a.name, a.album_count, artwork.local_path AS artwork_path, a.favorite,
           a.rating
    FROM artists a
    JOIN library_sync_state s ON s.profile_id = a.profile_id AND s.active_generation = a.generation
    LEFT JOIN artwork_cache artwork ON artwork.profile_id = a.profile_id
      AND artwork.kind = 'artist' AND artwork.remote_id = a.remote_id
    WHERE a.profile_id = ";

pub(crate) async fn artist(
    repo: &SqliteRepository,
    profile_id: &str,
    artist_id: &str,
) -> Result<Option<CachedArtist>, String> {
    let mut query = QueryBuilder::new(ARTIST_SELECT_FROM_ACTIVE_GENERATION);
    query
        .push_bind(profile_id)
        .push(" AND a.remote_id = ")
        .push_bind(artist_id);

    query
        .build_query_as::<CachedArtist>()
        .fetch_optional(&repo.pool)
        .await
        .map_err(|error| format!("Failed to read cached artist: {error}"))
}

pub(crate) async fn artist_albums(
    repo: &SqliteRepository,
    profile_id: &str,
    artist_id: &str,
) -> Result<Vec<CachedAlbum>, String> {
    sqlx::query_as::<_, CachedAlbum>(
        "SELECT a.remote_id, a.name, a.album_type, a.artist_name, a.artist_id, a.year,
                a.release_date, a.original_release_date, a.server_added_at, a.song_count,
                a.duration_seconds, artwork.local_path AS artwork_path, a.favorite, a.rating
         FROM albums a
         JOIN library_sync_state state
           ON state.profile_id = a.profile_id
          AND state.active_generation = a.generation
         LEFT JOIN artwork_cache artwork
           ON artwork.profile_id = a.profile_id
          AND artwork.kind = 'album'
          AND artwork.remote_id = a.remote_id
         WHERE a.profile_id = ? AND a.artist_id = ?
         ORDER BY COALESCE(a.original_release_date, a.release_date, CASE WHEN a.year IS NOT NULL THEN printf('%04d-12-31', a.year) END) IS NULL,
                  COALESCE(a.original_release_date, a.release_date, CASE WHEN a.year IS NOT NULL THEN printf('%04d-12-31', a.year) END) DESC,
                  a.name COLLATE NOCASE",
    )
        .bind(profile_id)
        .bind(artist_id)
        .fetch_all(&repo.pool)
        .await
        .map_err(|error| format!("Failed to read cached artist albums: {error}"))
}

pub(crate) async fn artist_page(
    repo: &SqliteRepository,
    profile_id: &str,
    search: &str,
    sort: ArtistPageSort,
    pagination: Pagination,
) -> Result<Paginated<CachedArtist>, String> {
    let mut items_query = artist_page_query(profile_id, search, sort, pagination);
    let items = items_query
        .build_query_as::<CachedArtist>()
        .fetch_all(&repo.pool)
        .await
        .map_err(|error| format!("Failed to read artist page: {error}"))?;

    let total = count_artists(repo, profile_id, search).await?;

    Ok(Paginated { items, total })
}

fn artist_page_query(
    profile_id: &str,
    search: &str,
    sort: ArtistPageSort,
    pagination: Pagination,
) -> QueryBuilder<Sqlite> {
    let mut query = QueryBuilder::new(ARTIST_SELECT_FROM_ACTIVE_GENERATION);
    query.push_bind(profile_id.to_owned());

    push_artist_page_search(&mut query, search);
    push_artist_page_sort(&mut query, sort);
    push_artist_page_pagination(&mut query, pagination);

    query
}

fn push_artist_page_sort(query: &mut QueryBuilder<Sqlite>, sort: ArtistPageSort) {
    query.push(" ").push(match sort {
        ArtistPageSort::Name => "ORDER BY a.name COLLATE NOCASE",
        ArtistPageSort::MostAlbums => "ORDER BY a.album_count DESC, a.name COLLATE NOCASE",
        ArtistPageSort::FewestAlbums => "ORDER BY a.album_count, a.name COLLATE NOCASE",
    });
}

fn push_artist_page_pagination(query: &mut QueryBuilder<Sqlite>, pagination: Pagination) {
    let pagination = pagination.normalized();
    query.push(" LIMIT ").push_bind(pagination.limit);
    query.push(" OFFSET ").push_bind(pagination.offset);
}

fn push_artist_page_search(query: &mut QueryBuilder<Sqlite>, search: &str) {
    let search = search.trim();
    if !search.is_empty() {
        query.push(" AND INSTR(LOWER(a.name), LOWER(");
        query.push_bind(search.to_owned()).push(")) > 0");
    }
}

async fn count_artists(
    repo: &SqliteRepository,
    profile_id: &str,
    search: &str,
) -> Result<i64, String> {
    let mut query = QueryBuilder::new(
        "SELECT COUNT(*) FROM artists a JOIN library_sync_state s
         ON s.profile_id = a.profile_id AND s.active_generation = a.generation WHERE a.profile_id = ",
    );
    query.push_bind(profile_id);
    push_artist_page_search(&mut query, search);

    query
        .build_query_scalar::<i64>()
        .fetch_one(&repo.pool)
        .await
        .map_err(|error| format!("Failed to count filtered artists: {error}"))
}

pub(crate) async fn insert_artists(
    transaction: &mut Transaction<'_, Sqlite>,
    profile_id: &str,
    generation: &str,
    artists: &[Artist],
) -> Result<(), String> {
    for artists in artists.chunks(SQLITE_BIND_LIMIT / 8) {
        let mut query = QueryBuilder::new(
            "INSERT INTO artists
             (profile_id, generation, remote_id, name, album_count, cover_art_id, favorite, rating) ",
        );
        query.push_values(artists, |mut row, artist| {
            row.push_bind(profile_id)
                .push_bind(generation)
                .push_bind(&artist.remote_id)
                .push_bind(&artist.name)
                .push_bind(artist.album_count)
                .push_bind(&artist.cover_art_id)
                .push_bind(artist.favorite)
                .push_bind(artist.rating);
        });
        query
            .build()
            .execute(&mut **transaction)
            .await
            .map_err(|error| format!("Failed to cache artists: {error}"))?;
    }
    Ok(())
}

pub(crate) async fn insert_artist_search(
    transaction: &mut Transaction<'_, Sqlite>,
    profile_id: &str,
    generation: &str,
    artists: &[Artist],
) -> Result<(), String> {
    for artists in artists.chunks(SQLITE_BIND_LIMIT / 4) {
        let mut query = QueryBuilder::new(
            "INSERT INTO artist_search
             (profile_id, generation, remote_id, name) ",
        );
        query.push_values(artists, |mut row, artist| {
            row.push_bind(profile_id)
                .push_bind(generation)
                .push_bind(&artist.remote_id)
                .push_bind(&artist.name);
        });
        query
            .build()
            .execute(&mut **transaction)
            .await
            .map_err(|error| format!("Failed to index artists for search: {error}"))?;
    }
    Ok(())
}

async fn fuzzy_artist_candidates(
    repo: &SqliteRepository,
    profile_id: &str,
) -> Result<Vec<CachedArtist>, String> {
    sqlx::query_as::<_, CachedArtist>(
        "SELECT artist.remote_id, artist.name, artist.album_count,
                artwork.local_path AS artwork_path, artist.favorite, artist.rating
         FROM artists artist
         JOIN library_sync_state state
           ON state.profile_id = artist.profile_id
          AND state.active_generation = artist.generation
         LEFT JOIN artwork_cache artwork
           ON artwork.profile_id = artist.profile_id
          AND artwork.kind = 'artist'
          AND artwork.remote_id = artist.remote_id
         WHERE artist.profile_id = ?",
    )
    .bind(profile_id)
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read fuzzy artist candidates: {error}"))
}

pub(crate) async fn search_artists(
    repo: &SqliteRepository,
    profile_id: &str,
    query: &str,
    limit: i64,
) -> Result<Vec<CachedArtist>, String> {
    let Some(fts_query) = super::search_query(query) else {
        return Ok(Vec::new());
    };
    let limit = limit.clamp(1, 500);
    let results = sqlx::query_as::<_, CachedArtist>(
        "SELECT artist.remote_id, artist.name, artist.album_count,
                artwork.local_path AS artwork_path, artist.favorite, artist.rating
         FROM artist_search
         JOIN library_sync_state state
           ON state.profile_id = artist_search.profile_id
          AND state.active_generation = artist_search.generation
         JOIN artists artist
           ON artist.profile_id = artist_search.profile_id
          AND artist.generation = artist_search.generation
          AND artist.remote_id = artist_search.remote_id
         LEFT JOIN artwork_cache artwork
           ON artwork.profile_id = artist.profile_id
          AND artwork.kind = 'artist'
          AND artwork.remote_id = artist.remote_id
         WHERE artist_search.profile_id = ?
           AND artist_search MATCH ?
         ORDER BY bm25(artist_search, 0.0, 0.0, 0.0, 6.0),
                  artist.name COLLATE NOCASE
         LIMIT ?",
    )
    .bind(profile_id)
    .bind(fts_query)
    .bind(limit)
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to search cached artists: {error}"))?;

    let Some(query) = fuzzy_search::should_use_fuzzy(query, results.len(), limit) else {
        return Ok(results);
    };

    let candidates = fuzzy_artist_candidates(repo, profile_id).await?;
    let fuzzy_results = fuzzy_search::rank_artists(&query, candidates, limit);
    Ok(fuzzy_search::merge_artists(results, fuzzy_results, limit))
}

pub(crate) async fn insert_genres(
    transaction: &mut Transaction<'_, Sqlite>,
    profile_id: &str,
    generation: &str,
    genres: &[Genre],
) -> Result<(), String> {
    for genres in genres.chunks(SQLITE_BIND_LIMIT / 5) {
        let mut query = QueryBuilder::new(
            "INSERT INTO genres
             (profile_id, generation, name, song_count, album_count) ",
        );
        query.push_values(genres, |mut row, genre| {
            row.push_bind(profile_id)
                .push_bind(generation)
                .push_bind(&genre.name)
                .push_bind(genre.song_count)
                .push_bind(genre.album_count);
        });
        query
            .build()
            .execute(&mut **transaction)
            .await
            .map_err(|error| format!("Failed to cache genres: {error}"))?;
    }
    Ok(())
}
