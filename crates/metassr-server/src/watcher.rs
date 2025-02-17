use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::Path, sync::mpsc::Receiver};
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

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                let tx = tx.clone();
                tokio::spawn(async move {
                    match res {
                        Ok(event) => {
                            println!("Detected change: {:?}", event.paths);
                            let _ = tx.send(event); // ignore send errors
                        }
                        Err(err) => {
                            eprintln!("watchr error: {:?}", err);
                        }
                    }
                });
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
