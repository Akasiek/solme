use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::audio::PlayerStatus;

use super::event::Event;

pub struct EventEmitter {
    app: Option<AppHandle>,
}

impl EventEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app: Some(app) }
    }

    #[cfg(test)]
    pub fn disabled() -> Self {
        Self { app: None }
    }

    pub fn player_status_changed(&self, status: PlayerStatus) -> Result<(), String> {
        self.emit(Event::PlayerStatusChanged, status)
    }

    fn emit<S: Serialize + Clone>(&self, event: Event, payload: S) -> Result<(), String> {
        let Some(app) = &self.app else {
            return Ok(());
        };
        let event = event.as_str();

        app.emit(event, payload)
            .map_err(|error| format!("Failed to emit {event}: {error}"))
    }
}
