use sqlx::{QueryBuilder, Sqlite, Transaction};

use super::super::{
    fuzzy_search,
    models::{
        CachedSong, CatalogFilter, Paginated, Pagination, Song, SongPage, SongPageItem,
        SongPageSort,
    },
};
use crate::database::SqliteRepository;

const SQLITE_BIND_LIMIT: usize = 999;

const SONG_PAGE_FROM: &str = " FROM songs song
    JOIN library_sync_state state ON state.profile_id = song.profile_id
        AND state.active_generation = song.generation
    LEFT JOIN albums album ON album.profile_id = song.profile_id
        AND album.generation = song.generation AND album.remote_id = song.album_id
    LEFT JOIN artwork_cache artwork ON artwork.profile_id = song.profile_id
        AND artwork.kind = 'album' AND artwork.remote_id = song.album_id
    WHERE song.profile_id = ";

pub(crate) async fn song_page(
    repo: &SqliteRepository,
    profile_id: &str,
    search: &str,
    filters: CatalogFilter,
    sort: SongPageSort,
    pagination: Pagination,
) -> Result<SongPage, String> {
    let mut items_query = QueryBuilder::new(
        "SELECT song.remote_id, song.album_id, song.artist_id, song.title,
            song.artist_name, song.album_name, artwork.local_path AS artwork_path,
            song.track_number, song.disc_number, song.duration_seconds,
            song.favorite, song.rating, album.release_date,
            album.original_release_date, song.year",
    );
    items_query
        .push(SONG_PAGE_FROM)
        .push_bind(profile_id.to_owned());
    push_song_page_filters(&mut items_query, search, filters.clone());
    items_query.push(" ").push(match sort {
        SongPageSort::Title => "ORDER BY song.title COLLATE NOCASE, song.artist_name COLLATE NOCASE, song.remote_id",
        SongPageSort::Artist => "ORDER BY song.artist_name COLLATE NOCASE, song.album_name COLLATE NOCASE, song.disc_number, song.track_number, song.remote_id",
        SongPageSort::Album => "ORDER BY song.album_name COLLATE NOCASE, song.disc_number, song.track_number, song.remote_id",
        SongPageSort::Newest => "ORDER BY COALESCE(album.original_release_date, album.release_date, printf('%04d', song.year)) IS NULL, COALESCE(album.original_release_date, album.release_date, printf('%04d', song.year)) DESC, song.title COLLATE NOCASE, song.remote_id",
        SongPageSort::Oldest => "ORDER BY COALESCE(album.original_release_date, album.release_date, printf('%04d', song.year)) IS NULL, COALESCE(album.original_release_date, album.release_date, printf('%04d', song.year)), song.title COLLATE NOCASE, song.remote_id",
    });
    let pagination = pagination.normalized();
    items_query
        .push(" LIMIT ")
        .push_bind(pagination.limit)
        .push(" OFFSET ")
        .push_bind(pagination.offset);
    let items = items_query
        .build_query_as::<SongPageItem>()
        .fetch_all(&repo.pool)
        .await
        .map_err(|error| format!("Failed to read song page: {error}"))?;

    let mut count_query = QueryBuilder::new("SELECT COUNT(*)");
    count_query
        .push(SONG_PAGE_FROM)
        .push_bind(profile_id.to_owned());
    push_song_page_filters(&mut count_query, search, filters);
    let total = count_query
        .build_query_scalar::<i64>()
        .fetch_one(&repo.pool)
        .await
        .map_err(|error| format!("Failed to count filtered songs: {error}"))?;

    let genres = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT sg.genre FROM song_genres sg
         JOIN library_sync_state state ON state.profile_id = sg.profile_id
            AND state.active_generation = sg.generation
         WHERE sg.profile_id = ? AND TRIM(sg.genre) != ''
         ORDER BY sg.genre COLLATE NOCASE",
    )
    .bind(profile_id)
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read song genres: {error}"))?;

    Ok(SongPage {
        page: Paginated { items, total },
        genres,
    })
}

