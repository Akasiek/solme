CREATE VIRTUAL TABLE artist_search USING fts5(
    profile_id UNINDEXED,
    generation UNINDEXED,
    remote_id UNINDEXED,
    name,
    tokenize = 'unicode61 remove_diacritics 2',
    prefix = '2 3 4'
);

INSERT INTO artist_search (profile_id, generation, remote_id, name)
SELECT profile_id, generation, remote_id, name
FROM artists;
