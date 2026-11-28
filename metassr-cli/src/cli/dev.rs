use std::env::current_dir;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

use anyhow::{Context, Result};

use crate::cli::traits::Exec;
use crate::cli::{Builder, BuildingType};
use metassr_build::rebuilder::{RebuildType, Rebuilder};
use metassr_build::{
    client::ClientBuilder, server::ServerSideBuilder, traits::Build, DuplicateRoutesPolicy,
};
use metassr_server::{RunningType, Server, ServerConfigs};
use metassr_utils::ansi::ansi_regex;
use metassr_watcher::FileWatcher;

use tracing::{debug, error, warn};

use super::traits::AsyncExec;

pub struct Dev {
    port: u16,
    ws_port: u16,
    // todo change this to a normal option, and edit impl asyncexec
    watcher: Arc<Mutex<Option<FileWatcher>>>,
    /// Directories currently watched by the file watcher.
    watched_dirs: Arc<Mutex<Vec<PathBuf>>>,
    /// Directories that do not exist yet, they are watched once created.
    pending_dirs: Arc<Mutex<Vec<PathBuf>>>,
    rebuilder: Arc<Rebuilder>,
    root_path: PathBuf,
    rebuild_tx: broadcast::Sender<RebuildType>,
    build_type: BuildingType,
    out_dir: String,
    allow_http_debug: bool,
}

impl Dev {
    pub fn new(
        port: u16,
        ws_port: u16,
        root_path: PathBuf,
        out_dir: String,
        build_type: BuildingType,
        allow_http_debug: bool,
    ) -> Result<Self> {
        let (rebuild_tx, _) = broadcast::channel(100); //channel for rebuild notifications

        let watcher = Arc::new(Mutex::new(None)); //FileWatcher::new()?;
        let watched_dirs = Arc::new(Mutex::new(Vec::new()));
        let pending_dirs = Arc::new(Mutex::new(Vec::new()));
        let rebuilder = Arc::new(Rebuilder::new(
            root_path.clone(),
            build_type,
            PathBuf::from(&out_dir),
        )?);

        Ok(Self {
            port,
            ws_port,
            watcher,
            watched_dirs,
            pending_dirs,
            rebuilder,
            root_path,
            rebuild_tx,
            build_type,
            out_dir: out_dir.to_string(),
            allow_http_debug,
        })
    }

    fn setup_watcher(&self) -> Result<()> {
        let mut watcher = FileWatcher::new()?;

        // `src/` is required (the initial build above just succeeded), so a
        // missing directory here is a genuine misconfiguration: keep it fatal.
        // `static/` is not watched: assets are served from disk per request,
        // so it may not exist.
        let src_dir = self.root_path.join("src");

        watcher
            .watch(&src_dir)
            .with_context(|| format!("Failed to watch directory '{}'", src_dir.display()))?;

        *self.watched_dirs.lock().unwrap() = vec![src_dir];

        // store the watcher in the option, by modifying it with a lock on the mutex
        let mut watcher_guard = self.watcher.lock().unwrap();
        *watcher_guard = Some(watcher);
        Ok(())
    }

    async fn handle_file_changes(&self) -> Result<()> {
        let watcher_guard = self.watcher.lock().unwrap();
        let watcher = watcher_guard.as_ref().unwrap();
        let mut rx = watcher.subscribe();
        drop(watcher_guard); // drop the lock, we don't need it anymore

        let rebuilder = self.rebuilder.clone();
        let rebuild_tx = self.rebuild_tx.clone();
        let file_watcher = self.watcher.clone();
        let watched_dirs = self.watched_dirs.clone();
        let pending_dirs = self.pending_dirs.clone();

        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                // A directory that did not exist when the dev server started may
                // have been created in the meantime.
                watch_created_dirs(&file_watcher, &pending_dirs, &watched_dirs, &event.paths);

                if !is_watched_event(&watched_dirs, &event.paths) {
                    debug!("Skipping event outside of the watched directories");
                    continue;
                }

                match rebuilder.handle_event(event) {
                    Ok(Some(rebuild_type)) => {
                        // Notify the server about what needs rebuilding
                        if let Err(err) = rebuild_tx.send(rebuild_type) {
                            error!("Error sending rebuild notification: {}", err);
                        }
                    }
                    Ok(None) => {
                        debug!("Skipping irrelevant watcher event");
                    }
                    Err(err) => {
                        warn!("Could not map file-change event to a rebuild type: {}", err);
                    }
                }
            }
        });

        Ok(())
    }

    async fn start_server(&self) -> Result<()> {
        let mut rebuild_rx: broadcast::Receiver<RebuildType> = self.rebuild_tx.subscribe();

        let rebuilder = Arc::clone(&self.rebuilder);

        tokio::spawn({
            let rebuilder = Arc::clone(&rebuilder);

            async move {
                while let Ok(rebuild_type) = rebuild_rx.recv().await {
                    if let Err(e) = rebuilder.clone().rebuild(rebuild_type) {
                        let err_msg = e.to_string();
                        let clean_log_msg = ansi_regex().replace_all(&err_msg, "");
                        error!("Rebuild failed: {}", clean_log_msg);
                    }
                }
            }
        });

        let server_configs = ServerConfigs {
            port: self.port,
            ws_port: self.ws_port,
            _enable_http_logging: self.allow_http_debug,
            root_path: self.root_path.clone(),
            running_type: RunningType::ServerSideRendering,
            mode: metassr_server::ServerMode::Development,
            rebuilder: Some(rebuilder),
        };

        Server::new(server_configs).run().await?;
        Ok(())
    }
}

