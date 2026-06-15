use super::{PlaybackSession, PlaybackSessionRepository};
use crate::audio::PlayerService;
use crate::server::MusicServerService;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

const SESSION_SAVE_INTERVAL: Duration = Duration::from_secs(10);

pub struct PlaybackSessionService {
    player: Arc<PlayerService>,
    server: Arc<MusicServerService>,
    repository: Arc<dyn PlaybackSessionRepository>,
    monitor_running: AtomicBool,
    monitor_suspended: AtomicBool,
}

impl PlaybackSessionService {
    pub fn new(
        player: Arc<PlayerService>,
        server: Arc<MusicServerService>,
        repository: Arc<dyn PlaybackSessionRepository>,
    ) -> Self {
        Self {
            player,
            server,
            repository,
            monitor_running: AtomicBool::new(false),
            monitor_suspended: AtomicBool::new(false),
        }
    }

    pub async fn restore(&self) -> Result<(), String> {
        let (profile_id, _) = self.server.current_server()?;
        let Some(session) = self.repository.load(&profile_id).await? else {
            return Ok(());
        };

        self.player.restore_session(session)
    }

    pub fn start(self: &Arc<Self>) {
        if self.monitor_running.swap(true, Ordering::SeqCst) {
            return;
        }

        let service = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            service.monitor_player().await;
        });
    }

    pub fn suspend_monitoring(&self) {
        self.monitor_suspended.store(true, Ordering::SeqCst);
    }

    pub fn resume_monitoring(&self) {
        self.monitor_suspended.store(false, Ordering::SeqCst);
    }

    async fn monitor_player(&self) {
        let mut last_saved: Option<(String, Option<PlaybackSession>)> = None;

        loop {
            tokio::time::sleep(SESSION_SAVE_INTERVAL).await;

            if self.monitor_suspended.load(Ordering::SeqCst) {
                continue;
            }

            let Ok((profile_id, _)) = self.server.current_server() else {
                continue;
            };
            let Ok(session) = self.player.session_snapshot() else {
                continue;
            };
            if last_saved
                .as_ref()
                .is_some_and(|saved| saved == &(profile_id.clone(), session.clone()))
            {
                continue;
            }

            if self
                .repository
                .save(&profile_id, session.as_ref())
                .await
                .is_ok()
            {
                last_saved = Some((profile_id, session));
            }
        }
    }
}
