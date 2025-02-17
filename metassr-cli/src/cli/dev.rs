use std::env::current_dir;
use std::path::Path;

use anyhow::Result;

// use metacall::switch;
use metassr_server::{watcher::FileWatcher, RunningType, Server, ServerConfigs};
use tracing::info;

use super::traits::AsyncExec;

pub struct Dev {
    port: u16,
    watcher: Option<FileWatcher>,
}

impl Dev {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            watcher: None,
        }
    }

    fn setup_watcher(&mut self) -> Result<()> {
        let mut watcher = FileWatcher::new()?;

        let src_dir = current_dir()?.join("src");

        watcher.watch(Path::new(&src_dir))?;

        // store the watcher in the option
        self.watcher = Some(watcher);
        Ok(())
    }
}

impl AsyncExec for Dev {
    async fn exec(&self) -> Result<()> {
        // let _metacall = switch::initialize().unwrap();

        info!(
            "Running your web application on {:?} mode",
            RunningType::SSR
        );

        let server_configs = ServerConfigs {
            port: self.port,
            _enable_http_logging: true,
            root_path: current_dir()?,
            running_type: RunningType::SSR,
        };

        // self.setup_watcher()?;

        Server::new(server_configs).run().await?;
        Ok(())
    }
}
