mod albums;
mod artists;
mod library;
mod search;
mod songs;

pub(crate) use albums::{insert_album_genres, insert_album_search, insert_albums, search_albums};
pub(crate) use artists::{insert_artists, insert_genres};
pub(crate) use library::delete_stale_generations;
pub(crate) use search::search_query;
pub(crate) use songs::{insert_song_genres, insert_song_search, insert_songs, search_songs};
