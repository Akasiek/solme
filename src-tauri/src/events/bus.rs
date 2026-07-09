use std::sync::Arc;

use tokio::sync::broadcast;

use crate::audio::PlayerStatus;

use super::EventEmitter;

pub struct EventBus {
    emitter: Arc<EventEmitter>,
    player_status: broadcast::Sender<PlayerStatus>,
}

impl EventBus {
    pub fn new(emitter: Arc<EventEmitter>) -> Self {
        let (player_status, _) = broadcast::channel(32);

        Self {
            emitter,
            player_status,
        }
    }

    #[cfg(test)]
    pub fn disabled() -> Self {
        Self::new(Arc::new(EventEmitter::disabled()))
    }

    pub fn publish_player_status(&self, status: PlayerStatus) -> Result<(), String> {
        let _ = self.player_status.send(status.clone());
        self.emitter.player_status_changed(status)
    }

    pub fn subscribe_player_status(&self) -> broadcast::Receiver<PlayerStatus> {
        self.player_status.subscribe()
    }
}
