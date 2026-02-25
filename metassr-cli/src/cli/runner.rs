use anyhow::Result;
use metacall::initialize;
use metassr_server::{RunningType, Server, ServerConfigs};
use std::env::current_dir;
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
        let _metacall = initialize().unwrap();
        let running_type = match self.is_served {
            true => RunningType::StaticSiteGeneration,
            false => RunningType::ServerSideRendering,
        };

        let server_configs = ServerConfigs {
            port: self.port,
            ws_port: 0, // not used in production mode
            _enable_http_logging: self.allow_http_debug,
            root_path: current_dir()?,
            running_type,
            mode: metassr_server::ServerMode::Production,
            rebuilder: None,
        };

        info!(
            message = "Running your web application",
            mode = running_type.to_string()
        );

        Server::new(server_configs).run().await?;
        Ok(())
    }
}
