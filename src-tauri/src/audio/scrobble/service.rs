use std::{
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crate::{
    audio::{PlaybackState, PlayerService},
    server::{MusicServerService, ScrobbleEvent},
};

use super::{PendingScrobble, ScrobbleRepository};

const MONITOR_INTERVAL: Duration = Duration::from_secs(1);
const MAX_COUNTED_INTERVAL: Duration = Duration::from_secs(2);
const RETRY_INTERVAL: Duration = Duration::from_secs(5);
const MAX_SCROBBLE_THRESHOLD_SECONDS: f64 = 4.0 * 60.0;
const RETRY_BATCH_SIZE: i64 = 20;
const MAX_RETRY_DELAY_SECONDS: i64 = 60 * 60;

pub struct ScrobbleService {
    player: Arc<PlayerService>,
    server: Arc<MusicServerService>,
    repository: Arc<dyn ScrobbleRepository>,
}

impl ScrobbleService {
    pub fn new(
        player: Arc<PlayerService>,
        server: Arc<MusicServerService>,
        repository: Arc<dyn ScrobbleRepository>,
    ) -> Self {
        Self {
            player,
            server,
            repository,
        }
    }

    pub fn start(self: &Arc<Self>) {
        self.start_playback_monitor();
        self.start_retry_worker();
    }

    fn start_playback_monitor(self: &Arc<Self>) {
        let service = Arc::clone(self);
        tauri::async_runtime::spawn(async move { service.run_playback_monitor().await });
    }

    async fn run_playback_monitor(&self) {
        let mut tracker = ScrobbleTracker::default();
        let mut last_sample = Instant::now();

        loop {
            tokio::time::sleep(MONITOR_INTERVAL).await;
            let elapsed = elapsed_since_last_sample(&mut last_sample);
            self.process_playback_sample(&mut tracker, elapsed).await;
        }
    }

    async fn process_playback_sample(&self, tracker: &mut ScrobbleTracker, elapsed: Duration) {
        let Ok(sample) = self.playback_sample() else {
            tracker.reset();
            return;
        };

        for action in tracker.update(&sample, elapsed, now_epoch_millis()) {
            self.handle_scrobble_action(tracker, action).await;
        }
    }

    fn playback_sample(&self) -> Result<PlaybackSample, String> {
        let status = self.player.status()?;
        let profile_id = self
            .server
            .current_server()
            .ok()
            .map(|(profile_id, _)| profile_id);
        let song_id = status
            .current_song
            .as_ref()
            .map(|song| song.remote_id.clone());
        let duration_seconds = status
            .current_song
            .map(|song| song.duration_seconds as f64)
            .unwrap_or(status.duration_seconds);

        Ok(PlaybackSample {
            state: status.state,
            profile_id,
            song_id,
            queue_position: status.queue_position,
            duration_seconds,
        })
    }

    async fn handle_scrobble_action(&self, tracker: &mut ScrobbleTracker, action: ScrobbleAction) {
        match action {
            ScrobbleAction::NowPlaying {
                profile_id,
                song_id,
                started_at_ms,
            } => self.send_now_playing(profile_id, song_id, started_at_ms),
            ScrobbleAction::Submission {
                profile_id,
                song_id,
                started_at_ms,
            } => {
                if self
                    .queue_submission(&profile_id, &song_id, started_at_ms)
                    .await
                    .is_err()
                {
                    tracker.allow_submission_retry(&profile_id, &song_id, started_at_ms);
                }
            }
        }
    }

    fn send_now_playing(&self, profile_id: String, song_id: String, started_at_ms: i64) {
        let Ok((current_profile_id, backend)) = self.server.current_server() else {
            return;
        };
        if current_profile_id != profile_id {
            return;
        }

        tauri::async_runtime::spawn(async move {
            let _ = backend
                .scrobble(&song_id, started_at_ms, ScrobbleEvent::NowPlaying)
                .await;
        });
    }

    async fn queue_submission(
        &self,
        profile_id: &str,
        song_id: &str,
        started_at_ms: i64,
    ) -> Result<(), String> {
        self.repository
            .enqueue(profile_id, song_id, started_at_ms, now_epoch_millis())
            .await
    }

    fn start_retry_worker(self: &Arc<Self>) {
        let service = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            loop {
                service.submit_pending().await;
                tokio::time::sleep(RETRY_INTERVAL).await;
            }
        });
    }

    async fn submit_pending(&self) {
        let Ok((profile_id, backend)) = self.server.current_server() else {
            return;
        };
        let now_ms = now_epoch_millis();
        let Ok(pending) = self
            .repository
            .due(&profile_id, now_ms, RETRY_BATCH_SIZE)
            .await
        else {
            return;
        };

        for scrobble in pending {
            let result = backend
                .scrobble(
                    &scrobble.song_id,
                    scrobble.started_at_ms,
                    ScrobbleEvent::Submission,
                )
                .await;
            if result.is_ok() {
                let _ = self.repository.complete(scrobble.id).await;
            } else {
                self.reschedule(scrobble, now_ms).await;
            }
        }
    }

    async fn reschedule(&self, scrobble: PendingScrobble, now_ms: i64) {
        let attempts = scrobble.attempts.saturating_add(1);
        let exponent = attempts.saturating_sub(1).min(10) as u32;
        let delay_seconds = 5_i64
            .saturating_mul(2_i64.saturating_pow(exponent))
            .min(MAX_RETRY_DELAY_SECONDS);
        let _ = self
            .repository
            .reschedule(
                scrobble.id,
                attempts,
                now_ms.saturating_add(delay_seconds * 1000),
            )
            .await;
    }
}

