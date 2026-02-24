pub mod utils;

use notify::RecursiveMode;
use notify_debouncer_full::{self, DebounceEventResult, DebouncedEvent};
use std::{path::Path, time::Duration};
use tokio::sync::broadcast;

pub struct FileWatcher {
    watcher: notify_debouncer_full::Debouncer<
        notify::RecommendedWatcher,
        notify_debouncer_full::RecommendedCache,
    >,
    sender: broadcast::Sender<DebouncedEvent>,
}

impl FileWatcher {
    pub fn new() -> notify::Result<Self> {
        // Create a broadcast channel with capacity for 100 messages
        // distributing file events to multiple subscribers
        let (sender, _) = broadcast::channel(100);
        let tx = sender.clone();

        let watcher = notify_debouncer_full::new_debouncer(
            Duration::from_millis(100),
            None,
            move |res: DebounceEventResult| match res {
                Ok(events) => {
                    for event in events {
                        let _ = tx.send(event);
                    }
                }
                Err(errors) => {
                    for err in errors {
                        eprintln!("Watch Error: {err}");
                    }
                }
            },
        )?;

        Ok(FileWatcher { watcher, sender })
    }

    pub fn watch(&mut self, path: &Path) -> notify::Result<()> {
        self.watcher.watch(path, RecursiveMode::Recursive)?;

        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DebouncedEvent> {
        self.sender.subscribe()
    }
}
