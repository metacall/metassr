use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use tokio::sync::broadcast;

use metacall::initialize;
use metassr_build::server::BuildingType;
use metassr_build::{client::ClientBuilder, server::ServerSideBuilder, traits::Build};
use metassr_server::rebuilder::{RebuildType, Rebuilder};
use metassr_server::{RunningType, Server, ServerConfigs};
use metassr_watcher::FileWatcher;
use regex::Regex;
use tracing::{debug, error};

use super::traits::AsyncExec;

pub struct Dev {
    port: u16,
    ws_port: u16,
    // todo change this to a normal option, and edit impl asyncexec
    watcher: Arc<Mutex<Option<FileWatcher>>>,
    rebuilder: Arc<Rebuilder>,
    root_path: PathBuf,
    rebuild_tx: broadcast::Sender<RebuildType>,
}

impl Dev {
    pub fn new(
        port: u16,
        ws_port: u16,
        root_path: PathBuf,
        building_type: BuildingType,
    ) -> Result<Self> {
        let (rebuild_tx, _) = broadcast::channel(100); //channel for rebuild notifications

        let watcher = Arc::new(Mutex::new(None)); //FileWatcher::new()?;
        let rebuilder = Arc::new(Rebuilder::new(root_path.clone(), building_type)?);

        Ok(Self {
            port,
            ws_port,
            watcher,
            rebuilder,
            root_path,
            rebuild_tx,
        })
    }

    fn setup_watcher(&self) -> Result<()> {
        let mut watcher = FileWatcher::new()?;

        let src_dir = current_dir()?.join("src");
        let static_dir = current_dir()?.join("static");

        watcher.watch(Path::new(&src_dir))?;
        watcher.watch(Path::new(&static_dir))?;

        // store the watcher in the option, by modifing it with a lock on the mutex
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
                    Ok(rebuild_type) => {
                        // Notify the server about what needs rebuilding
                        if let Err(err) = rebuild_tx.send(rebuild_type) {
                            error!("Error sending rebuild notification: {}", err);
                        }
                    }
                    Err(err) => {
                        error!("Error handling file change: {}", err)
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
                let ansi_regex = regex::Regex::new(r"\\u\{1b\}\[[0-9;]*m").unwrap();
                while let Ok(rebuild_type) = rebuild_rx.recv().await {
                    if let Err(e) = rebuilder
                        .clone()
                        // .expect("Rebuild failed")
                        .rebuild(rebuild_type)
                    {
                        let err_msg = e.to_string();
                        let clean_log_msg = ansi_regex.replace_all(&err_msg, "");
                        error!("Rebuild failed: {}", clean_log_msg);
                    }
                }
            }
        });

        let server_configs = ServerConfigs {
            port: self.port,
            ws_port: self.ws_port,
            _enable_http_logging: true,
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
        let _metacall = initialize().unwrap();

        self.setup_watcher()?;

        let current = current_dir()?;
        debug!("Current directory: {:?}", current);

        let cache_dir = current.join("dist/cache/pages");
        debug!("Checking cache directory: {:?}", cache_dir);

        // perform an initial build pass to catch any pre existing errors
        let out_dir = self.rebuilder.out_dir().to_string_lossy().to_string();

        // client build
        if let Err(e) = ClientBuilder::new("", &out_dir)?.build() {
            let bundling_err = metassr_bundler::BUNDLING_ERROR.lock().unwrap().clone();
            let err_msg = if let Some(bundling_msg) = bundling_err {
                format!("Client-side build failed: {}", bundling_msg)
            } else {
                format!("Client build failed: {}", e)
            };
            // Clean up the error message for the terminal
            let ansi_regex = Regex::new(r"\\u\{1b\}\[[0-9;]*m").unwrap();
            let clean_log_msg = ansi_regex.replace_all(&err_msg, "");

            error!(
                "Initial build failed, caching error for overlay: {}",
                clean_log_msg
            );
            *self.rebuilder.last_errors.lock().unwrap() = Some(vec![err_msg]);
        } else {
            // server build
            let stype = self.rebuilder.building_type();
            if let Err(e) = ServerSideBuilder::new("", &out_dir, stype)?.build() {
                let err_msg = format!("Server build failed: {}", e);
                let ansi_regex = Regex::new(r"\\u\{1b\}\[[0-9;]*m").unwrap();
                let clean_log_msg = ansi_regex.replace_all(&err_msg, "");
                error!(
                    "Initial build failed, caching error for overlay: {}",
                    clean_log_msg
                );
                *self.rebuilder.last_errors.lock().unwrap() = Some(vec![err_msg]);
            }
        }

        self.handle_file_changes().await?;

        self.start_server().await?;

        Ok(())
    }
}