struct PlaybackSample {
    state: PlaybackState,
    profile_id: Option<String>,
    song_id: Option<String>,
    queue_position: Option<usize>,
    duration_seconds: f64,
}

#[derive(Debug, Eq, PartialEq)]
enum ScrobbleAction {
    NowPlaying {
        profile_id: String,
        song_id: String,
        started_at_ms: i64,
    },
    Submission {
        profile_id: String,
        song_id: String,
        started_at_ms: i64,
    },
}

#[derive(Default)]
struct ScrobbleTracker {
    active: Option<ActivePlayback>,
}

impl ScrobbleTracker {
    /// Updates the tracker with a new playback snapshot and returns the actions
    /// that should be performed as a consequence.
    ///
    /// This method only decides when `NowPlaying` or final submission should
    /// happen. It does not talk to the server or persist anything by itself.
    fn update(
        &mut self,
        sample: &PlaybackSample,
        elapsed: Duration,
        now_ms: i64,
    ) -> Vec<ScrobbleAction> {
        let Some(sample) = self.validated_sample(sample) else {
            return Vec::new();
        };

        if self.is_same_playback(&sample) {
            self.update_active_playback(&sample, elapsed)
        } else {
            vec![self.start_new_playback(&sample, now_ms)]
        }
    }

    fn validated_sample<'a>(&mut self, sample: &'a PlaybackSample) -> Option<TrackerSample<'a>> {
        if sample.state == PlaybackState::Stopped {
            self.reset();
            return None;
        }

        let (Some(profile_id), Some(song_id), Some(queue_position)) = (
            sample.profile_id.as_deref(),
            sample.song_id.as_deref(),
            sample.queue_position,
        ) else {
            self.reset();
            return None;
        };

        Some(TrackerSample {
            profile_id,
            song_id,
            queue_position,
            duration_seconds: sample.duration_seconds,
            is_playing: sample.state == PlaybackState::Playing,
        })
    }

    fn is_same_playback(&self, sample: &TrackerSample<'_>) -> bool {
        self.active.as_ref().is_some_and(|active| {
            active.profile_id == sample.profile_id
                && active.song_id == sample.song_id
                && active.queue_position == sample.queue_position
        })
    }

    fn start_new_playback(&mut self, sample: &TrackerSample<'_>, now_ms: i64) -> ScrobbleAction {
        self.active = Some(ActivePlayback {
            profile_id: sample.profile_id.to_string(),
            song_id: sample.song_id.to_string(),
            queue_position: sample.queue_position,
            started_at_ms: now_ms,
            duration_seconds: sample.duration_seconds,
            listened: Duration::ZERO,
            submission_queued: false,
        });

        ScrobbleAction::NowPlaying {
            profile_id: sample.profile_id.to_string(),
            song_id: sample.song_id.to_string(),
            started_at_ms: now_ms,
        }
    }

    fn update_active_playback(
        &mut self,
        sample: &TrackerSample<'_>,
        elapsed: Duration,
    ) -> Vec<ScrobbleAction> {
        let active = self.active.as_mut().expect("active playback was checked");
        active.duration_seconds = sample.duration_seconds;
        if sample.is_playing {
            active.listened = active.listened.saturating_add(elapsed);
        }

        if !active.submission_queued && active.reached_scrobble_threshold() {
            active.submission_queued = true;
            return vec![ScrobbleAction::Submission {
                profile_id: active.profile_id.clone(),
                song_id: active.song_id.clone(),
                started_at_ms: active.started_at_ms,
            }];
        }

        Vec::new()
    }

    fn allow_submission_retry(&mut self, profile_id: &str, song_id: &str, started_at_ms: i64) {
        if let Some(active) = self.active.as_mut().filter(|active| {
            active.profile_id == profile_id
                && active.song_id == song_id
                && active.started_at_ms == started_at_ms
        }) {
            active.submission_queued = false;
        }
    }

    fn reset(&mut self) {
        self.active = None;
    }
}

