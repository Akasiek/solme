use std::{cmp::Ordering, collections::HashSet};

use super::models::{CachedAlbum, CachedSong};

const FUZZY_MIN_RESULTS: usize = 5;

pub fn should_use_fuzzy(query: &str, result_count: usize, limit: i64) -> Option<String> {
    let query = normalized_search_text(query);

    if query.chars().count() < 3 || result_count >= FUZZY_MIN_RESULTS.min(limit as usize) {
        return None;
    }

    Some(query)
}

pub fn rank_albums(query: &str, candidates: Vec<CachedAlbum>, limit: i64) -> Vec<CachedAlbum> {
    rank_candidates(query, candidates, limit, album_score)
}

pub fn rank_songs(query: &str, candidates: Vec<CachedSong>, limit: i64) -> Vec<CachedSong> {
    rank_candidates(query, candidates, limit, song_score)
}

pub fn merge_albums(
    mut results: Vec<CachedAlbum>,
    fuzzy_results: Vec<CachedAlbum>,
    limit: i64,
) -> Vec<CachedAlbum> {
    let mut seen = results
        .iter()
        .map(|album| album.remote_id.clone())
        .collect::<HashSet<_>>();

    results.extend(
        fuzzy_results
            .into_iter()
            .filter_map(|album| seen.insert(album.remote_id.clone()).then_some(album)),
    );
    results.truncate(limit as usize);
    results
}

pub fn merge_songs(
    mut results: Vec<CachedSong>,
    fuzzy_results: Vec<CachedSong>,
    limit: i64,
) -> Vec<CachedSong> {
    let mut seen = results
        .iter()
        .map(|song| song.remote_id.clone())
        .collect::<HashSet<_>>();

    results.extend(
        fuzzy_results
            .into_iter()
            .filter_map(|song| seen.insert(song.remote_id.clone()).then_some(song)),
    );
    results.truncate(limit as usize);
    results
}

fn rank_candidates<T>(
    query: &str,
    candidates: Vec<T>,
    limit: i64,
    score: fn(&str, &T) -> f64,
) -> Vec<T> {
    let threshold = threshold(query);
    let mut ranked = candidates
        .into_iter()
        .filter_map(|candidate| {
            let score = score(query, &candidate);
            (score >= threshold).then_some((candidate, score))
        })
        .collect::<Vec<_>>();

    ranked.sort_by(|(_, left_score), (_, right_score)| {
        right_score
            .partial_cmp(left_score)
            .unwrap_or(Ordering::Equal)
    });
    ranked.truncate(limit as usize);
    ranked
        .into_iter()
        .map(|(candidate, _)| candidate)
        .collect::<Vec<_>>()
}

