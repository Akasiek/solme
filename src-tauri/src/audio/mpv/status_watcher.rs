use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use rsmpv::{Client, Event, Format, Mpv, PropertyData};

use crate::audio::backend::{AudioStatusChange, AudioStatusChangeCallback};

type Callback = Arc<Mutex<Option<AudioStatusChangeCallback>>>;

pub(super) struct StatusWatcher {
    stopped: Arc<AtomicBool>,
    worker: JoinHandle<()>,
}

impl StatusWatcher {
    pub fn start(mpv: Arc<Mpv>, callback: Callback) -> Result<Self, String> {
        let stopped = Arc::new(AtomicBool::new(false));
        let worker_stopped = Arc::clone(&stopped);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);

        let worker = thread::Builder::new()
            .name("mpv-status".to_string())
            .spawn(move || {
                let client = create_client(&mpv);
                let ready = client.as_ref().map(|_| ()).map_err(Clone::clone);
                if ready_tx.send(ready).is_err() {
                    return;
                }

                if let Ok(client) = client {
                    watch(&client, &callback, &worker_stopped);
                }
            })
            .map_err(|error| format!("Failed to start MPV status watcher: {error}"))?;

        let watcher = Self { stopped, worker };
        ready_rx
            .recv()
            .map_err(|error| format!("MPV status watcher did not initialize: {error}"))??;

        Ok(watcher)
    }
}

impl Drop for StatusWatcher {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        self.worker.thread().unpark();
        // The worker owns its core reference and drops its client before the core.
        // Do not join: a final callback can release the backend on this thread.
    }
}

fn create_client(mpv: &Mpv) -> Result<Client<'_>, String> {
    let client = mpv
        .create_client(None)
        .map_err(|error| format!("Failed to create MPV status client: {error}"))?;

    for (id, name, format) in [
        (1, "playlist-pos", Format::Int64),
        (2, "idle-active", Format::Flag),
        (3, "playlist", Format::None),
    ] {
        client
            .observe_property(id, name, format)
            .map_err(|error| format!("Failed to observe MPV {name}: {error}"))?;
    }

    Ok(client)
}

fn watch(client: &Client<'_>, callback: &Callback, stopped: &AtomicBool) {
    let worker = thread::current();
    client.set_wakeup_callback(move || worker.unpark());
    let mut was_idle = true;
    let mut last_entry_id = None;

    while !stopped.load(Ordering::Acquire) {
        let Some(event) = client.poll_event() else {
            thread::park_timeout(Duration::from_mins(1));
            continue;
        };
        if matches!(event, Event::Shutdown) {
            return;
        }
        let Some(change) = status_change(event, &mut was_idle, &mut last_entry_id) else {
            continue;
        };

        notify(callback, change);
    }
}

fn status_change(
    event: Event,
    was_idle: &mut bool,
    last_entry_id: &mut Option<i64>,
) -> Option<AudioStatusChange> {
    match event {
        Event::StartFile { playlist_entry_id } => {
            *last_entry_id = Some(playlist_entry_id);
            *was_idle = false;
            Some(AudioStatusChange::Changed)
        }
        Event::FileLoaded => Some(AudioStatusChange::Changed),
        Event::EndFile { error, .. } => {
            if let Some(error) = error {
                log::warn!("MPV could not play a queue entry: {error}");
            }
            Some(AudioStatusChange::Changed)
        }
        Event::PropertyChange { name, data, .. } => {
            property_change(&name, data, was_idle, *last_entry_id)
        }
        _ => None,
    }
}

fn property_change(
    name: &str,
    data: PropertyData,
    was_idle: &mut bool,
    last_entry_id: Option<i64>,
) -> Option<AudioStatusChange> {
    match (name, data) {
        ("idle-active", PropertyData::Flag(idle)) => {
            let change = if idle && !*was_idle {
                AudioStatusChange::BecameIdle { last_entry_id }
            } else {
                AudioStatusChange::Changed
            };
            *was_idle = idle;
            Some(change)
        }
        ("playlist-pos" | "playlist", _) => Some(AudioStatusChange::Changed),
        _ => None,
    }
}

fn notify(callback: &Callback, change: AudioStatusChange) {
    let callback = if let Ok(callback) = callback.lock() {
        callback.clone()
    } else {
        log::error!("Failed to read MPV status change callback");
        return;
    };

    if let Some(callback) = callback {
        callback(change);
    }
}
