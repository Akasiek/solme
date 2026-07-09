pub(crate) enum Event {
    PlayerStatusChanged,
}

impl Event {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::PlayerStatusChanged => "player-status-changed",
        }
    }
}
