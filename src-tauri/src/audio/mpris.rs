use std::{path::Path, sync::Arc, time::Duration};
use mpris_server::{
    zbus::{fdo, Result},
    LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface, Property, RootInterface,
    Server, Time, TrackId, Volume,
};
use super::{PlaybackState, PlayerService, PlayerStatus};

const STATUS_INTERVAL: Duration = Duration::from_secs(1);

pub fn start_mpris_service(player: Arc<PlayerService>) {
    tauri::async_runtime::spawn(async move {
        let handler = MprisPlayer::new(player);
        let bus_name = format!("solme.instance{}", std::process::id());
        let Ok(server) = Server::new(&bus_name, handler).await else {
            return;
        };

        monitor_player(server).await;
    });
}

struct MprisPlayer {
    player: Arc<PlayerService>,
}

impl MprisPlayer {
    fn new(player: Arc<PlayerService>) -> Self {
        Self { player }
    }

    fn status(&self) -> fdo::Result<PlayerStatus> {
        self.player.status().map_err(fdo::Error::Failed)
    }
}

impl RootInterface for MprisPlayer {
    async fn raise(&self) -> fdo::Result<()> {
        Ok(())
    }

    async fn quit(&self) -> fdo::Result<()> {
        Ok(())
    }

    async fn can_quit(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn set_fullscreen(&self, _fullscreen: bool) -> Result<()> {
        Ok(())
    }

    async fn can_set_fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn can_raise(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn has_track_list(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn identity(&self) -> fdo::Result<String> {
        Ok("Solme".to_string())
    }

    async fn desktop_entry(&self) -> fdo::Result<String> {
        Ok("solme".to_string())
    }

    async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> {
        Ok(Vec::new())
    }
}

impl PlayerInterface for MprisPlayer {
    async fn next(&self) -> fdo::Result<()> {
        self.player.next().map_err(fdo::Error::Failed)
    }

    async fn previous(&self) -> fdo::Result<()> {
        self.player.previous().map_err(fdo::Error::Failed)
    }

    async fn pause(&self) -> fdo::Result<()> {
        self.player.pause().map_err(fdo::Error::Failed)
    }

    async fn play_pause(&self) -> fdo::Result<()> {
        match self.status()?.state {
            PlaybackState::Playing => self.player.pause(),
            PlaybackState::Paused | PlaybackState::Stopped => self.player.resume(),
        }
        .map_err(fdo::Error::Failed)
    }

    async fn stop(&self) -> fdo::Result<()> {
        self.player.stop().map_err(fdo::Error::Failed)
    }

    async fn play(&self) -> fdo::Result<()> {
        self.player.resume().map_err(fdo::Error::Failed)
    }

    async fn seek(&self, offset: Time) -> fdo::Result<()> {
        let position = self.status()?.position_seconds + offset.as_micros() as f64 / 1_000_000.0;
        self.player
            .seek(position.max(0.0))
            .map_err(fdo::Error::Failed)
    }

    async fn set_position(&self, track_id: TrackId, position: Time) -> fdo::Result<()> {
        let status = self.status()?;
        if current_track_id(&status) != track_id || position.is_negative() {
            return Ok(());
        }

        let position_seconds = position.as_micros() as f64 / 1_000_000.0;
        if position_seconds > status.duration_seconds {
            return Ok(());
        }

        self.player
            .seek(position_seconds)
            .map_err(fdo::Error::Failed)
    }

    async fn open_uri(&self, _uri: String) -> fdo::Result<()> {
        Err(fdo::Error::NotSupported(
            "Opening arbitrary URIs is not supported".to_string(),
        ))
    }

    async fn playback_status(&self) -> fdo::Result<PlaybackStatus> {
        Ok(playback_status(self.status()?.state))
    }

    async fn loop_status(&self) -> fdo::Result<LoopStatus> {
        Ok(LoopStatus::None)
    }

    async fn set_loop_status(&self, _loop_status: LoopStatus) -> Result<()> {
        Err(fdo::Error::NotSupported("Looping is not supported".to_string()).into())
    }

    async fn rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn set_rate(&self, _rate: PlaybackRate) -> Result<()> {
        Err(fdo::Error::NotSupported("Playback rate is not supported".to_string()).into())
    }

    async fn shuffle(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn set_shuffle(&self, _shuffle: bool) -> Result<()> {
        Err(fdo::Error::NotSupported("Shuffle is not supported".to_string()).into())
    }

    async fn metadata(&self) -> fdo::Result<Metadata> {
        Ok(metadata(&self.status()?))
    }

    async fn volume(&self) -> fdo::Result<Volume> {
        Ok(self.status()?.volume / 100.0)
    }

    async fn set_volume(&self, volume: Volume) -> Result<()> {
        self.player
            .set_volume(volume.max(0.0) * 100.0)
            .map_err(|error| fdo::Error::Failed(error).into())
    }

    async fn position(&self) -> fdo::Result<Time> {
        Ok(time_from_seconds(self.status()?.position_seconds))
    }

    async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn can_go_next(&self) -> fdo::Result<bool> {
        Ok(can_go_next(&self.status()?))
    }

    async fn can_go_previous(&self) -> fdo::Result<bool> {
        Ok(can_go_previous(&self.status()?))
    }

    async fn can_play(&self) -> fdo::Result<bool> {
        Ok(self.status()?.current_song.is_some())
    }

    async fn can_pause(&self) -> fdo::Result<bool> {
        Ok(self.status()?.current_song.is_some())
    }

    async fn can_seek(&self) -> fdo::Result<bool> {
        Ok(self.status()?.current_song.is_some())
    }

    async fn can_control(&self) -> fdo::Result<bool> {
        Ok(true)
    }
}

async fn monitor_player(server: Server<MprisPlayer>) {
    let mut previous = None;

    loop {
        if let Ok(status) = server.imp().status() {
            let current = MprisSnapshot::from(&status);
            if let Some(previous) = previous.as_ref() {
                let properties = changed_properties(previous, &current, &status);
                if !properties.is_empty() {
                    let _ = server.properties_changed(properties).await;
                }
            }
            previous = Some(current);
        }

        tokio::time::sleep(STATUS_INTERVAL).await;
    }
}

#[derive(PartialEq)]
struct MprisSnapshot {
    state: PlaybackState,
    track_id: TrackId,
    volume: i64,
    queue_position: Option<usize>,
    queue_length: usize,
}

impl From<&PlayerStatus> for MprisSnapshot {
    fn from(status: &PlayerStatus) -> Self {
        Self {
            state: status.state,
            track_id: current_track_id(status),
            volume: (status.volume * 1000.0).round() as i64,
            queue_position: status.queue_position,
            queue_length: status.queue_length,
        }
    }
}

fn changed_properties(
    previous: &MprisSnapshot,
    current: &MprisSnapshot,
    status: &PlayerStatus,
) -> Vec<Property> {
    let mut properties = Vec::new();

    if previous.state != current.state {
        properties.push(Property::PlaybackStatus(playback_status(current.state)));
    }
    if previous.track_id != current.track_id {
        properties.push(Property::Metadata(metadata(status)));
        properties.push(Property::CanPlay(status.current_song.is_some()));
        properties.push(Property::CanPause(status.current_song.is_some()));
        properties.push(Property::CanSeek(status.current_song.is_some()));
    }
    if previous.volume != current.volume {
        properties.push(Property::Volume(status.volume / 100.0));
    }
    if previous.queue_position != current.queue_position
        || previous.queue_length != current.queue_length
    {
        properties.push(Property::CanGoNext(can_go_next(status)));
        properties.push(Property::CanGoPrevious(can_go_previous(status)));
    }

    properties
}

fn metadata(status: &PlayerStatus) -> Metadata {
    let Some(song) = status.current_song.as_ref() else {
        return Metadata::new();
    };

    let mut metadata = Metadata::new();
    metadata.set_trackid(Some(current_track_id(status)));
    metadata.set_title(Some(song.title.clone()));
    metadata.set_artist(Some([song.artist_name.clone()]));
    metadata.set_album(Some(song.album_name.clone()));
    metadata.set_album_artist(Some([song.artist_name.clone()]));
    metadata.set_track_number(song.track_number.and_then(to_i32));
    metadata.set_disc_number(song.disc_number.and_then(to_i32));
    metadata.set_length(Some(Time::from_secs(song.duration_seconds.max(0))));
    metadata.set_art_url(song.artwork_path.as_deref().and_then(file_uri));
    metadata
}

fn current_track_id(status: &PlayerStatus) -> TrackId {
    let (Some(song), Some(queue_position)) = (status.current_song.as_ref(), status.queue_position)
    else {
        return TrackId::NO_TRACK;
    };
    let digest = md5::compute(song.remote_id.as_bytes());
    TrackId::try_from(format!("/com/solme/track/t{queue_position}_{digest:x}"))
        .unwrap_or(TrackId::NO_TRACK)
}

fn playback_status(state: PlaybackState) -> PlaybackStatus {
    match state {
        PlaybackState::Stopped => PlaybackStatus::Stopped,
        PlaybackState::Playing => PlaybackStatus::Playing,
        PlaybackState::Paused => PlaybackStatus::Paused,
    }
}

fn can_go_next(status: &PlayerStatus) -> bool {
    status
        .queue_position
        .is_some_and(|position| position < status.queue_length)
}

fn can_go_previous(status: &PlayerStatus) -> bool {
    status.queue_position.is_some_and(|position| position > 1)
}

fn time_from_seconds(seconds: f64) -> Time {
    Time::from_micros((seconds.max(0.0) * 1_000_000.0).round() as i64)
}

fn file_uri(path: &str) -> Option<String> {
    reqwest::Url::from_file_path(Path::new(path))
        .ok()
        .map(Into::into)
}

fn to_i32(value: i64) -> Option<i32> {
    i32::try_from(value).ok()
}

#[cfg(test)]
mod tests {
    use super::{can_go_next, can_go_previous, current_track_id, metadata, playback_status};
    use crate::{
        audio::{PlaybackState, PlayerStatus},
        library::CachedSong,
    };
    use mpris_server::{PlaybackStatus, Time, TrackId};

    #[test]
    fn maps_player_status_to_mpris_metadata() {
        let status = status(PlaybackState::Playing, Some(2), 3);

        let metadata = metadata(&status);

        assert_ne!(metadata.trackid(), Some(TrackId::NO_TRACK));
        assert_eq!(metadata.title(), Some("Song"));
        assert_eq!(metadata.album(), Some("Album"));
        assert_eq!(metadata.artist(), Some(vec!["Artist".to_string()]));
        assert_eq!(metadata.length(), Some(Time::from_secs(180)));
        assert_eq!(current_track_id(&status), metadata.trackid().unwrap());
    }

    #[test]
    fn maps_playback_state_and_queue_capabilities() {
        let status = status(PlaybackState::Paused, Some(2), 3);

        assert_eq!(playback_status(status.state), PlaybackStatus::Paused);
        assert!(can_go_next(&status));
        assert!(can_go_previous(&status));
    }

    #[test]
    fn uses_no_track_without_current_song() {
        let status = PlayerStatus {
            state: PlaybackState::Stopped,
            current_song: None,
            position_seconds: 0.0,
            duration_seconds: 0.0,
            queue_position: None,
            queue_length: 0,
            volume: 50.0,
        };

        assert_eq!(current_track_id(&status), TrackId::NO_TRACK);
        assert_eq!(metadata(&status).trackid(), None);
    }

    fn status(
        state: PlaybackState,
        queue_position: Option<usize>,
        queue_length: usize,
    ) -> PlayerStatus {
        PlayerStatus {
            state,
            current_song: Some(CachedSong {
                remote_id: "song-1".to_string(),
                album_id: "album-1".to_string(),
                title: "Song".to_string(),
                artist_name: "Artist".to_string(),
                album_name: "Album".to_string(),
                artwork_path: None,
                track_number: Some(2),
                disc_number: Some(1),
                duration_seconds: 180,
            }),
            position_seconds: 30.0,
            duration_seconds: 180.0,
            queue_position,
            queue_length,
            volume: 50.0,
        }
    }
}