struct ActivePlayback {
    profile_id: String,
    song_id: String,
    queue_position: usize,
    started_at_ms: i64,
    duration_seconds: f64,
    listened: Duration,
    submission_queued: bool,
}

struct TrackerSample<'a> {
    profile_id: &'a str,
    song_id: &'a str,
    queue_position: usize,
    duration_seconds: f64,
    is_playing: bool,
}

impl ActivePlayback {
    fn reached_scrobble_threshold(&self) -> bool {
        let threshold = (self.duration_seconds / 2.0).min(MAX_SCROBBLE_THRESHOLD_SECONDS);
        self.listened.as_secs_f64() >= threshold
    }
}

fn elapsed_since_last_sample(last_sample: &mut Instant) -> Duration {
    let now = Instant::now();
    let elapsed = now.duration_since(*last_sample).min(MAX_COUNTED_INTERVAL);
    *last_sample = now;
    elapsed
}

fn now_epoch_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::audio::PlaybackState;

    use super::{PlaybackSample, ScrobbleAction, ScrobbleTracker};

    #[test]
    fn reports_now_playing_when_track_starts() {
        let mut tracker = ScrobbleTracker::default();

        assert_eq!(
            tracker.update(
                &sample("song-1", PlaybackState::Playing, 180.0),
                Duration::ZERO,
                1000
            ),
            [ScrobbleAction::NowPlaying {
                profile_id: "profile".to_string(),
                song_id: "song-1".to_string(),
                started_at_ms: 1000,
            }]
        );
    }

    #[test]
    fn counts_only_playing_time_and_submits_once() {
        let mut tracker = ScrobbleTracker::default();
        tracker.update(
            &sample("song-1", PlaybackState::Playing, 180.0),
            Duration::ZERO,
            1000,
        );

        assert!(tracker
            .update(
                &sample("song-1", PlaybackState::Playing, 180.0),
                Duration::from_secs(89),
                2000,
            )
            .is_empty());
        assert!(tracker
            .update(
                &sample("song-1", PlaybackState::Paused, 180.0),
                Duration::from_secs(30),
                3000,
            )
            .is_empty());
        assert_eq!(
            tracker.update(
                &sample("song-1", PlaybackState::Playing, 180.0),
                Duration::from_secs(1),
                4000,
            ),
            [ScrobbleAction::Submission {
                profile_id: "profile".to_string(),
                song_id: "song-1".to_string(),
                started_at_ms: 1000,
            }]
        );
        assert!(tracker
            .update(
                &sample("song-1", PlaybackState::Playing, 180.0),
                Duration::from_secs(90),
                5000,
            )
            .is_empty());
    }

    #[test]
    fn caps_long_track_threshold_at_four_minutes() {
        let mut tracker = ScrobbleTracker::default();
        tracker.update(
            &sample("song-1", PlaybackState::Playing, 3600.0),
            Duration::ZERO,
            1000,
        );

        assert_eq!(
            tracker.update(
                &sample("song-1", PlaybackState::Playing, 3600.0),
                Duration::from_secs(240),
                2000,
            ),
            [ScrobbleAction::Submission {
                profile_id: "profile".to_string(),
                song_id: "song-1".to_string(),
                started_at_ms: 1000,
            }]
        );
    }

    #[test]
    fn submits_short_tracks_after_half_their_duration() {
        let mut tracker = ScrobbleTracker::default();
        tracker.update(
            &sample("song-1", PlaybackState::Playing, 20.0),
            Duration::ZERO,
            1000,
        );

        assert_eq!(
            tracker.update(
                &sample("song-1", PlaybackState::Playing, 20.0),
                Duration::from_secs(10),
                2000,
            ),
            [ScrobbleAction::Submission {
                profile_id: "profile".to_string(),
                song_id: "song-1".to_string(),
                started_at_ms: 1000,
            }]
        );
    }

    #[test]
    fn starts_new_session_when_queue_advances() {
        let mut tracker = ScrobbleTracker::default();
        tracker.update(
            &sample("song-1", PlaybackState::Playing, 180.0),
            Duration::ZERO,
            1000,
        );

        assert_eq!(
            tracker.update(
                &sample("song-2", PlaybackState::Playing, 180.0),
                Duration::ZERO,
                2000
            ),
            [ScrobbleAction::NowPlaying {
                profile_id: "profile".to_string(),
                song_id: "song-2".to_string(),
                started_at_ms: 2000,
            }]
        );
    }

    fn sample(song_id: &str, state: PlaybackState, duration_seconds: f64) -> PlaybackSample {
        PlaybackSample {
            state,
            profile_id: Some("profile".to_string()),
            song_id: Some(song_id.to_string()),
            queue_position: Some(if song_id == "song-1" { 1 } else { 2 }),
            duration_seconds,
        }
    }
}
