use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
};

use tauri::Manager;

use crate::{
    audio::{
        MpvBackend, PlaybackSessionRepository, PlaybackSessionService, PlayerService,
        ScrobbleRepository, ScrobbleService,
    },
    credentials::SystemCredentialStore,
    database::{SqliteRepository, DATABASE_FILE_NAME},
    library::{LibraryRepository, LibrarySyncService},
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
    let database_path = dirs.data.join(DATABASE_FILE_NAME);
    let repository = create_repository(&database_path)?;
    let library_repository: Arc<dyn LibraryRepository> = repository.clone();
    let scrobble_repository: Arc<dyn ScrobbleRepository> = repository.clone();
    let session_repository: Arc<dyn PlaybackSessionRepository> = repository;
    let library_sync = create_library_sync(&dirs.cache, &server, &library_repository);
    let player = create_player(&server, library_repository)?;
    let scrobble_service = create_scrobble_service(&player, &server, scrobble_repository);
    let session_service = create_session_service(&player, &server, session_repository);

    app.manage(Arc::clone(&server));
    app.manage(Arc::clone(&library_sync));
    app.manage(Arc::clone(&player));
    app.manage(Arc::clone(&scrobble_service));
    app.manage(Arc::clone(&session_service));
    scrobble_service.start();
    #[cfg(target_os = "linux")]
    crate::audio::start_mpris_service(Arc::clone(&player));
    start_saved_server_connection(server, library_sync, session_service);
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

fn create_repository(database_path: &Path) -> SetupResult<Arc<SqliteRepository>> {
    let repository = tauri::async_runtime::block_on(SqliteRepository::open(database_path))
        .map_err(std::io::Error::other)?;
    Ok(Arc::new(repository))
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

fn create_scrobble_service(
    player: &Arc<PlayerService>,
    server: &Arc<MusicServerService>,
    repository: Arc<dyn ScrobbleRepository>,
) -> Arc<ScrobbleService> {
    Arc::new(ScrobbleService::new(
        Arc::clone(player),
        Arc::clone(server),
        repository,
    ))
}

fn create_session_service(
    player: &Arc<PlayerService>,
    server: &Arc<MusicServerService>,
    repository: Arc<dyn PlaybackSessionRepository>,
) -> Arc<PlaybackSessionService> {
    Arc::new(PlaybackSessionService::new(
        Arc::clone(player),
        Arc::clone(server),
        repository,
    ))
}

fn start_saved_server_connection(
    server: Arc<MusicServerService>,
    library_sync: Arc<LibrarySyncService>,
    session: Arc<PlaybackSessionService>,
) {
    tauri::async_runtime::spawn(async move {
        if server.connect_saved().await.is_ok() {
            let _ = session.restore().await;
            session.start();
            let _ = library_sync.start(false);
        }
    });
}
