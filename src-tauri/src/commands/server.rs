use std::sync::Arc;

use tauri::State;

use crate::server::{MusicServerService, SavedServerProfile, ServerConnectionConfig, ServerInfo};
use crate::{
    audio::{PlaybackSessionService, PlayerService},
    library::LibrarySyncService,
};

#[tauri::command]
pub async fn connect_music_server(
    config: ServerConnectionConfig,
    server: State<'_, Arc<MusicServerService>>,
    library: State<'_, Arc<LibrarySyncService>>,
    player: State<'_, Arc<PlayerService>>,
    session: State<'_, Arc<PlaybackSessionService>>,
) -> Result<ServerInfo, String> {
    session.suspend_monitoring();
    let connection = server.connect(config).await;
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
    let _ = library.start(false);
    Ok(info)
}

#[tauri::command]
pub async fn ping_music_server(
    server: State<'_, Arc<MusicServerService>>,
) -> Result<ServerInfo, String> {
    server.ping().await
}

#[tauri::command]
pub async fn get_saved_server_profile(
    server: State<'_, Arc<MusicServerService>>,
) -> Result<Option<SavedServerProfile>, String> {
    server.saved_profile().await
}

#[tauri::command]
pub async fn connect_saved_music_server(
    server: State<'_, Arc<MusicServerService>>,
    library: State<'_, Arc<LibrarySyncService>>,
    player: State<'_, Arc<PlayerService>>,
    session: State<'_, Arc<PlaybackSessionService>>,
) -> Result<ServerInfo, String> {
    session.suspend_monitoring();
    let connection = server.connect_saved().await;
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
    let _ = library.start(false);
    Ok(info)
}

#[tauri::command]
pub async fn forget_saved_server_profile(
    server: State<'_, Arc<MusicServerService>>,
) -> Result<(), String> {
    server.forget_saved_profile().await
}
