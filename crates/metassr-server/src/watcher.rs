use chrono::Date;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::Path,
    sync::mpsc::{self, Receiver},
};
use tokio::sync::broadcast;

pub struct FileWatcher {
    watcher: RecommendedWatcher,
    sender: broadcast::Sender<Event>,
}

impl FileWatcher {
    pub fn new() -> notify::Result<Self> {
        // Create a broadcast channel with capacity for 100 messages
        // distributing file events to multiple subscribers
        let (sender, _) = broadcast::channel(100);
        let tx = sender.clone();
        let (notify_tx, notify_rx) = mpsc::channel();

        std::thread::spawn(move || {
            while let Ok(event) = notify_rx.recv() {
                if is_relevant_event(&event) {
                    let _ = tx.send(event);
                }
            }
        });

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| match res {
                Ok(event) => {
                    if is_relevant_event(&event) {
                        println!("Detected change: {:?}", event);
                        let _ = notify_tx.send(event);
                    }
                }
                Err(errr) => {
                    eprintln!("Error: {}", errr);
                }
            },
            Config::default(),
        )?;

        Ok(FileWatcher { watcher, sender })
    }

    pub fn watch(&mut self, path: &Path) -> notify::Result<()> {
        self.watcher.watch(path, RecursiveMode::Recursive)?;

        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}

// Helper functions

fn is_relevant_event(event: &Event) -> bool {
    use notify::event::{CreateKind, Event, EventKind, ModifyKind};
    use notify::EventKind::*;

    match event.kind {
        Create(_) => true,
        Modify(ModifyKind::Data(_)) => true,
        Modify(ModifyKind::Name(_)) => true,
        Remove(_) => true,
        _ => false,
    }
}

fn format_event(event: &Event) -> String {
    use notify::event::{CreateKind, Event, EventKind, ModifyKind};
    use notify::EventKind::*;

    let action = match event.kind {
        Create(_) => "created",
        Modify(ModifyKind::Data(_)) => "modified",
        Modify(ModifyKind::Name(_)) => "renamed",
        Remove(_) => "deleted",
        _ => "unknown action",
    };

    let paths = event
        .paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    format!("{} {}", action, paths)
}
