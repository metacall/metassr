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
    // todo change this to a normal option, and edit impl asyncexec
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
                        if let Err(e) = rebuild_tx.send(rebuild_type) {
                            error!("Error sending rebuild notification: {}", e);
                        }
                    }
                    Err(e) => error!("Error handling file change: {}", e),
                }
            }
        });

        Ok(())
    }

    async fn start_server(&self) -> Result<()> {
        let server_configs: ServerConfigs = ServerConfigs {
            port: self.port,
            _enable_http_logging: true,
            root_path: self.root_path.clone(),
            running_type: RunningType::SSR,
            mode: metassr_server::ServerMode::Development,
        };
        println!("{:?}", self.root_path);
        let mut rebuild_rx: broadcast::Receiver<RebuildType> = self.rebuild_tx.subscribe();

        let rebuilder: Option<Arc<Rebuilder>> = Some(self.rebuilder.clone());
        let rebuilder_clone: Option<Arc<Rebuilder>> = Some(self.rebuilder.clone());

        tokio::spawn(async move {
            while let Ok(rebuild_type) = rebuild_rx.recv().await {
                if let Err(e) = rebuilder_clone
                    .clone()
                    .expect("Rebuild failed")
                    .rebuild(rebuild_type)
                    .await
                {
                    error!("Rebuild failed: {}", e);
                }
            }
        });

        Server::new(server_configs).run(rebuilder).await?;
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

        self.handle_file_changes().await?;

        self.start_server().await?;

        info!("Running your web application on dev mode",);

        Ok(())
    }
}
