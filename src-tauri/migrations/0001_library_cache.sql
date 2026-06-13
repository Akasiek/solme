PRAGMA foreign_keys = ON;

CREATE TABLE library_sync_state (
    profile_id TEXT PRIMARY KEY NOT NULL,
    active_generation TEXT,
    server_revision TEXT,
    last_success_at INTEGER,
    artist_count INTEGER NOT NULL DEFAULT 0,
    album_count INTEGER NOT NULL DEFAULT 0,
    song_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE artists (
    profile_id TEXT NOT NULL,
    generation TEXT NOT NULL,
    remote_id TEXT NOT NULL,
    name TEXT NOT NULL,
    album_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (profile_id, generation, remote_id)
);

CREATE TABLE albums (
    profile_id TEXT NOT NULL,
    generation TEXT NOT NULL,
    remote_id TEXT NOT NULL,
    name TEXT NOT NULL,
    artist_id TEXT,
    artist_name TEXT NOT NULL,
    year INTEGER,
    song_count INTEGER NOT NULL DEFAULT 0,
    duration_seconds INTEGER NOT NULL DEFAULT 0,
    cover_art_id TEXT,
    PRIMARY KEY (profile_id, generation, remote_id)
);

CREATE INDEX albums_active_order
    ON albums (profile_id, generation, artist_name, year, name);

CREATE TABLE songs (
    profile_id TEXT NOT NULL,
    generation TEXT NOT NULL,
    remote_id TEXT NOT NULL,
    album_id TEXT NOT NULL,
    artist_id TEXT,
    title TEXT NOT NULL,
    artist_name TEXT NOT NULL,
    album_name TEXT NOT NULL,
    track_number INTEGER,
    disc_number INTEGER,
    year INTEGER,
    duration_seconds INTEGER NOT NULL DEFAULT 0,
    suffix TEXT,
    content_type TEXT,
    cover_art_id TEXT,
    PRIMARY KEY (profile_id, generation, remote_id),
    FOREIGN KEY (profile_id, generation, album_id)
        REFERENCES albums (profile_id, generation, remote_id)
        ON DELETE CASCADE
);

CREATE INDEX songs_by_album
    ON songs (profile_id, generation, album_id, disc_number, track_number, title);

CREATE TABLE genres (
    profile_id TEXT NOT NULL,
    generation TEXT NOT NULL,
    name TEXT NOT NULL,
    song_count INTEGER NOT NULL DEFAULT 0,
    album_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (profile_id, generation, name)
);

CREATE TABLE album_genres (
    profile_id TEXT NOT NULL,
    generation TEXT NOT NULL,
    album_id TEXT NOT NULL,
    genre TEXT NOT NULL,
    PRIMARY KEY (profile_id, generation, album_id, genre),
    FOREIGN KEY (profile_id, generation, album_id)
        REFERENCES albums (profile_id, generation, remote_id)
        ON DELETE CASCADE
);

CREATE TABLE song_genres (
    profile_id TEXT NOT NULL,
    generation TEXT NOT NULL,
    song_id TEXT NOT NULL,
    genre TEXT NOT NULL,
    PRIMARY KEY (profile_id, generation, song_id, genre),
    FOREIGN KEY (profile_id, generation, song_id)
        REFERENCES songs (profile_id, generation, remote_id)
        ON DELETE CASCADE
);

CREATE TABLE artwork_cache (
    profile_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    remote_id TEXT NOT NULL,
    source_key TEXT,
    local_path TEXT,
    content_type TEXT,
    etag TEXT,
    last_modified TEXT,
    downloaded_at INTEGER,
    last_accessed_at INTEGER,
    PRIMARY KEY (profile_id, kind, remote_id)
);
