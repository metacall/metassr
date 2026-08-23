use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

use anyhow::{self, Result};

use crate::cli::traits::Exec;
use crate::cli::{Builder, BuildingType};
use metassr_build::rebuilder::{RebuildType, Rebuilder};
use metassr_build::{client::ClientBuilder, server::ServerSideBuilder, traits::Build};
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
        let rebuilder = Arc::new(Rebuilder::new(
            root_path.clone(),
            build_type,
            PathBuf::from(&out_dir),
        )?);

        Ok(Self {
            port,
            ws_port,
            watcher,
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

        let src_dir = current_dir()?.join("src");
        let static_dir = current_dir()?.join("static");

        watcher.watch(Path::new(&src_dir))?;
        watcher.watch(Path::new(&static_dir))?;

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

        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
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
        Builder::new(self.build_type, self.out_dir.clone()).exec()?;

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
