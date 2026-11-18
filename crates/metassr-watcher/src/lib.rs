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

    /// Watches the given path and all of its subdirectories.
    pub fn watch(&mut self, path: &Path) -> notify::Result<()> {
        self.watcher.watch(path, RecursiveMode::Recursive)?;

        Ok(())
    }

    /// Watches the given path without descending into its subdirectories.
    ///
    /// This is useful for detecting the creation of directories that do not
    /// exist yet, without watching the whole project.
    pub fn watch_non_recursive(&mut self, path: &Path) -> notify::Result<()> {
        self.watcher.watch(path, RecursiveMode::NonRecursive)?;

        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DebouncedEvent> {
        self.sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs;

    fn missing_path() -> std::path::PathBuf {
        temp_dir().join(format!("metassr-watcher-missing-{}", std::process::id()))
    }

    // Each test needs its own directory: the tests run in parallel, so a
    // shared path lets one test's cleanup delete the directory while another
    // is still watching it.
    fn existing_path(name: &str) -> std::path::PathBuf {
        let path = temp_dir().join(format!(
            "metassr-watcher-existing-{}-{}",
            name,
            std::process::id()
        ));

        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();

        path
    }

    #[test]
    fn watching_a_missing_path_fails() {
        let mut watcher = FileWatcher::new().unwrap();

        assert!(watcher.watch(&missing_path()).is_err());
    }

    #[test]
    fn watching_an_existing_path_succeeds() {
        let mut watcher = FileWatcher::new().unwrap();
        let path = existing_path("recursive");

        // Report the underlying notify::Error on failure (e.g. inotify
        // `ENOSPC`/`EMFILE` inside containers) instead of discarding it.
        let result = watcher.watch(&path);
        assert!(
            result.is_ok(),
            "watch({path:?}) failed: {:?}",
            result.unwrap_err()
        );

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn watching_non_recursively_succeeds() {
        let mut watcher = FileWatcher::new().unwrap();
        let path = existing_path("non-recursive");

        let result = watcher.watch_non_recursive(&path);
        assert!(
            result.is_ok(),
            "watch_non_recursive({path:?}) failed: {:?}",
            result.unwrap_err()
        );

        let _ = fs::remove_dir_all(&path);
    }
}