fn normalized_search_text(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|character| match character {
            'ą' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
            'ć' | 'ç' => 'c',
            'ę' | 'è' | 'é' | 'ê' | 'ë' => 'e',
            'ł' => 'l',
            'ń' | 'ñ' => 'n',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ś' => 's',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ź' | 'ż' => 'z',
            character => character,
        })
        .filter(|character| character.is_alphanumeric() || character.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn threshold(query: &str) -> f64 {
    match query.chars().count() {
        0..=2 => 1.0,
        3..=4 => 0.9,
        5..=7 => 0.82,
        _ => 0.78,
    }
}

fn album_score(query: &str, album: &CachedAlbum) -> f64 {
    text_similarity(query, &album.name).max(text_similarity(query, &album.artist_name) * 1.05)
}

fn song_score(query: &str, song: &CachedSong) -> f64 {
    text_similarity(query, &song.title)
        .max(text_similarity(query, &song.artist_name) * 0.95)
        .max(text_similarity(query, &song.album_name) * 0.85)
}

fn text_similarity(query: &str, text: &str) -> f64 {
    let text = normalized_search_text(text);

    if text.is_empty() {
        return 0.0;
    }

    let mut score = jaro_winkler(query, &text);
    for token in text.split_whitespace() {
        score = score.max(jaro_winkler(query, token));
    }

    score
}

/// Returns the Jaro-Winkler similarity for two normalized search strings.
///
/// The score is in the `0.0..=1.0` range, where `1.0` means an exact match.
/// Jaro-Winkler favors strings that share an early prefix, which makes it a
/// useful fallback for short music-search typos like `nibana` vs `nirvana`.
fn jaro_winkler(left: &str, right: &str) -> f64 {
    if left == right {
        return 1.0;
    }

    let left = left.chars().collect::<Vec<_>>();
    let right = right.chars().collect::<Vec<_>>();
    let left_len = left.len();
    let right_len = right.len();

    if left_len == 0 || right_len == 0 {
        return 0.0;
    }

    let match_distance = left_len.max(right_len).saturating_div(2).saturating_sub(1);
    let mut left_matches = vec![false; left_len];
    let mut right_matches = vec![false; right_len];
    let mut matches = 0;

    for left_index in 0..left_len {
        let start = left_index.saturating_sub(match_distance);
        let end = (left_index + match_distance + 1).min(right_len);

        for right_index in start..end {
            if right_matches[right_index] || left[left_index] != right[right_index] {
                continue;
            }

            left_matches[left_index] = true;
            right_matches[right_index] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut transpositions = 0;
    let mut right_index = 0;

    for left_index in 0..left_len {
        if !left_matches[left_index] {
            continue;
        }

        while !right_matches[right_index] {
            right_index += 1;
        }

        if left[left_index] != right[right_index] {
            transpositions += 1;
        }

        right_index += 1;
    }

    let matches = matches as f64;
    let jaro = ((matches / left_len as f64)
        + (matches / right_len as f64)
        + ((matches - (transpositions as f64 / 2.0)) / matches))
        / 3.0;
    let prefix_len = left
        .iter()
        .zip(right.iter())
        .take_while(|(left, right)| left == right)
        .take(4)
        .count() as f64;

    jaro + (prefix_len * 0.1 * (1.0 - jaro))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enables_fuzzy_only_for_long_enough_queries_with_few_results() {
        assert_eq!(should_use_fuzzy("ni", 0, 20), None);
        assert_eq!(should_use_fuzzy("nibana", 5, 20), None);
        assert_eq!(should_use_fuzzy("nibana", 0, 20).as_deref(), Some("nibana"));
    }

    #[test]
    fn normalizes_case_spacing_and_common_diacritics() {
        assert_eq!(
            should_use_fuzzy("  BJÖRK   Guðmundsdóttir ", 0, 20).as_deref(),
            Some("bjork guðmundsdottir")
        );
        assert_eq!(
            should_use_fuzzy("Zażółć Gęślą Jaźń", 0, 20).as_deref(),
            Some("zazolc gesla jazn")
        );
    }

    #[test]
    fn ranks_album_typos_against_artist_and_album_names() {
        let albums = vec![
            album("album-1", "Nevermind", "Nirvana"),
            album("album-2", "Blue Train", "John Coltrane"),
        ];

        let by_artist = rank_albums("nibana", albums.clone(), 20);
        let by_album = rank_albums("nevermimd", albums, 20);

        assert_eq!(by_artist.len(), 1);
        assert_eq!(by_artist[0].remote_id, "album-1");
        assert_eq!(by_album.len(), 1);
        assert_eq!(by_album[0].remote_id, "album-1");
    }

    #[test]
    fn ranks_song_typos_against_title_artist_and_album_names() {
        let songs = vec![
            song("song-1", "Smells Like Teen Spirit", "Nirvana", "Nevermind"),
            song("song-2", "So What", "Miles Davis", "Kind of Blue"),
        ];

        let by_title = rank_songs("smels", songs.clone(), 20);
        let by_artist = rank_songs("nibana", songs, 20);

        assert_eq!(by_title.len(), 1);
        assert_eq!(by_title[0].remote_id, "song-1");
        assert_eq!(by_artist.len(), 1);
        assert_eq!(by_artist[0].remote_id, "song-1");
    }

    #[test]
    fn merge_keeps_fts_results_first_and_deduplicates_fuzzy_results() {
        let merged = merge_albums(
            vec![album("album-1", "Nevermind", "Nirvana")],
            vec![
                album("album-1", "Nevermind", "Nirvana"),
                album("album-2", "In Utero", "Nirvana"),
            ],
            20,
        );

        assert_eq!(
            merged
                .into_iter()
                .map(|album| album.remote_id)
                .collect::<Vec<_>>(),
            ["album-1", "album-2"]
        );
    }

    fn album(remote_id: &str, name: &str, artist_name: &str) -> CachedAlbum {
        CachedAlbum {
            remote_id: remote_id.to_string(),
            name: name.to_string(),
            artist_name: artist_name.to_string(),
            artist_id: Some("artist-1".to_string()),
            year: Some(2026),
            song_count: 1,
            artwork_path: None,
        }
    }

    fn song(remote_id: &str, title: &str, artist_name: &str, album_name: &str) -> CachedSong {
        CachedSong {
            remote_id: remote_id.to_string(),
            album_id: "album-1".to_string(),
            artist_id: Some("artist-1".to_string()),
            title: title.to_string(),
            artist_name: artist_name.to_string(),
            album_name: album_name.to_string(),
            artwork_path: None,
            track_number: Some(1),
            disc_number: Some(1),
            duration_seconds: 180,
        }
    }
}
