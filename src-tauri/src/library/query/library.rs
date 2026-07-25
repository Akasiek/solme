use crate::database::SqliteRepository;
use crate::library::models::{LibrarySnapshot, LibrarySummary};

pub(crate) async fn server_revision(
    repo: &SqliteRepository,
    profile_id: &str,
) -> Result<Option<String>, String> {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT server_revision FROM library_sync_state WHERE profile_id = ?",
    )
    .bind(profile_id)
    .fetch_optional(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read library revision: {error}"))
    .map(Option::flatten)
}

pub(crate) async fn activate_snapshot(
    repo: &SqliteRepository,
    profile_id: &str,
    generation: &str,
    revision: Option<&str>,
    snapshot: &LibrarySnapshot,
    completed_at: i64,
) -> Result<(), String> {
    let mut transaction = repo
        .pool
        .begin()
        .await
        .map_err(|error| format!("Failed to begin library transaction: {error}"))?;

    super::insert_artists(&mut transaction, profile_id, generation, &snapshot.artists).await?;
    super::insert_artist_search(&mut transaction, profile_id, generation, &snapshot.artists)
        .await?;
    super::insert_genres(&mut transaction, profile_id, generation, &snapshot.genres).await?;
    super::insert_albums(&mut transaction, profile_id, generation, &snapshot.albums).await?;
    super::insert_album_genres(&mut transaction, profile_id, generation, &snapshot.albums).await?;
    super::insert_album_search(&mut transaction, profile_id, generation, &snapshot.albums).await?;

    let songs = snapshot
        .albums
        .iter()
        .flat_map(|details| &details.songs)
        .collect::<Vec<_>>();
    super::insert_songs(&mut transaction, profile_id, generation, &songs).await?;
    super::insert_song_genres(&mut transaction, profile_id, generation, &songs).await?;
    super::insert_song_search(&mut transaction, profile_id, generation, &songs).await?;
    let song_count = songs.len() as i64;

    sqlx::query(
        "INSERT INTO library_sync_state
         (profile_id, active_generation, server_revision, last_success_at,
          artist_count, album_count, song_count, genre_count)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(profile_id) DO UPDATE SET
           active_generation = excluded.active_generation,
           server_revision = excluded.server_revision,
           last_success_at = excluded.last_success_at,
           artist_count = excluded.artist_count,
           album_count = excluded.album_count,
           song_count = excluded.song_count,
           genre_count = excluded.genre_count",
    )
    .bind(profile_id)
    .bind(generation)
    .bind(revision)
    .bind(completed_at)
    .bind(snapshot.artists.len() as i64)
    .bind(snapshot.albums.len() as i64)
    .bind(song_count)
    .bind(snapshot.genres.len() as i64)
    .execute(&mut *transaction)
    .await
    .map_err(|error| format!("Failed to activate library generation: {error}"))?;

    transaction
        .commit()
        .await
        .map_err(|error| format!("Failed to commit library generation: {error}"))?;

    delete_stale_generations(repo, profile_id, generation).await;
    Ok(())
}

pub(crate) async fn summary(
    repo: &SqliteRepository,
    profile_id: &str,
) -> Result<LibrarySummary, String> {
    let summary = sqlx::query_as::<_, LibrarySummary>(
        "SELECT artist_count, album_count, song_count, genre_count, last_success_at
         FROM library_sync_state WHERE profile_id = ?",
    )
    .bind(profile_id)
    .fetch_optional(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read library summary: {error}"))?;

    Ok(summary.unwrap_or(LibrarySummary {
        artist_count: 0,
        album_count: 0,
        song_count: 0,
        genre_count: 0,
        last_success_at: None,
    }))
}

pub(crate) async fn delete_stale_generations(
    repo: &SqliteRepository,
    profile_id: &str,
    generation: &str,
) {
    let _ = sqlx::query("DELETE FROM artist_search WHERE profile_id = ? AND generation <> ?")
        .bind(profile_id)
        .bind(generation)
        .execute(&repo.pool)
        .await;
    let _ = sqlx::query("DELETE FROM album_search WHERE profile_id = ? AND generation <> ?")
        .bind(profile_id)
        .bind(generation)
        .execute(&repo.pool)
        .await;
    let _ = sqlx::query("DELETE FROM song_search WHERE profile_id = ? AND generation <> ?")
        .bind(profile_id)
        .bind(generation)
        .execute(&repo.pool)
        .await;
    let _ = sqlx::query("DELETE FROM albums WHERE profile_id = ? AND generation <> ?")
        .bind(profile_id)
        .bind(generation)
        .execute(&repo.pool)
        .await;
    let _ = sqlx::query("DELETE FROM genres WHERE profile_id = ? AND generation <> ?")
        .bind(profile_id)
        .bind(generation)
        .execute(&repo.pool)
        .await;
    let _ = sqlx::query("DELETE FROM artists WHERE profile_id = ? AND generation <> ?")
        .bind(profile_id)
        .bind(generation)
        .execute(&repo.pool)
        .await;
}
