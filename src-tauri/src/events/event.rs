pub(crate) enum Event {
    PlayerQueueChanged,
    PlayerStatusChanged,
}

impl Event {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::PlayerQueueChanged => "player-queue-changed",
            Self::PlayerStatusChanged => "player-status-changed",
        }
    }
}
