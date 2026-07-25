use std::{
    collections::HashSet,
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
};

use tauri::Manager;

use crate::{
    audio::{MpvBackend, PlaybackSessionService, PlayerService, ScrobbleService},
    credentials::SystemCredentialStore,
    database::{SqliteRepository, DATABASE_FILE_NAME},
    events::{EventBus, EventEmitter},
    library::{LibraryCatalogService, LibrarySyncService},
    server::{MusicServerService, SavedServerEndpoint},
};

type SetupResult<T> = Result<T, Box<dyn Error>>;

struct AppDirs {
    data: PathBuf,
    cache: PathBuf,
}

pub fn setup_app(app: &mut tauri::App) -> SetupResult<()> {
    let dirs = resolve_app_dirs(app)?;
    let database_path = dirs.data.join(DATABASE_FILE_NAME);
    let repository = create_repository(&database_path)?;
    let server = create_server(&repository)?;
    let event_bus = create_event_bus(app);
    let library_catalog = create_library_catalog(&server, &repository);
    let library_sync = create_library_sync(&dirs.cache, &server, &repository);
    let player = create_player(&server, &repository, event_bus)?;
    let scrobble_service = create_scrobble_service(&player, &server, &repository);
    let session_service = create_session_service(&player, &server, &repository);

    app.manage(Arc::clone(&server));
    app.manage(library_catalog);
    app.manage(Arc::clone(&library_sync));
    app.manage(Arc::clone(&player));
    app.manage(Arc::clone(&scrobble_service));
    app.manage(Arc::clone(&session_service));
    scrobble_service.start();
    library_sync.start_periodic();
    #[cfg(target_os = "linux")]
    crate::audio::start_mpris_service(Arc::clone(&player));
    start_saved_server_connection(server, library_sync, player, session_service);
    Ok(())
}

pub(crate) async fn connect_saved_server(
    profile_id: Option<String>,
    server: &Arc<MusicServerService>,
    library: &Arc<LibrarySyncService>,
    player: &Arc<PlayerService>,
    session: &Arc<PlaybackSessionService>,
) -> Result<crate::server::ServerInfo, String> {
    session.suspend_monitoring();
    let connection = server.connect_saved(profile_id).await;
    finish_saved_server_connection(connection, library, player, session).await
}

pub(crate) async fn connect_saved_server_endpoint(
    profile_id: Option<String>,
    endpoint: SavedServerEndpoint,
    server: &Arc<MusicServerService>,
    library: &Arc<LibrarySyncService>,
    player: &Arc<PlayerService>,
    session: &Arc<PlaybackSessionService>,
) -> Result<crate::server::ServerInfo, String> {
    session.suspend_monitoring();
    let connection = server.connect_saved_endpoint(profile_id, endpoint).await;
    finish_saved_server_connection(connection, library, player, session).await
}

async fn finish_saved_server_connection(
    connection: Result<crate::server::ServerInfo, String>,
    library: &Arc<LibrarySyncService>,
    player: &Arc<PlayerService>,
    session: &Arc<PlaybackSessionService>,
) -> Result<crate::server::ServerInfo, String> {
    let info = match connection {
        Ok(info) => info,
        Err(error) => {
            session.resume_monitoring();
            return Err(error);
        }
    };

    let _ = player.restore_preferences().await;
    let _ = session.restore().await;
    session.resume_monitoring();
    session.start();
    library.start(false)?;

    Ok(info)
}

fn resolve_app_dirs(app: &tauri::App) -> SetupResult<AppDirs> {
    Ok(AppDirs {
        data: app.path().app_data_dir()?,
        cache: app.path().app_cache_dir()?,
    })
}

fn create_server(repository: &Arc<SqliteRepository>) -> SetupResult<Arc<MusicServerService>> {
    Ok(Arc::new(MusicServerService::new(
        repository.pool.clone(),
        Box::new(SystemCredentialStore::new().map_err(std::io::Error::other)?),
    )))
}

fn create_repository(database_path: &Path) -> SetupResult<Arc<SqliteRepository>> {
    let repository = tauri::async_runtime::block_on(SqliteRepository::open(database_path))
        .map_err(std::io::Error::other)?;
    Ok(Arc::new(repository))
}

fn create_event_bus(app: &tauri::App) -> Arc<EventBus> {
    Arc::new(EventBus::new(Arc::new(EventEmitter::new(
        app.handle().clone(),
    ))))
}

fn create_library_sync(
    cache_dir: &Path,
    server: &Arc<MusicServerService>,
    repository: &Arc<SqliteRepository>,
) -> Arc<LibrarySyncService> {
    let repository = Arc::clone(repository);
    Arc::new(LibrarySyncService::new(
        Arc::clone(server),
        repository,
        cache_dir.join("artwork"),
    ))
}

fn create_library_catalog(
    server: &Arc<MusicServerService>,
    repository: &Arc<SqliteRepository>,
) -> Arc<LibraryCatalogService> {
    let repository = Arc::clone(repository);
    Arc::new(LibraryCatalogService::new(Arc::clone(server), repository))
}

fn create_player(
    server: &Arc<MusicServerService>,
    repository: &Arc<SqliteRepository>,
    event_bus: Arc<EventBus>,
) -> SetupResult<Arc<PlayerService>> {
    let audio = MpvBackend::new().map_err(std::io::Error::other)?;
    let library_repository = Arc::clone(repository);
    let preference_repository = Arc::clone(repository);
    Ok(Arc::new(PlayerService::new(
        Box::new(audio),
        Arc::clone(server),
        library_repository,
        preference_repository,
        event_bus,
    )))
}

fn create_scrobble_service(
    player: &Arc<PlayerService>,
    server: &Arc<MusicServerService>,
    repository: &Arc<SqliteRepository>,
) -> Arc<ScrobbleService> {
    let repository = Arc::clone(repository);
    Arc::new(ScrobbleService::new(
        Arc::clone(player),
        Arc::clone(server),
        repository,
    ))
}

fn create_session_service(
    player: &Arc<PlayerService>,
    server: &Arc<MusicServerService>,
    repository: &Arc<SqliteRepository>,
) -> Arc<PlaybackSessionService> {
    let repository = Arc::clone(repository);
    Arc::new(PlaybackSessionService::new(
        Arc::clone(player),
        Arc::clone(server),
        repository,
    ))
}

fn start_saved_server_connection(
    server: Arc<MusicServerService>,
    library_sync: Arc<LibrarySyncService>,
    player: Arc<PlayerService>,
    session: Arc<PlaybackSessionService>,
) {
    tauri::async_runtime::spawn(async move {
        cleanup_orphaned_artwork(&server, &library_sync).await;

        match connect_saved_server(None, &server, &library_sync, &player, &session).await {
            Ok(_) => {}
            Err(error) if error == "No server profile is saved" => {}
            Err(error) => log::error!("Failed to restore saved music server on startup: {error}"),
        }
    });
}

async fn cleanup_orphaned_artwork(
    server: &Arc<MusicServerService>,
    library: &Arc<LibrarySyncService>,
) {
    let profiles = match server.saved_profiles().await {
        Ok(profiles) => profiles,
        Err(error) => {
            log::warn!("Failed to list profiles for artwork cleanup: {error}");
            return;
        }
    };
    let profile_ids = profiles
        .into_iter()
        .map(|profile| profile.id)
        .collect::<HashSet<_>>();

    if let Err(error) = library.remove_orphaned_artwork(profile_ids).await {
        log::warn!("Failed to clean orphaned artwork: {error}");
    }
}
