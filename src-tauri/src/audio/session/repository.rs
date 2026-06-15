use async_trait::async_trait;

use crate::database::SqliteRepository;

use super::PlaybackSession;

#[async_trait]
pub trait PlaybackSessionRepository: Send + Sync {
    async fn load(&self, profile_id: &str) -> Result<Option<PlaybackSession>, String>;
    async fn save(&self, profile_id: &str, session: Option<&PlaybackSession>)
        -> Result<(), String>;
}

#[async_trait]
impl PlaybackSessionRepository for SqliteRepository {
    async fn load(&self, profile_id: &str) -> Result<Option<PlaybackSession>, String> {
        let row = sqlx::query_as::<_, StoredPlaybackSession>(
            "SELECT queue_json, active_index, position_seconds
             FROM playback_sessions
             WHERE profile_id = ?",
        )
        .bind(profile_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| format!("Failed to read playback session: {error}"))?;

        row.map(TryInto::try_into).transpose()
    }

    async fn save(
        &self,
        profile_id: &str,
        session: Option<&PlaybackSession>,
    ) -> Result<(), String> {
        let Some(session) = session else {
            sqlx::query("DELETE FROM playback_sessions WHERE profile_id = ?")
                .bind(profile_id)
                .execute(&self.pool)
                .await
                .map_err(|error| format!("Failed to delete playback session: {error}"))?;
            return Ok(());
        };

        let queue_json = serde_json::to_string(&session.queue)
            .map_err(|error| format!("Failed to serialize playback queue: {error}"))?;
        let active_index = i64::try_from(session.active_index)
            .map_err(|_| "Playback queue index is too large".to_string())?;

        sqlx::query(
            "INSERT INTO playback_sessions
             (profile_id, queue_json, active_index, position_seconds)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(profile_id) DO UPDATE SET
               queue_json = excluded.queue_json,
               active_index = excluded.active_index,
               position_seconds = excluded.position_seconds",
        )
        .bind(profile_id)
        .bind(queue_json)
        .bind(active_index)
        .bind(session.position_seconds)
        .execute(&self.pool)
        .await
        .map_err(|error| format!("Failed to save playback session: {error}"))?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct StoredPlaybackSession {
    queue_json: String,
    active_index: i64,
    position_seconds: f64,
}

impl TryFrom<StoredPlaybackSession> for PlaybackSession {
    type Error = String;

    fn try_from(stored: StoredPlaybackSession) -> Result<Self, Self::Error> {
        let queue = serde_json::from_str(&stored.queue_json)
            .map_err(|error| format!("Failed to deserialize playback queue: {error}"))?;
        let active_index = usize::try_from(stored.active_index)
            .map_err(|_| "Stored playback queue index is invalid".to_string())?;

        Ok(Self {
            queue,
            active_index,
            position_seconds: stored.position_seconds.max(0.0),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use uuid::Uuid;

    use super::PlaybackSessionRepository;
    use crate::{
        audio::session::PlaybackSession,
        database::{SqliteRepository, DATABASE_FILE_NAME},
        library::CachedSong,
    };

    #[test]
    fn saves_loads_and_deletes_playback_session() {
        tauri::async_runtime::block_on(async {
            let directory =
                std::env::temp_dir().join(format!("solme-playback-session-{}", Uuid::new_v4()));
            let repository = SqliteRepository::open(&directory.join(DATABASE_FILE_NAME))
                .await
                .unwrap();
            let session = PlaybackSession {
                queue: vec![song("song-1"), song("song-2")],
                active_index: 1,
                position_seconds: 42.5,
            };

            repository.save("profile", Some(&session)).await.unwrap();
            assert_eq!(repository.load("profile").await.unwrap(), Some(session));

            repository.save("profile", None).await.unwrap();
            assert_eq!(repository.load("profile").await.unwrap(), None);

            repository.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }

    fn song(id: &str) -> CachedSong {
        CachedSong {
            remote_id: id.to_string(),
            album_id: "album".to_string(),
            title: id.to_string(),
            artist_name: "Artist".to_string(),
            album_name: "Album".to_string(),
            artwork_path: None,
            track_number: Some(1),
            disc_number: Some(1),
            duration_seconds: 180,
        }
    }
}
