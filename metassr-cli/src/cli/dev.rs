use std::env::current_dir;
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use metacall::switch;
use metassr_server::{RunningType, Server, ServerConfigs};
use metassr_watcher::FileWatcher;

use tracing::info;

use super::traits::AsyncExec;

pub struct Dev {
    port: u16,
    watcher: Arc<Mutex<Option<FileWatcher>>>,
}

impl Dev {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            watcher: Arc::new(Mutex::new(None)),
        }
    }

    fn setup_watcher(&self) -> Result<()> {
        let mut watcher = FileWatcher::new()?;

        let src_dir = current_dir()?.join("src");

        watcher.watch(Path::new(&src_dir))?;

        // store the watcher in the option, by modifing it with a lock on the mutex
        let mut watcher_guard = self.watcher.lock().unwrap();
        *watcher_guard = Some(watcher);
        Ok(())
    }
}

impl AsyncExec for Dev {
    async fn exec(&self) -> Result<()> {
        let _metacall = switch::initialize().unwrap();

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

        info!("Running your web application on dev mode",);

        let server_configs = ServerConfigs {
            port: self.port,
            _enable_http_logging: true,
            root_path: current_dir()?,
            running_type: RunningType::SSR,
        };

        self.setup_watcher()?;

        if let Some(watcher) = &*self.watcher.lock().unwrap() {
            let mut rx = watcher.subscribe();

            tokio::spawn(async move {
                while let Ok(event) = rx.recv().await {
                    // info!("Detected change: {:?}\n", event);
                }
            });
        }

        // info!("Starting server with config: {:?}", server_configs);
        Server::new(server_configs).run().await?;
        Ok(())
    }
}
