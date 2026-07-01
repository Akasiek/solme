use crate::database::SqliteRepository;

pub(crate) async fn delete_stale_generations(
    repo: &SqliteRepository,
    profile_id: &str,
    generation: &str,
) {
    let _ = sqlx::query!(
        "DELETE FROM album_search WHERE profile_id = ? AND generation <> ?",
        profile_id,
        generation,
    )
    .execute(&repo.pool)
    .await;
    let _ = sqlx::query!(
        "DELETE FROM song_search WHERE profile_id = ? AND generation <> ?",
        profile_id,
        generation,
    )
    .execute(&repo.pool)
    .await;
    let _ = sqlx::query!(
        "DELETE FROM albums WHERE profile_id = ? AND generation <> ?",
        profile_id,
        generation,
    )
    .execute(&repo.pool)
    .await;
    let _ = sqlx::query!(
        "DELETE FROM genres WHERE profile_id = ? AND generation <> ?",
        profile_id,
        generation,
    )
    .execute(&repo.pool)
    .await;
    let _ = sqlx::query!(
        "DELETE FROM artists WHERE profile_id = ? AND generation <> ?",
        profile_id,
        generation,
    )
    .execute(&repo.pool)
    .await;
}
