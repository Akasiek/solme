use sqlx::Sqlite;

use crate::database::SqliteRepository;

use super::super::models::GenreSummary;

pub(crate) async fn genres(
    repo: &SqliteRepository,
    profile_id: &str,
) -> Result<Vec<GenreSummary>, String> {
    sqlx::query_as::<Sqlite, GenreSummary>(
        "SELECT ag.genre AS name, COUNT(DISTINCT ag.album_id) AS album_count
         FROM album_genres ag
         JOIN library_sync_state sync
           ON sync.profile_id = ag.profile_id
          AND sync.active_generation = ag.generation
         WHERE ag.profile_id = ? AND TRIM(ag.genre) != ''
         GROUP BY ag.genre
         HAVING COUNT(DISTINCT ag.album_id) > 0
         ORDER BY album_count DESC, ag.genre COLLATE NOCASE",
    )
    .bind(profile_id)
    .fetch_all(&repo.pool)
    .await
    .map_err(|error| format!("Failed to read cached genres: {error}"))
}
