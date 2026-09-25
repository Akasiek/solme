CREATE INDEX songs_active_title
    ON songs (profile_id, generation, title COLLATE NOCASE, artist_name COLLATE NOCASE, remote_id);
