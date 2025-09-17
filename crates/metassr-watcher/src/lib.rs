pub mod utils;

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::Path,
    sync::mpsc::{self, Receiver},
};
use tokio::sync::broadcast;
use utils::{format_event, is_relevant_event};

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

        // spawn a new thread to handle file events
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
                        println!("File system change detected: {}", format_event(&event));
                        let _ = notify_tx.send(event);
                    }
                }
                Err(err) => {
                    eprintln!("Error: {err}");
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
