use crate::{
    database::SqliteRepository,
    library::models::{LibraryItemAnnotation, LibraryItemKind},
};

macro_rules! annotation_query {
    ($table:literal) => {
        concat!(
            "SELECT item.favorite, item.rating FROM ",
            $table,
            " item JOIN library_sync_state state ON state.profile_id = item.profile_id ",
            "AND state.active_generation = item.generation ",
            "WHERE item.profile_id = ? AND item.remote_id = ?"
        )
    };
}

macro_rules! update_query {
    ($table:literal, $column:literal) => {
        concat!(
            "UPDATE ",
            $table,
            " SET ",
            $column,
            " = ? WHERE profile_id = ? AND remote_id = ? AND generation = ",
            "(SELECT active_generation FROM library_sync_state WHERE profile_id = ?)"
        )
    };
}

pub(crate) async fn annotation(
    repo: &SqliteRepository,
    profile_id: &str,
    item_kind: LibraryItemKind,
    item_id: &str,
) -> Result<Option<LibraryItemAnnotation>, String> {
    sqlx::query_as::<_, LibraryItemAnnotation>(annotation_sql(item_kind))
        .bind(profile_id)
        .bind(item_id)
        .fetch_optional(&repo.pool)
        .await
        .map_err(|error| {
            format!(
                "Failed to read cached {} annotation: {error}",
                item_kind.label()
            )
        })
}

pub(crate) async fn set_favorite(
    repo: &SqliteRepository,
    profile_id: &str,
    item_kind: LibraryItemKind,
    item_id: &str,
    favorite: bool,
) -> Result<(), String> {
    let result = sqlx::query(set_favorite_sql(item_kind))
        .bind(favorite)
        .bind(profile_id)
        .bind(item_id)
        .bind(profile_id)
        .execute(&repo.pool)
        .await
        .map_err(|error| {
            format!(
                "Failed to update cached {} annotation: {error}",
                item_kind.label()
            )
        })?;
    ensure_updated(result.rows_affected(), item_kind, item_id)
}

pub(crate) async fn set_rating(
    repo: &SqliteRepository,
    profile_id: &str,
    item_kind: LibraryItemKind,
    item_id: &str,
    rating: Option<i64>,
) -> Result<(), String> {
    let result = sqlx::query(set_rating_sql(item_kind))
        .bind(rating)
        .bind(profile_id)
        .bind(item_id)
        .bind(profile_id)
        .execute(&repo.pool)
        .await
        .map_err(|error| {
            format!(
                "Failed to update cached {} annotation: {error}",
                item_kind.label()
            )
        })?;
    ensure_updated(result.rows_affected(), item_kind, item_id)
}

fn ensure_updated(
    rows_affected: u64,
    item_kind: LibraryItemKind,
    item_id: &str,
) -> Result<(), String> {
    if rows_affected != 1 {
        return Err(format!(
            "Cached {} with ID {item_id} was not found",
            item_kind.label()
        ));
    }
    Ok(())
}

fn annotation_sql(item_kind: LibraryItemKind) -> &'static str {
    match item_kind {
        LibraryItemKind::Artist => ANNOTATION_ARTIST_SQL,
        LibraryItemKind::Album => ANNOTATION_ALBUM_SQL,
        LibraryItemKind::Song => ANNOTATION_SONG_SQL,
    }
}

fn set_favorite_sql(item_kind: LibraryItemKind) -> &'static str {
    match item_kind {
        LibraryItemKind::Artist => SET_ARTIST_FAVORITE_SQL,
        LibraryItemKind::Album => SET_ALBUM_FAVORITE_SQL,
        LibraryItemKind::Song => SET_SONG_FAVORITE_SQL,
    }
}

fn set_rating_sql(item_kind: LibraryItemKind) -> &'static str {
    match item_kind {
        LibraryItemKind::Artist => SET_ARTIST_RATING_SQL,
        LibraryItemKind::Album => SET_ALBUM_RATING_SQL,
        LibraryItemKind::Song => SET_SONG_RATING_SQL,
    }
}

const ANNOTATION_ARTIST_SQL: &str = annotation_query!("artists");
const ANNOTATION_ALBUM_SQL: &str = annotation_query!("albums");
const ANNOTATION_SONG_SQL: &str = annotation_query!("songs");
const SET_ARTIST_FAVORITE_SQL: &str = update_query!("artists", "favorite");
const SET_ALBUM_FAVORITE_SQL: &str = update_query!("albums", "favorite");
const SET_SONG_FAVORITE_SQL: &str = update_query!("songs", "favorite");
const SET_ARTIST_RATING_SQL: &str = update_query!("artists", "rating");
const SET_ALBUM_RATING_SQL: &str = update_query!("albums", "rating");
const SET_SONG_RATING_SQL: &str = update_query!("songs", "rating");
