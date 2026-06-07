use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

use anyhow::{self, Result};

use crate::cli::traits::Exec;
use crate::cli::{Builder, BuildingType};
use metassr_build::server::BuildingType as ServerBuildingType;
use metassr_server::rebuilder::{RebuildType, Rebuilder};
use metassr_server::{RunningType, Server, ServerConfigs};
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

        // There is a difference between BuildingType in CLI and Server crates. I remember trying to
        // make them shared but i failed for some reason. The current pattern matching is for me to
        // be able to pass building_type to the Server crate. This is not the best solution and sure needs to be improved later
        let building_type: ServerBuildingType = match build_type {
            BuildingType::Ssr => ServerBuildingType::ServerSideRendering,
            BuildingType::Ssg => ServerBuildingType::StaticSiteGeneration,
        };

        let watcher = Arc::new(Mutex::new(None)); //FileWatcher::new()?;
        let rebuilder = Arc::new(Rebuilder::new(root_path.clone(), building_type)?);

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
                    if let Err(e) = rebuilder
                        .clone()
                        // .expect("Rebuild failed")
                        .rebuild(rebuild_type)
                    {
                        error!("Rebuild failed: {}", e);
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

        self.handle_file_changes().await?;

        self.start_server().await?;

        Ok(())
    }
}