fn push_song_page_filters(query: &mut QueryBuilder<Sqlite>, search: &str, filters: CatalogFilter) {
    let search = search.trim();
    let filters = filters.normalized();
    if !search.is_empty() {
        query
            .push(" AND (INSTR(LOWER(song.title), LOWER(")
            .push_bind(search.to_owned())
            .push(")) > 0 OR INSTR(LOWER(song.artist_name), LOWER(")
            .push_bind(search.to_owned())
            .push(")) > 0 OR INSTR(LOWER(song.album_name), LOWER(")
            .push_bind(search.to_owned())
            .push(")) > 0)");
    }
    if filters.favorite_only {
        query.push(" AND song.favorite = 1");
    }
    if let Some(rating) = filters.minimum_rating {
        query.push(" AND song.rating >= ").push_bind(rating);
    }
    if filters.unrated_only {
        query.push(" AND song.rating IS NULL");
    }
    if let Some(year) = filters.from_year {
        query.push(" AND COALESCE(CAST(SUBSTR(COALESCE(album.original_release_date, album.release_date), 1, 4) AS INTEGER), song.year) >= ").push_bind(year);
    }
    if let Some(year) = filters.to_year {
        query.push(" AND COALESCE(CAST(SUBSTR(COALESCE(album.original_release_date, album.release_date), 1, 4) AS INTEGER), song.year) <= ").push_bind(year);
    }
    if !filters.genres.is_empty() {
        query.push(
            " AND EXISTS (SELECT 1 FROM song_genres sg
            WHERE sg.profile_id = song.profile_id AND sg.generation = song.generation
                AND sg.song_id = song.remote_id AND sg.genre IN (",
        );
        let mut genres = query.separated(", ");
        for genre in filters.genres {
            genres.push_bind(genre);
        }
        genres.push_unseparated("))");
    }
}

pub(crate) async fn insert_songs(
    transaction: &mut Transaction<'_, Sqlite>,
    profile_id: &str,
    generation: &str,
    songs: &[&Song],
) -> Result<(), String> {
    for songs in songs.chunks(SQLITE_BIND_LIMIT / 20) {
        let mut query = QueryBuilder::new(
            "INSERT INTO songs
             (profile_id, generation, remote_id, album_id, artist_id, title,
              artist_name, album_name, track_number, disc_number, year,
              duration_seconds, suffix, content_type, bit_rate, bit_depth, sample_rate,
              cover_art_id, favorite, rating) ",
        );
        query.push_values(songs, |mut row, song| {
            row.push_bind(profile_id)
                .push_bind(generation)
                .push_bind(&song.remote_id)
                .push_bind(&song.album_id)
                .push_bind(&song.artist_id)
                .push_bind(&song.title)
                .push_bind(&song.artist_name)
                .push_bind(&song.album_name)
                .push_bind(song.track_number)
                .push_bind(song.disc_number)
                .push_bind(song.year)
                .push_bind(song.duration_seconds)
                .push_bind(&song.suffix)
                .push_bind(&song.content_type)
                .push_bind(song.bit_rate)
                .push_bind(song.bit_depth)
                .push_bind(song.sample_rate)
                .push_bind(&song.cover_art_id)
                .push_bind(song.favorite)
                .push_bind(song.rating);
        });
        query
            .build()
            .execute(&mut **transaction)
            .await
            .map_err(|error| format!("Failed to cache songs: {error}"))?;
    }
    Ok(())
}

pub(crate) async fn insert_song_genres(
    transaction: &mut Transaction<'_, Sqlite>,
    profile_id: &str,
    generation: &str,
    songs: &[&Song],
) -> Result<(), String> {
    let genres = songs
        .iter()
        .flat_map(|song| song.genres.iter().map(|genre| (&song.remote_id, genre)))
        .collect::<Vec<_>>();

    for genres in genres.chunks(SQLITE_BIND_LIMIT / 4) {
        let mut query = QueryBuilder::new(
            "INSERT OR IGNORE INTO song_genres
             (profile_id, generation, song_id, genre) ",
        );
        query.push_values(genres, |mut row, (song_id, genre)| {
            row.push_bind(profile_id)
                .push_bind(generation)
                .push_bind(song_id)
                .push_bind(genre);
        });
        query
            .build()
            .execute(&mut **transaction)
            .await
            .map_err(|error| format!("Failed to cache song genres: {error}"))?;
    }
    Ok(())
}

pub(crate) async fn insert_song_search(
    transaction: &mut Transaction<'_, Sqlite>,
    profile_id: &str,
    generation: &str,
    songs: &[&Song],
) -> Result<(), String> {
    for songs in songs.chunks(SQLITE_BIND_LIMIT / 8) {
        let mut query = QueryBuilder::new(
            "INSERT INTO song_search
             (profile_id, generation, remote_id, album_id, title,
              artist_name, album_name, genres) ",
        );
        query.push_values(songs, |mut row, song| {
            row.push_bind(profile_id)
                .push_bind(generation)
                .push_bind(&song.remote_id)
                .push_bind(&song.album_id)
                .push_bind(&song.title)
                .push_bind(&song.artist_name)
                .push_bind(&song.album_name)
                .push_bind(song.genres.join(" "));
        });
        query
            .build()
            .execute(&mut **transaction)
            .await
            .map_err(|error| format!("Failed to index songs for search: {error}"))?;
    }
    Ok(())
}

pub(crate) async fn fuzzy_song_candidates(
    repo: &SqliteRepository,
    profile_id: &str,
) -> Result<Vec<CachedSong>, String> {
    sqlx::query_as::<_, CachedSong>(
        "SELECT song.remote_id, song.album_id, song.artist_id, song.title, song.artist_name,
                song.album_name, artwork.local_path AS artwork_path,
                song.track_number, song.disc_number, song.duration_seconds,
                song.favorite, song.rating
         FROM songs song
         JOIN library_sync_state state
           ON state.profile_id = song.profile_id
          AND state.active_generation = song.generation
         LEFT JOIN artwork_cache artwork
           ON artwork.profile_id = song.profile_id
          AND artwork.kind = 'album'
          AND artwork.remote_id = song.album_id
         WHERE song.profile_id = ?",
    )
    .bind(profile_id)
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read fuzzy song candidates: {error}"))
}

pub(crate) async fn songs(
    repo: &SqliteRepository,
    profile_id: &str,
    album_id: &str,
) -> Result<Vec<CachedSong>, String> {
    sqlx::query_as::<_, CachedSong>(
        "SELECT song.remote_id, song.album_id, song.artist_id, song.title, song.artist_name,
                song.album_name, artwork.local_path AS artwork_path,
                song.track_number, song.disc_number, song.duration_seconds,
                song.favorite, song.rating
         FROM songs song
         JOIN library_sync_state state
           ON state.profile_id = song.profile_id
          AND state.active_generation = song.generation
         LEFT JOIN artwork_cache artwork
           ON artwork.profile_id = song.profile_id
          AND artwork.kind = 'album'
          AND artwork.remote_id = song.album_id
         WHERE song.profile_id = ? AND song.album_id = ?
         ORDER BY COALESCE(song.disc_number, 1),
                  COALESCE(song.track_number, 2147483647),
                  song.title COLLATE NOCASE",
    )
    .bind(profile_id)
    .bind(album_id)
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read cached songs: {error}"))
}

pub(crate) async fn search_songs(
    repo: &SqliteRepository,
    profile_id: &str,
    query: &str,
    limit: i64,
) -> Result<Vec<CachedSong>, String> {
    let Some(fts_query) = super::search_query(query) else {
        return Ok(Vec::new());
    };
    let limit = limit.clamp(1, 500);
    let results = sqlx::query_as::<_, CachedSong>(
        "SELECT song.remote_id, song.album_id, song.artist_id, song.title, song.artist_name,
                song.album_name, artwork.local_path AS artwork_path,
                song.track_number, song.disc_number, song.duration_seconds,
                song.favorite, song.rating
         FROM song_search
         JOIN library_sync_state state
           ON state.profile_id = song_search.profile_id
          AND state.active_generation = song_search.generation
         JOIN songs song
           ON song.profile_id = song_search.profile_id
          AND song.generation = song_search.generation
          AND song.remote_id = song_search.remote_id
         LEFT JOIN artwork_cache artwork
           ON artwork.profile_id = song.profile_id
          AND artwork.kind = 'album'
          AND artwork.remote_id = song.album_id
         WHERE song_search.profile_id = ?
           AND song_search MATCH ?
         ORDER BY bm25(song_search, 0.0, 0.0, 0.0, 0.0, 6.0, 4.0, 2.0, 1.0),
                  song.artist_name COLLATE NOCASE,
                  song.album_name COLLATE NOCASE,
                  song.disc_number,
                  song.track_number,
                  song.title COLLATE NOCASE
         LIMIT ?",
    )
    .bind(profile_id)
    .bind(fts_query)
    .bind(limit)
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to search cached songs: {error}"))?;

    let Some(query) = fuzzy_search::should_use_fuzzy(query, results.len(), limit) else {
        return Ok(results);
    };

    let candidates = fuzzy_song_candidates(repo, profile_id).await?;
    let fuzzy_results = fuzzy_search::rank_songs(&query, candidates, limit);
    Ok(fuzzy_search::merge_songs(results, fuzzy_results, limit))
}
