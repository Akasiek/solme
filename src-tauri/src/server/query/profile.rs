use sqlx::{Row, SqlitePool};

use super::super::models::{ServerType, StoredServerProfile};

const ACTIVE_SERVER_PROFILE_KEY: &str = "active_server_profile_id";
const PROFILE_DATA_DELETE_STATEMENTS: &[(&str, &str)] = &[
    (
        "song_search",
        "DELETE FROM song_search WHERE profile_id = ?",
    ),
    (
        "album_search",
        "DELETE FROM album_search WHERE profile_id = ?",
    ),
    (
        "artist_search",
        "DELETE FROM artist_search WHERE profile_id = ?",
    ),
    (
        "song_genres",
        "DELETE FROM song_genres WHERE profile_id = ?",
    ),
    (
        "album_genres",
        "DELETE FROM album_genres WHERE profile_id = ?",
    ),
    ("songs", "DELETE FROM songs WHERE profile_id = ?"),
    ("albums", "DELETE FROM albums WHERE profile_id = ?"),
    ("artists", "DELETE FROM artists WHERE profile_id = ?"),
    ("genres", "DELETE FROM genres WHERE profile_id = ?"),
    (
        "artwork_cache",
        "DELETE FROM artwork_cache WHERE profile_id = ?",
    ),
    (
        "library_sync_state",
        "DELETE FROM library_sync_state WHERE profile_id = ?",
    ),
    (
        "pending_scrobbles",
        "DELETE FROM pending_scrobbles WHERE profile_id = ?",
    ),
    (
        "playback_sessions",
        "DELETE FROM playback_sessions WHERE profile_id = ?",
    ),
    (
        "preferences",
        "DELETE FROM preferences WHERE profile_id = ?",
    ),
];

pub(in crate::server) async fn load_profile(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<StoredServerProfile>, String> {
    let row = sqlx::query(
        "SELECT id, server_type, url, secondary_url, username
         FROM server_profiles
         WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|error| format!("Failed to load server profile: {error}"))?;

    row.map(profile_from_row).transpose()
}

pub(in crate::server) async fn load_profiles(
    pool: &SqlitePool,
) -> Result<Vec<StoredServerProfile>, String> {
    let rows = sqlx::query(
        "SELECT id, server_type, url, secondary_url, username
         FROM server_profiles
         ORDER BY rowid",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| format!("Failed to load server profiles: {error}"))?;

    rows.into_iter().map(profile_from_row).collect()
}

pub(in crate::server) async fn load_first_profile(
    pool: &SqlitePool,
) -> Result<Option<StoredServerProfile>, String> {
    let row = sqlx::query(
        "SELECT id, server_type, url, secondary_url, username
         FROM server_profiles
         ORDER BY rowid
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|error| format!("Failed to load server profile: {error}"))?;

    row.map(profile_from_row).transpose()
}

pub(in crate::server) async fn active_profile_id(
    pool: &SqlitePool,
) -> Result<Option<String>, String> {
    sqlx::query("SELECT value FROM app_state WHERE key = ?")
        .bind(ACTIVE_SERVER_PROFILE_KEY)
        .fetch_optional(pool)
        .await
        .map_err(|error| format!("Failed to load active server profile: {error}"))
        .map(|row| row.map(|row| row.get("value")))
}

pub(in crate::server) async fn save_profile(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    profile: &StoredServerProfile,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO server_profiles (id, server_type, url, secondary_url, username)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             server_type = excluded.server_type,
             url = excluded.url,
             secondary_url = excluded.secondary_url,
             username = excluded.username",
    )
    .bind(&profile.id)
    .bind(profile.server_type.as_storage_value())
    .bind(&profile.url)
    .bind(&profile.secondary_url)
    .bind(&profile.username)
    .execute(&mut **transaction)
    .await
    .map_err(|error| format!("Failed to save server profile: {error}"))?;
    Ok(())
}

pub(in crate::server) async fn delete_profile_data(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    profile_id: &str,
) -> Result<(), String> {
    for (table, statement) in PROFILE_DATA_DELETE_STATEMENTS {
        sqlx::query(*statement)
            .bind(profile_id)
            .execute(&mut **transaction)
            .await
            .map_err(|error| format!("Failed to delete profile data from {table}: {error}"))?;
    }
    Ok(())
}

pub(in crate::server) async fn delete_profile(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    profile_id: &str,
) -> Result<(), String> {
    sqlx::query("DELETE FROM server_profiles WHERE id = ?")
        .bind(profile_id)
        .execute(&mut **transaction)
        .await
        .map_err(|error| format!("Failed to delete server profile: {error}"))?;
    Ok(())
}

pub(in crate::server) async fn next_profile_id(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<Option<String>, String> {
    sqlx::query("SELECT id FROM server_profiles ORDER BY rowid LIMIT 1")
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|error| format!("Failed to choose next server profile: {error}"))
        .map(|row| row.map(|row| row.get("id")))
}

pub(in crate::server) async fn save_active_profile_id(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    profile_id: &str,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO app_state (key, value)
         VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(ACTIVE_SERVER_PROFILE_KEY)
    .bind(profile_id)
    .execute(&mut **transaction)
    .await
    .map_err(|error| format!("Failed to save active server profile: {error}"))?;
    Ok(())
}

pub(in crate::server) async fn clear_active_profile_id(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM app_state WHERE key = ?")
        .bind(ACTIVE_SERVER_PROFILE_KEY)
        .execute(&mut **transaction)
        .await
        .map_err(|error| format!("Failed to clear active server profile: {error}"))?;
    Ok(())
}

fn profile_from_row(row: sqlx::sqlite::SqliteRow) -> Result<StoredServerProfile, String> {
    let server_type = ServerType::from_storage_value(row.get("server_type"))?;
    Ok(StoredServerProfile {
        id: row.get("id"),
        server_type,
        url: row.get("url"),
        secondary_url: row.get("secondary_url"),
        username: row.get("username"),
    })
}

#[cfg(test)]
pub(in crate::server) fn profile_data_tables() -> impl Iterator<Item = &'static str> {
    PROFILE_DATA_DELETE_STATEMENTS
        .iter()
        .map(|(table, _)| *table)
}

