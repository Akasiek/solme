use sqlx::{QueryBuilder, Sqlite, Transaction};

use super::super::{
    fuzzy_search,
    models::{CachedSong, Song},
};
use crate::database::SqliteRepository;

const SQLITE_BIND_LIMIT: usize = 999;

pub(crate) async fn insert_songs(
    transaction: &mut Transaction<'_, Sqlite>,
    profile_id: &str,
    generation: &str,
    songs: &[&Song],
) -> Result<(), String> {
    for songs in songs.chunks(SQLITE_BIND_LIMIT / 15) {
        let mut query = QueryBuilder::new(
            "INSERT INTO songs
             (profile_id, generation, remote_id, album_id, artist_id, title,
              artist_name, album_name, track_number, disc_number, year,
              duration_seconds, suffix, content_type, cover_art_id) ",
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
                .push_bind(&song.cover_art_id);
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
                song.track_number, song.disc_number, song.duration_seconds
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
    let results = sqlx::query_as!(
        CachedSong,
        "SELECT song.remote_id, song.album_id, song.artist_id, song.title, song.artist_name,
                song.album_name, artwork.local_path AS artwork_path,
                song.track_number, song.disc_number, song.duration_seconds
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
        profile_id,
        fts_query,
        limit,
    )
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
