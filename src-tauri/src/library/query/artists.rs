use sqlx::{QueryBuilder, Sqlite, Transaction};

use super::super::models::{Artist, Genre};

const SQLITE_BIND_LIMIT: usize = 999;

pub(crate) async fn insert_artists(
    transaction: &mut Transaction<'_, Sqlite>,
    profile_id: &str,
    generation: &str,
    artists: &[Artist],
) -> Result<(), String> {
    for artists in artists.chunks(SQLITE_BIND_LIMIT / 5) {
        let mut query = QueryBuilder::new(
            "INSERT INTO artists
             (profile_id, generation, remote_id, name, album_count) ",
        );
        query.push_values(artists, |mut row, artist| {
            row.push_bind(profile_id)
                .push_bind(generation)
                .push_bind(&artist.remote_id)
                .push_bind(&artist.name)
                .push_bind(artist.album_count);
        });
        query
            .build()
            .execute(&mut **transaction)
            .await
            .map_err(|error| format!("Failed to cache artists: {error}"))?;
    }
    Ok(())
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
