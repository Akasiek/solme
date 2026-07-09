CREATE VIRTUAL TABLE album_search USING fts5(
    profile_id UNINDEXED,
    generation UNINDEXED,
    remote_id UNINDEXED,
    name,
    artist_name,
    genres,
    tokenize = 'unicode61 remove_diacritics 2',
    prefix = '2 3 4'
);

CREATE VIRTUAL TABLE song_search USING fts5(
    profile_id UNINDEXED,
    generation UNINDEXED,
    remote_id UNINDEXED,
    album_id UNINDEXED,
    title,
    artist_name,
    album_name,
    genres,
    tokenize = 'unicode61 remove_diacritics 2',
    prefix = '2 3 4'
);
