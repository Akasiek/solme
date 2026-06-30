use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use super::backend::{AudioBackend, AudioBackendStatus, AudioStatusChangeCallback};

const FADE_DURATION: Duration = Duration::from_millis(300);
const FADE_STEPS: u32 = 12;

#[derive(Clone)]
pub struct FadingAudioBackend {
    inner: Arc<dyn AudioBackend>,
    state: Arc<Mutex<FadeState>>,
}

struct FadeState {
    generation: u64,
    volume: f64,
    phase: FadePhase,
}

#[derive(Clone, Copy, PartialEq)]
enum FadePhase {
    Idle,
    Pausing,
    Resuming,
}

impl FadingAudioBackend {
    pub fn new(inner: Arc<dyn AudioBackend>) -> Self {
        let volume = inner.status().volume;

        Self {
            inner,
            state: Arc::new(Mutex::new(FadeState {
                generation: 0,
                volume,
                phase: FadePhase::Idle,
            })),
        }
    }

    fn begin_fade(&self, phase: FadePhase) -> Result<u64, String> {
        let mut state = self.lock_state()?;
        state.generation = state.generation.wrapping_add(1);
        state.phase = phase;
        Ok(state.generation)
    }

    fn cancel(&self) -> Result<(), String> {
        let volume = {
            let mut state = self.lock_state()?;
            state.generation = state.generation.wrapping_add(1);
            state.phase = FadePhase::Idle;
            state.volume
        };

        self.inner.set_volume(volume)
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, FadeState>, String> {
        self.state
            .lock()
            .map_err(|_| "Audio fade lock was poisoned".to_string())
    }

    fn is_fade_running(&self, phase: FadePhase) -> Result<bool, String> {
        Ok(self.lock_state()?.phase == phase)
    }

    fn pause_with_fade(&self) -> Result<(), String> {
        let status = self.inner.status();
        let is_already_paused = status.paused;
        let is_pausing = self.is_fade_running(FadePhase::Pausing)?;

        if is_already_paused || is_pausing {
            return Ok(());
        }
        if !status.playing {
            return self.inner.pause();
        }

        let generation = self.begin_fade(FadePhase::Pausing)?;
        spawn_fade_out(
            Arc::clone(&self.inner),
            Arc::clone(&self.state),
            generation,
            status.volume,
        );
        Ok(())
    }

    fn resume_with_fade(&self) -> Result<(), String> {
        let is_paused = self.inner.status().paused;
        let is_resuming = self.is_fade_running(FadePhase::Resuming)?;

        if is_paused && is_resuming {
            return Ok(());
        }

        let generation = self.begin_fade(FadePhase::Resuming)?;
        let status = self.inner.status();
        let start_volume = if status.paused {
            self.inner.set_volume(0.0)?;
            self.inner.resume()?;
            0.0
        } else {
            self.inner.resume()?;
            status.volume
        };

        spawn_fade_in(
            Arc::clone(&self.inner),
            Arc::clone(&self.state),
            generation,
            start_volume,
        );
        Ok(())
    }

    fn set_logical_volume(&self, volume: f64) -> Result<(), String> {
        let volume = volume.clamp(0.0, 100.0);
        let should_apply = {
            let mut state = self.lock_state()?;
            state.volume = volume;
            state.phase == FadePhase::Idle
        };

        if should_apply {
            self.inner.set_volume(volume)?;
        }
        Ok(())
    }

    fn volume(&self) -> Result<f64, String> {
        Ok(self.lock_state()?.volume)
    }
}

impl AudioBackend for FadingAudioBackend {
    fn load_queue(&self, sources: &[String], start_index: usize) -> Result<(), String> {
        self.cancel()?;
        self.inner.load_queue(sources, start_index)
    }

    fn load_queue_paused(
        &self,
        sources: &[String],
        start_index: usize,
        position_seconds: Option<f64>,
    ) -> Result<(), String> {
        self.cancel()?;
        self.inner
            .load_queue_paused(sources, start_index, position_seconds)
    }

    fn prepend_queue(&self, sources: &[String]) -> Result<(), String> {
        self.inner.prepend_queue(sources)
    }

    fn append_queue(&self, sources: &[String]) -> Result<(), String> {
        self.inner.append_queue(sources)
    }

    fn pause(&self) -> Result<(), String> {
        self.pause_with_fade()
    }

