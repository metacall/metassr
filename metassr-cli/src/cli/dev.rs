use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

use anyhow::Result;

use metacall::switch;
use metassr_build::server::BuildingType;
use metassr_server::rebuilder::{RebuildType, Rebuilder};
use metassr_server::{RunningType, Server, ServerConfigs};
use metassr_watcher::FileWatcher;

use tracing::{error, info};

use super::traits::AsyncExec;

pub struct Dev {
    port: u16,
    watcher: Arc<Mutex<Option<FileWatcher>>>,
    rebuilder: Arc<Rebuilder>,
    root_path: PathBuf,
    rebuild_tx: broadcast::Sender<RebuildType>,
}

impl Dev {
    pub fn new(port: u16, root_path: PathBuf, building_type: BuildingType) -> Result<Self> {
        let rebuild_tx: broadcast::Sender<RebuildType> = broadcast::channel(100).0; //channel for rebuild notifications

        let watcher: Arc<Mutex<Option<FileWatcher>>> = Arc::new(Mutex::new(None)); //FileWatcher::new()?;
        let rebuilder: Arc<Rebuilder> = Arc::new(Rebuilder::new(root_path.clone(), building_type)?);

        Ok(Self {
            port,
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

    async fn start_server(&self) -> Result<()> {
        let configs = ServerConfigs {
            port: self.port,
            _enable_http_logging: true,
            root_path: self.root_path.clone(),
            running_type: RunningType::SSR,
        };

        let server = Server::new(configs);
        let mut rebuild_rx: broadcast::Receiver<RebuildType> = self.rebuild_tx.subscribe();

        let rebuilder = self.rebuilder.clone();

        tokio::spawn(async move {
            while let Ok(rebuild_type) = rebuild_rx.recv().await {
                if let Err(e) = rebuilder.rebuild(rebuild_type).await {
                    error!("Rebuild failed: {}", e);
                }
            }
        });

        server.run().await?;

        Ok(())
    }
}

impl AsyncExec for Dev {
    async fn exec(&self) -> Result<()> {
        let _metacall = switch::initialize().unwrap();

        self.setup_watcher()?;

        let current = current_dir()?;
        info!("Current directory: {:?}", current);

        let cache_dir = current.join("dist/cache/pages");
        info!("Checking cache directory: {:?}", cache_dir);

        if let Ok(entries) = std::fs::read_dir(&cache_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    info!("Found file: {:?}", entry.path());
                }
            }
        }

        self.start_server().await?;

        info!("Running your web application on dev mode",);

        Ok(())
    }
}
