use std::env::current_dir;

use anyhow::Result;
use metassr_server::{RunningType, Server, ServerConfigs};
use tracing::info;

use super::traits::AsyncExec;

pub struct Runner {
    port: u16,
    is_served: bool,
    allow_http_debug: bool,
}

impl Runner {
    pub fn new(port: u16, is_served: bool, allow_http_debug: bool) -> Self {
        Self {
            port,
            is_served,
            allow_http_debug,
        }
    }
}
impl AsyncExec for Runner {
    async fn exec(&self) -> Result<()> {
        let running_type = match self.is_served {
            true => RunningType::SSG,
            false => RunningType::SSR,
        };

        let server_configs = ServerConfigs {
            port: self.port,
            _enable_http_logging: self.allow_http_debug,
            root_path: current_dir()?,
            running_type,
        };

        info!("Running your web application on {:?} mode", running_type);

        Server::new(server_configs).run().await?;
        Ok(())
    }
}
