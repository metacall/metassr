use notify::{event, Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
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
                let _ = tx.send(event);
            }
        });

        let watcher = RecommendedWatcher::new(
            move |res| match res {
                Ok(event) => {
                    println!("Event: {event:?}");
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
