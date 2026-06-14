use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
};

use tauri::Manager;

use crate::{
    audio::{MpvBackend, PlayerService},
    credentials::SystemCredentialStore,
    library::{LibraryRepository, LibrarySyncService, SqliteLibraryRepository},
    server::MusicServerService,
};

type SetupResult<T> = Result<T, Box<dyn Error>>;

struct AppDirs {
    config: PathBuf,
    data: PathBuf,
    cache: PathBuf,
}

pub fn setup_app(app: &mut tauri::App) -> SetupResult<()> {
    let dirs = resolve_app_dirs(app)?;
    let server = create_server(&dirs.config)?;
    let library_repository = create_library_repository(&dirs.data)?;
    let library_sync = create_library_sync(&dirs.cache, &server, &library_repository);
    let player = create_player(&server, library_repository)?;

    app.manage(Arc::clone(&server));
    app.manage(Arc::clone(&library_sync));
    app.manage(player);
    start_saved_server_connection(server, library_sync);
    Ok(())
}

fn resolve_app_dirs(app: &tauri::App) -> SetupResult<AppDirs> {
    Ok(AppDirs {
        config: app.path().app_config_dir()?,
        data: app.path().app_data_dir()?,
        cache: app.path().app_cache_dir()?,
    })
}

fn create_server(config_dir: &Path) -> SetupResult<Arc<MusicServerService>> {
    Ok(Arc::new(MusicServerService::new(
        config_dir.join("server-profile.json"),
        Box::new(SystemCredentialStore::new().map_err(std::io::Error::other)?),
    )))
}

fn create_library_repository(data_dir: &Path) -> SetupResult<Arc<dyn LibraryRepository>> {
    let library_repository = tauri::async_runtime::block_on(SqliteLibraryRepository::open(
        &data_dir.join("library.sqlite"),
    ))
    .map_err(std::io::Error::other)?;
    Ok(Arc::new(library_repository))
}

fn create_library_sync(
    cache_dir: &Path,
    server: &Arc<MusicServerService>,
    library_repository: &Arc<dyn LibraryRepository>,
) -> Arc<LibrarySyncService> {
    Arc::new(LibrarySyncService::new(
        Arc::clone(server),
        Arc::clone(library_repository),
        cache_dir.join("artwork"),
    ))
}

fn create_player(
    server: &Arc<MusicServerService>,
    library_repository: Arc<dyn LibraryRepository>,
) -> SetupResult<Arc<PlayerService>> {
    let audio = MpvBackend::new().map_err(std::io::Error::other)?;
    Ok(Arc::new(PlayerService::new(
        Box::new(audio),
        Arc::clone(server),
        library_repository,
    )))
}

fn start_saved_server_connection(
    server: Arc<MusicServerService>,
    library_sync: Arc<LibrarySyncService>,
) {
    tauri::async_runtime::spawn(async move {
        if server.connect_saved().await.is_ok() {
            let _ = library_sync.start(false);
        }
    });
}