#[cfg(test)]
pub(in crate::server) async fn seed_profile_data(
    pool: &SqlitePool,
    profile_id: &str,
    suffix: &str,
) {
    let generation = format!("generation-{suffix}");
    let artist_id = format!("artist-{suffix}");
    let album_id = format!("album-{suffix}");
    let song_id = format!("song-{suffix}");
    let artist_name = format!("Artist {suffix}");
    let album_name = format!("Album {suffix}");
    let song_title = format!("Song {suffix}");
    let genre = format!("Genre {suffix}");
    let statements = [
        format!(
            "INSERT INTO library_sync_state (profile_id, active_generation) VALUES ('{profile_id}', '{generation}')"
        ),
        format!(
            "INSERT INTO artists (profile_id, generation, remote_id, name) VALUES ('{profile_id}', '{generation}', '{artist_id}', '{artist_name}')"
        ),
        format!(
            "INSERT INTO albums (profile_id, generation, remote_id, name, artist_id, artist_name) VALUES ('{profile_id}', '{generation}', '{album_id}', '{album_name}', '{artist_id}', '{artist_name}')"
        ),
        format!(
            "INSERT INTO songs (profile_id, generation, remote_id, album_id, artist_id, title, artist_name, album_name) VALUES ('{profile_id}', '{generation}', '{song_id}', '{album_id}', '{artist_id}', '{song_title}', '{artist_name}', '{album_name}')"
        ),
        format!(
            "INSERT INTO genres (profile_id, generation, name) VALUES ('{profile_id}', '{generation}', '{genre}')"
        ),
        format!(
            "INSERT INTO album_genres (profile_id, generation, album_id, genre) VALUES ('{profile_id}', '{generation}', '{album_id}', '{genre}')"
        ),
        format!(
            "INSERT INTO song_genres (profile_id, generation, song_id, genre) VALUES ('{profile_id}', '{generation}', '{song_id}', '{genre}')"
        ),
        format!(
            "INSERT INTO artwork_cache (profile_id, kind, remote_id) VALUES ('{profile_id}', 'album', '{album_id}')"
        ),
        format!(
            "INSERT INTO album_search (profile_id, generation, remote_id, name, artist_name, genres) VALUES ('{profile_id}', '{generation}', '{album_id}', '{album_name}', '{artist_name}', '{genre}')"
        ),
        format!(
            "INSERT INTO song_search (profile_id, generation, remote_id, album_id, title, artist_name, album_name, genres) VALUES ('{profile_id}', '{generation}', '{song_id}', '{album_id}', '{song_title}', '{artist_name}', '{album_name}', '{genre}')"
        ),
        format!(
            "INSERT INTO artist_search (profile_id, generation, remote_id, name) VALUES ('{profile_id}', '{generation}', '{artist_id}', '{artist_name}')"
        ),
        format!(
            "INSERT INTO pending_scrobbles (profile_id, song_id, started_at_ms, next_attempt_at_ms) VALUES ('{profile_id}', '{song_id}', 1, 1)"
        ),
        format!(
            "INSERT INTO playback_sessions (profile_id, queue_json, active_index, position_seconds) VALUES ('{profile_id}', '[]', 0, 0)"
        ),
        format!(
            "INSERT INTO preferences (profile_id, key, value) VALUES ('{profile_id}', 'volume', '50')"
        ),
    ];

    for statement in statements {
        sqlx::query(sqlx::AssertSqlSafe(statement))
            .execute(pool)
            .await
            .unwrap();
    }
}

#[cfg(test)]
pub(in crate::server) async fn profile_row_count(
    pool: &SqlitePool,
    table: &str,
    profile_id: &str,
) -> i64 {
    let statement = format!("SELECT COUNT(*) FROM {table} WHERE profile_id = ?");
    sqlx::query_scalar(sqlx::AssertSqlSafe(statement))
        .bind(profile_id)
        .fetch_one(pool)
        .await
        .unwrap()
}
