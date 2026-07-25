use std::{
    collections::HashSet,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::server::backend::MusicServer;

use super::{
    models::{ArtworkCacheRecord, ArtworkCandidate, BinaryArtwork},
    repository::LibrarySyncRepository,
    time::now_epoch_seconds,
};

pub async fn synchronize_artwork_item(
    profile_id: &str,
    candidate: ArtworkCandidate,
    server: Arc<dyn MusicServer>,
    repository: Arc<dyn LibrarySyncRepository>,
    artwork_root: &Path,
    fresh_after: i64,
) -> Result<(), String> {
    let expected_source = (candidate.kind == "album").then_some(candidate.source_id.as_str());
    if repository
        .artwork_is_fresh(
            profile_id,
            candidate.kind,
            &candidate.remote_id,
            expected_source,
            fresh_after,
        )
        .await?
    {
        return Ok(());
    }

    let artwork = match candidate.kind {
        "album" => server.album_artwork(&candidate.source_id).await?,
        "artist" => server.artist_artwork(&candidate.source_id).await?,
        kind => return Err(format!("Unsupported artwork kind: {kind}")),
    };
    let Some(artwork) = artwork else {
        return Ok(());
    };

    let downloaded_at = now_epoch_seconds()?;
    let path = write_artwork(
        artwork_root,
        profile_id,
        candidate.kind,
        &candidate.remote_id,
        &artwork,
    )
    .await?;
    repository
        .save_artwork(
            profile_id,
            ArtworkCacheRecord {
                kind: candidate.kind,
                remote_id: candidate.remote_id,
                source_key: artwork.source_key,
                local_path: path.to_string_lossy().into_owned(),
                content_type: artwork.content_type,
                etag: artwork.etag,
                last_modified: artwork.last_modified,
                downloaded_at,
            },
        )
        .await
}

async fn write_artwork(
    root: &Path,
    profile_id: &str,
    kind: &str,
    remote_id: &str,
    artwork: &BinaryArtwork,
) -> Result<PathBuf, String> {
    let directory = root.join(profile_id).join(kind);
    let extension = content_type_extension(&artwork.content_type);
    let filename = format!("{:x}.{extension}", md5::compute(remote_id.as_bytes()));
    let path = directory.join(filename);
    let temporary_path = path.with_extension(format!("{extension}.tmp"));
    let bytes = artwork.bytes.clone();
    let final_path = path.clone();

    tauri::async_runtime::spawn_blocking(move || {
        std::fs::create_dir_all(&directory)
            .map_err(|error| format!("Failed to create artwork directory: {error}"))?;
        std::fs::write(&temporary_path, bytes)
            .map_err(|error| format!("Failed to write artwork: {error}"))?;
        std::fs::rename(&temporary_path, &final_path)
            .map_err(|error| format!("Failed to activate artwork file: {error}"))
    })
    .await
    .map_err(|error| format!("Artwork writer task failed: {error}"))??;

    Ok(path)
}

fn content_type_extension(content_type: &str) -> &'static str {
    match content_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/avif" => "avif",
        _ => "img",
    }
}

pub(super) fn remove_profile_artwork(root: &Path, profile_id: &str) -> Result<(), String> {
    remove_artwork_directory(&root.join(profile_id))
}

fn remove_artwork_directory(directory: &Path) -> Result<(), String> {
    match std::fs::remove_dir_all(directory) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "Failed to remove artwork directory {}: {error}",
            directory.display()
        )),
    }
}

pub(super) fn remove_orphaned_artwork(
    root: &Path,
    valid_profile_ids: &HashSet<String>,
) -> Result<(), String> {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "Failed to read artwork cache directory {}: {error}",
                root.display()
            ))
        }
    };

    for entry in entries {
        let entry =
            entry.map_err(|error| format!("Failed to read artwork cache entry: {error}"))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("Failed to inspect artwork cache entry: {error}"))?;
        if !file_type.is_dir() {
            continue;
        }

        if !valid_profile_ids.contains(entry.file_name().to_string_lossy().as_ref()) {
            remove_artwork_directory(&entry.path())?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use uuid::Uuid;

    use super::{remove_orphaned_artwork, remove_profile_artwork};

    #[test]
    fn removes_only_orphaned_profile_directories() {
        let root = std::env::temp_dir().join(format!("solme-artwork-cleanup-{}", Uuid::new_v4()));
        let active_profile_id = "active-profile";
        let orphaned_profile_id = "orphaned-profile";
        std::fs::create_dir_all(root.join(active_profile_id).join("album")).unwrap();
        std::fs::create_dir_all(root.join(orphaned_profile_id).join("artist")).unwrap();
        std::fs::write(root.join("cache-marker"), []).unwrap();

        remove_orphaned_artwork(&root, &HashSet::from([active_profile_id.to_string()])).unwrap();

        assert!(root.join(active_profile_id).is_dir());
        assert!(!root.join(orphaned_profile_id).exists());
        assert!(root.join("cache-marker").is_file());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn removing_profile_artwork_is_idempotent() {
        let root = std::env::temp_dir().join(format!("solme-artwork-remove-{}", Uuid::new_v4()));
        std::fs::create_dir_all(root.join("profile").join("album")).unwrap();

        remove_profile_artwork(&root, "profile").unwrap();
        remove_profile_artwork(&root, "profile").unwrap();

        assert!(!root.join("profile").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