    fn resume(&self) -> Result<(), String> {
        self.resume_with_fade()
    }

    fn stop(&self) -> Result<(), String> {
        self.cancel()?;
        self.inner.stop()
    }

    fn next(&self) -> Result<(), String> {
        self.cancel()?;
        self.inner.next()
    }

    fn previous(&self) -> Result<(), String> {
        self.cancel()?;
        self.inner.previous()
    }

    fn seek(&self, position_seconds: f64) -> Result<(), String> {
        self.inner.seek(position_seconds)
    }

    fn set_volume(&self, volume: f64) -> Result<(), String> {
        self.set_logical_volume(volume)
    }

    fn status(&self) -> AudioBackendStatus {
        let mut status = self.inner.status();
        if let Ok(volume) = self.volume() {
            status.volume = volume;
        }
        status
    }

    fn set_status_change_callback(&self, callback: AudioStatusChangeCallback) {
        self.inner.set_status_change_callback(callback);
    }
}

fn spawn_fade_out(
    audio: Arc<dyn AudioBackend>,
    state: Arc<Mutex<FadeState>>,
    generation: u64,
    start_volume: f64,
) {
    tauri::async_runtime::spawn(async move {
        for step in 1..=FADE_STEPS {
            tokio::time::sleep(fade_step_duration()).await;
            if !is_current_fade(&state, generation) {
                return;
            }

            let progress = f64::from(step) / f64::from(FADE_STEPS);
            if audio.set_volume(start_volume * (1.0 - progress)).is_err() {
                finish_fade(&state, generation);
                return;
            }
        }

        if !is_current_fade(&state, generation) || audio.pause().is_err() {
            finish_fade(&state, generation);
            return;
        }

        if let Some(volume) = current_volume(&state, generation) {
            let _ = audio.set_volume(volume);
        }
        finish_fade(&state, generation);
    });
}

fn spawn_fade_in(
    audio: Arc<dyn AudioBackend>,
    state: Arc<Mutex<FadeState>>,
    generation: u64,
    start_volume: f64,
) {
    tauri::async_runtime::spawn(async move {
        for step in 1..=FADE_STEPS {
            tokio::time::sleep(fade_step_duration()).await;
            let Some(target_volume) = current_volume(&state, generation) else {
                return;
            };

            let progress = f64::from(step) / f64::from(FADE_STEPS);
            let volume = start_volume + (target_volume - start_volume) * progress;
            if audio.set_volume(volume).is_err() {
                finish_fade(&state, generation);
                return;
            }
        }

        finish_fade(&state, generation);
    });
}

fn fade_step_duration() -> Duration {
    FADE_DURATION / FADE_STEPS
}

fn is_current_fade(state: &Mutex<FadeState>, generation: u64) -> bool {
    state
        .lock()
        .is_ok_and(|state| state.generation == generation)
}

fn current_volume(state: &Mutex<FadeState>, generation: u64) -> Option<f64> {
    state
        .lock()
        .ok()
        .filter(|state| state.generation == generation)
        .map(|state| state.volume)
}

fn finish_fade(state: &Mutex<FadeState>, generation: u64) {
    if let Ok(mut state) = state.lock() {
        if state.generation == generation {
            state.phase = FadePhase::Idle;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::{AudioBackend, FadingAudioBackend, FADE_DURATION};
    use crate::audio::backend::{AudioBackendStatus, AudioStatusChangeCallback};

    #[test]
    fn fades_out_before_pausing_and_preserves_logical_volume() {
        tauri::async_runtime::block_on(async {
            let state = Arc::new(Mutex::new(MockState::playing(80.0)));
            let audio = fading_audio(Arc::clone(&state));

            audio.pause().unwrap();

            assert_eq!(audio.status().volume, 80.0);
            tokio::time::sleep(FADE_DURATION + FADE_DURATION).await;

            let state = state.lock().unwrap();
            assert!(state.paused);
            assert_eq!(state.volume, 80.0);
        });
    }

    #[test]
    fn fades_in_after_resuming() {
        tauri::async_runtime::block_on(async {
            let state = Arc::new(Mutex::new(MockState {
                playing: true,
                paused: true,
                volume: 70.0,
            }));
            let audio = fading_audio(Arc::clone(&state));

            audio.resume().unwrap();
            assert!(!state.lock().unwrap().paused);

            tokio::time::sleep(FADE_DURATION + FADE_DURATION).await;
            assert_eq!(state.lock().unwrap().volume, 70.0);
        });
    }

    #[test]
    fn resume_cancels_pending_pause() {
        tauri::async_runtime::block_on(async {
            let state = Arc::new(Mutex::new(MockState::playing(60.0)));
            let audio = fading_audio(Arc::clone(&state));

            audio.pause().unwrap();
            tokio::time::sleep(FADE_DURATION / 3).await;
            audio.resume().unwrap();
            tokio::time::sleep(FADE_DURATION + FADE_DURATION).await;

            let state = state.lock().unwrap();
            assert!(!state.paused);
            assert_eq!(state.volume, 60.0);
        });
    }

    #[test]
    fn volume_changed_during_fade_is_restored_after_pause() {
        tauri::async_runtime::block_on(async {
            let state = Arc::new(Mutex::new(MockState::playing(90.0)));
            let audio = fading_audio(Arc::clone(&state));

            audio.pause().unwrap();
            audio.set_volume(35.0).unwrap();
            tokio::time::sleep(FADE_DURATION + FADE_DURATION).await;

            let state = state.lock().unwrap();
            assert!(state.paused);
            assert_eq!(state.volume, 35.0);
        });
    }

    #[test]
    fn stop_cancels_pending_fade_and_restores_logical_volume() {
        tauri::async_runtime::block_on(async {
            let state = Arc::new(Mutex::new(MockState::playing(65.0)));
            let audio = fading_audio(Arc::clone(&state));

            audio.pause().unwrap();
            tokio::time::sleep(FADE_DURATION / 3).await;
            audio.stop().unwrap();
            tokio::time::sleep(FADE_DURATION + FADE_DURATION).await;

            let state = state.lock().unwrap();
            assert!(!state.playing);
            assert_eq!(state.volume, 65.0);
        });
    }

    fn fading_audio(state: Arc<Mutex<MockState>>) -> FadingAudioBackend {
        let audio: Arc<dyn AudioBackend> = Arc::new(MockBackend::new(state));
        FadingAudioBackend::new(audio)
    }

    struct MockState {
        playing: bool,
        paused: bool,
        volume: f64,
    }

    impl MockState {
        fn playing(volume: f64) -> Self {
            Self {
                playing: true,
                paused: false,
                volume,
            }
        }
    }

    struct MockBackend {
        state: Arc<Mutex<MockState>>,
    }

    impl MockBackend {
        fn new(state: Arc<Mutex<MockState>>) -> Self {
            Self { state }
        }
    }

    impl AudioBackend for MockBackend {
        fn load_queue(&self, _sources: &[String], _start_index: usize) -> Result<(), String> {
            self.state.lock().unwrap().playing = true;
            Ok(())
        }

        fn set_status_change_callback(&self, _callback: AudioStatusChangeCallback) {}

        fn load_queue_paused(
            &self,
            _sources: &[String],
            _start_index: usize,
            _position_seconds: Option<f64>,
        ) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            state.playing = true;
            state.paused = true;
            Ok(())
        }

        fn append_queue(&self, _sources: &[String]) -> Result<(), String> {
            Ok(())
        }

        fn prepend_queue(&self, _sources: &[String]) -> Result<(), String> {
            Ok(())
        }

        fn pause(&self) -> Result<(), String> {
            self.state.lock().unwrap().paused = true;
            Ok(())
        }

        fn resume(&self) -> Result<(), String> {
            self.state.lock().unwrap().paused = false;
            Ok(())
        }

        fn stop(&self) -> Result<(), String> {
            self.state.lock().unwrap().playing = false;
            Ok(())
        }

        fn next(&self) -> Result<(), String> {
            Ok(())
        }

        fn previous(&self) -> Result<(), String> {
            Ok(())
        }

        fn seek(&self, _position_seconds: f64) -> Result<(), String> {
            Ok(())
        }

        fn set_volume(&self, volume: f64) -> Result<(), String> {
            self.state.lock().unwrap().volume = volume;
            Ok(())
        }

        fn status(&self) -> AudioBackendStatus {
            let state = self.state.lock().unwrap();
            AudioBackendStatus {
                playing: state.playing,
                paused: state.paused,
                position_seconds: 0.0,
                duration_seconds: 180.0,
                playlist_position: Some(0),
                volume: state.volume,
            }
        }
    }
}
