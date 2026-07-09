CREATE TABLE server_profiles (
    id TEXT PRIMARY KEY NOT NULL,
    server_type TEXT NOT NULL,
    url TEXT NOT NULL,
    username TEXT NOT NULL,
    UNIQUE(server_type, url, username)
);

CREATE TABLE app_state (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