impl AsyncExec for Dev {
    async fn exec(&self) -> Result<()> {
        Builder::new(self.build_type, self.out_dir.clone())
            .with_duplicate_routes_policy(DuplicateRoutesPolicy::Warn)
            .exec()?;

        self.setup_watcher()?;

        let current = current_dir()?;
        debug!("Current directory: {:?}", current);

        let cache_dir = current.join("dist/cache/pages");
        debug!("Checking cache directory: {:?}", cache_dir);

        // perform an initial build pass to catch any pre existing errors
        // client build
        if let Err(e) = ClientBuilder::new("", &self.out_dir, true)?.build() {
            // exec() now propagates the bundling error through its return value,
            // so e already contains the full compilation error message.
            let err_msg = format!("Client-side build failed: {}", e);
            let clean_log_msg = ansi_regex().replace_all(&err_msg, "");

            error!(
                "Initial build failed, caching error for overlay: {}",
                clean_log_msg
            );
            self.rebuilder.set_last_errors(vec![err_msg]);
        } else {
            // server build
            let stype = self.rebuilder.building_type();
            if let Err(e) = ServerSideBuilder::new("", &self.out_dir, stype, true)?.build() {
                let err_msg = format!("Server build failed: {}", e);
                let clean_log_msg = ansi_regex().replace_all(&err_msg, "");
                error!(
                    "Initial build failed, caching error for overlay: {}",
                    clean_log_msg
                );
                self.rebuilder.set_last_errors(vec![err_msg]);
            }
        }

        self.handle_file_changes().await?;

        self.start_server().await?;

        Ok(())
    }
}

/// Watches the directories that did not exist when the dev server started and
/// have been created since then.
fn watch_created_dirs(
    file_watcher: &Arc<Mutex<Option<FileWatcher>>>,
    pending_dirs: &Arc<Mutex<Vec<PathBuf>>>,
    watched_dirs: &Arc<Mutex<Vec<PathBuf>>>,
    paths: &[PathBuf],
) {
    let mut pending = pending_dirs.lock().unwrap();
    if pending.is_empty() {
        return;
    }

    let created: Vec<PathBuf> = pending
        .iter()
        .filter(|dir| dir.is_dir() && paths.iter().any(|path| path == *dir))
        .cloned()
        .collect();

    if created.is_empty() {
        return;
    }

    let mut watcher_guard = file_watcher.lock().unwrap();
    let Some(watcher) = watcher_guard.as_mut() else {
        return;
    };

    for dir in created {
        match watcher.watch(&dir) {
            Ok(()) => {
                debug!("Watching '{}'", dir.display());
                watched_dirs.lock().unwrap().push(dir.clone());
                pending.retain(|pending_dir| pending_dir != &dir);
            }
            Err(err) => error!("Failed to watch directory '{}': {}", dir.display(), err),
        }
    }
}

/// Whether the given paths are inside one of the watched directories.
///
/// Events on the watched directories themselves are ignored, they carry no
/// meaningful change.
fn is_watched_event(watched_dirs: &Arc<Mutex<Vec<PathBuf>>>, paths: &[PathBuf]) -> bool {
    let watched = watched_dirs.lock().unwrap();

    paths.iter().any(|path| {
        watched
            .iter()
            .any(|dir| path != dir && path.starts_with(dir))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("metassr-dev-{name}-{}", std::process::id()));

        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        dir
    }

    fn dev(root_path: PathBuf) -> Dev {
        Dev::new(
            8080,
            8561,
            root_path,
            "dist".to_string(),
            BuildingType::ServerSideRendering,
            false,
        )
        .unwrap()
    }

    #[test]
    fn setup_watcher_succeeds_with_no_static_dir() {
        let root = temp_dir("missing-static");
        fs::create_dir_all(root.join("src")).unwrap();
        // deliberately no `static/` directory
        assert!(!root.join("static").exists());

        let dev = dev(root.clone());
        let result = dev.setup_watcher();

        assert!(
            result.is_ok(),
            "setup_watcher failed without static/: {:?}",
            result.unwrap_err()
        );

        let watched = dev.watched_dirs.lock().unwrap().clone();
        assert_eq!(watched, vec![root.join("src")]);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_directories_are_watched_once_created() {
        let root = temp_dir("created-later");
        let static_dir = root.join("static");

        let file_watcher = Arc::new(Mutex::new(Some(FileWatcher::new().unwrap())));
        let pending_dirs = Arc::new(Mutex::new(vec![static_dir.clone()]));
        let watched_dirs = Arc::new(Mutex::new(Vec::new()));

        // The directory does not exist yet, so it is not watched.
        watch_created_dirs(
            &file_watcher,
            &pending_dirs,
            &watched_dirs,
            std::slice::from_ref(&static_dir),
        );
        assert!(watched_dirs.lock().unwrap().is_empty());

        // Once created, it is watched.
        fs::create_dir_all(&static_dir).unwrap();
        watch_created_dirs(
            &file_watcher,
            &pending_dirs,
            &watched_dirs,
            std::slice::from_ref(&static_dir),
        );

        assert_eq!(
            watched_dirs.lock().unwrap().clone(),
            vec![static_dir.clone()]
        );
        assert!(pending_dirs.lock().unwrap().is_empty());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn events_outside_of_the_watched_directories_are_ignored() {
        let watched_dirs = Arc::new(Mutex::new(vec![PathBuf::from("/project/src")]));

        assert!(is_watched_event(
            &watched_dirs,
            &[PathBuf::from("/project/src/pages/index.tsx")]
        ));
        assert!(!is_watched_event(
            &watched_dirs,
            &[PathBuf::from("/project/package.json")]
        ));
        // The watched directory itself carries no meaningful change.
        assert!(!is_watched_event(
            &watched_dirs,
            &[PathBuf::from("/project/src")]
        ));
        assert!(!is_watched_event(&watched_dirs, &[]));
    }
}
