use std::sync::Arc;

use crate::{
    audio::{PlaybackSessionService, PlayerService},
    library::LibrarySyncService,
    server::{MusicServerService, ServerInfo},
};

pub async fn connect_saved_server(
    server: &Arc<MusicServerService>,
    library: &Arc<LibrarySyncService>,
    player: &Arc<PlayerService>,
    session: &Arc<PlaybackSessionService>,
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
    library.start(false)?;

    Ok(info)
}
