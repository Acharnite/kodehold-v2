use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use notify::{Config, Event, EventKind, RecursiveMode, Watcher as NotifyWatcher};

use crate::events::{Dispatcher, Event as KodeholdEvent, EventSource};

pub struct Watcher {
    _handle: Box<dyn NotifyWatcher>,
}

impl Watcher {
    pub fn start(paths: &[String], dispatcher: Dispatcher) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();

        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;

        for p in paths {
            let watch_path = Path::new(p);
            if watch_path.exists() {
                watcher.watch(watch_path, RecursiveMode::Recursive)?;
                eprintln!("  watching: {p}");
            }
        }

        // Debounce: batch events within 2s windows
        std::thread::spawn(move || {
            let debounce = Duration::from_secs(2);
            let mut last_trigger = Instant::now();
            let mut pending = false;

            loop {
                match rx.recv_timeout(Duration::from_millis(500)) {
                    Ok(Ok(_event)) => {
                        pending = true;
                        last_trigger = Instant::now();
                    }
                    Ok(Err(e)) => {
                        eprintln!("watcher error: {:?}", e);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if pending && last_trigger.elapsed() >= debounce {
                            let payload = serde_json::json!({"trigger": "file_change"});
                            let event =
                                KodeholdEvent::new(EventSource::LocalWatcher, "file_change", payload);
                            if let Err(e) = dispatcher.dispatch(&event) {
                                eprintln!("watcher dispatch error: {e}");
                            }
                            pending = false;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        Ok(Self {
            _handle: Box::new(watcher),
        })
    }
}
