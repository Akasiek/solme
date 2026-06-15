CREATE TABLE playback_sessions (
    profile_id TEXT PRIMARY KEY NOT NULL,
    queue_json TEXT NOT NULL,
    active_index INTEGER NOT NULL,
    position_seconds REAL NOT NULL
);
