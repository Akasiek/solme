CREATE TABLE pending_scrobbles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    profile_id TEXT NOT NULL,
    song_id TEXT NOT NULL,
    started_at_ms INTEGER NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    next_attempt_at_ms INTEGER NOT NULL,
    UNIQUE (profile_id, song_id, started_at_ms)
);

CREATE INDEX pending_scrobbles_due
    ON pending_scrobbles (profile_id, next_attempt_at_ms);
