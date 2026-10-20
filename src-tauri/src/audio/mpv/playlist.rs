use rsmpv::Node;

use crate::audio::backend::{AudioPlaylist, AudioPlaylistEntry};

pub(super) fn parse_playlist(node: Node) -> Result<AudioPlaylist, String> {
    let Node::Array(entries) = node else {
        return Err("MPV playlist is not an array".to_string());
    };
    let entries = entries
        .into_iter()
        .map(parse_entry)
        .collect::<Result<_, _>>()?;
    Ok(AudioPlaylist { entries })
}

fn parse_entry(node: Node) -> Result<AudioPlaylistEntry, String> {
    let Node::Map(fields) = node else {
        return Err("MPV playlist entry is not a map".to_string());
    };
    let mut id = None;
    let mut source = None;
    let mut current = false;
    let mut playing = false;

    for (name, value) in fields {
        match (name.as_str(), value) {
            ("id", Node::Int64(value)) => id = Some(value),
            ("filename", Node::String(value)) => source = Some(value),
            ("current", Node::Flag(value)) => current = value,
            ("playing", Node::Flag(value)) => playing = value,
            _ => {}
        }
    }

    Ok(AudioPlaylistEntry {
        id: id.ok_or_else(|| "MPV playlist entry has no valid ID".to_string())?,
        source: source.ok_or_else(|| "MPV playlist entry has no valid source".to_string())?,
        current,
        playing,
    })
}
