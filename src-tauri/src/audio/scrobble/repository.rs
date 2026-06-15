use async_trait::async_trait;

use crate::database::SqliteRepository;

#[derive(Clone, Debug, Eq, PartialEq, sqlx::FromRow)]
pub struct PendingScrobble {
    pub id: i64,
    pub profile_id: String,
    pub song_id: String,
    pub started_at_ms: i64,
    pub attempts: i64,
}

#[async_trait]
pub trait ScrobbleRepository: Send + Sync {
    async fn enqueue(
        &self,
        profile_id: &str,
        song_id: &str,
        started_at_ms: i64,
        next_attempt_at_ms: i64,
    ) -> Result<(), String>;
    async fn due(
        &self,
        profile_id: &str,
        now_ms: i64,
        limit: i64,
    ) -> Result<Vec<PendingScrobble>, String>;
    async fn complete(&self, id: i64) -> Result<(), String>;
    async fn reschedule(
        &self,
        id: i64,
        attempts: i64,
        next_attempt_at_ms: i64,
    ) -> Result<(), String>;
}

#[async_trait]
impl ScrobbleRepository for SqliteRepository {
    async fn enqueue(
        &self,
        profile_id: &str,
        song_id: &str,
        started_at_ms: i64,
        next_attempt_at_ms: i64,
    ) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO pending_scrobbles
             (profile_id, song_id, started_at_ms, next_attempt_at_ms)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(profile_id, song_id, started_at_ms) DO NOTHING",
        )
        .bind(profile_id)
        .bind(song_id)
        .bind(started_at_ms)
        .bind(next_attempt_at_ms)
        .execute(&self.pool)
        .await
        .map_err(|error| format!("Failed to enqueue scrobble: {error}"))?;
        Ok(())
    }

    async fn due(
        &self,
        profile_id: &str,
        now_ms: i64,
        limit: i64,
    ) -> Result<Vec<PendingScrobble>, String> {
        sqlx::query_as::<_, PendingScrobble>(
            "SELECT id, profile_id, song_id, started_at_ms, attempts
             FROM pending_scrobbles
             WHERE profile_id = ? AND next_attempt_at_ms <= ?
             ORDER BY next_attempt_at_ms, id
             LIMIT ?",
        )
        .bind(profile_id)
        .bind(now_ms)
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await
        .map_err(|error| format!("Failed to read pending scrobbles: {error}"))
    }

    async fn complete(&self, id: i64) -> Result<(), String> {
        sqlx::query("DELETE FROM pending_scrobbles WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|error| format!("Failed to complete scrobble: {error}"))?;
        Ok(())
    }

    async fn reschedule(
        &self,
        id: i64,
        attempts: i64,
        next_attempt_at_ms: i64,
    ) -> Result<(), String> {
        sqlx::query(
            "UPDATE pending_scrobbles
             SET attempts = ?, next_attempt_at_ms = ?
             WHERE id = ?",
        )
        .bind(attempts)
        .bind(next_attempt_at_ms)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|error| format!("Failed to reschedule scrobble: {error}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use uuid::Uuid;

    use super::ScrobbleRepository;
    use crate::database::{SqliteRepository, DATABASE_FILE_NAME};

    #[test]
    fn persists_and_deduplicates_pending_scrobbles() {
        tauri::async_runtime::block_on(async {
            let directory = std::env::temp_dir().join(format!("solme-scrobble-{}", Uuid::new_v4()));
            let path = directory.join(DATABASE_FILE_NAME);
            let repository = SqliteRepository::open(&path).await.unwrap();

            repository
                .enqueue("profile", "song-1", 1000, 1000)
                .await
                .unwrap();
            repository
                .enqueue("profile", "song-1", 1000, 1000)
                .await
                .unwrap();

            let pending = repository.due("profile", 1000, 20).await.unwrap();
            assert_eq!(pending.len(), 1);
            assert_eq!(pending[0].song_id, "song-1");

            repository.complete(pending[0].id).await.unwrap();
            assert!(repository
                .due("profile", 1000, 20)
                .await
                .unwrap()
                .is_empty());

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn reschedules_failed_scrobble() {
        tauri::async_runtime::block_on(async {
            let directory = std::env::temp_dir().join(format!("solme-scrobble-{}", Uuid::new_v4()));
            let path = directory.join(DATABASE_FILE_NAME);
            let repository = SqliteRepository::open(&path).await.unwrap();
            repository
                .enqueue("profile", "song-1", 1000, 1000)
                .await
                .unwrap();

            let pending = repository.due("profile", 1000, 20).await.unwrap();
            repository.reschedule(pending[0].id, 1, 5000).await.unwrap();

            assert!(repository
                .due("profile", 4999, 20)
                .await
                .unwrap()
                .is_empty());
            let pending = repository.due("profile", 5000, 20).await.unwrap();
            assert_eq!(pending[0].attempts, 1);

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }
}
