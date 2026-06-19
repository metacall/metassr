mod fallback;
mod handler;
mod layers;
pub mod live_reload;
mod router;

use fallback::Fallback;
use handler::PagesHandler;
use layers::tracing::{LayerSetup, TracingLayer, TracingLayerOptions};

use anyhow::Result;
use axum::routing::get;
use axum::{http::StatusCode, response::Redirect, Router};
use live_reload::LiveReloadServer;
use metassr_build::rebuilder::Rebuilder;
use router::RouterMut;
use std::{
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::{debug, info, warn};

use crate::live_reload::inject_live_reload_script;

#[derive(Debug, Clone, Copy)]
pub enum ServerMode {
    Development,
    Production,
}

impl std::fmt::Display for ServerMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Development => write!(f, "development"),
            Self::Production => write!(f, "production"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RunningType {
    StaticSiteGeneration,
    ServerSideRendering,
}

impl std::fmt::Display for RunningType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StaticSiteGeneration => write!(f, "SSG"),
            Self::ServerSideRendering => write!(f, "SSR"),
        }
    }
}

pub struct ServerConfigs {
    pub port: u16,
    pub ws_port: u16,
    pub _enable_http_logging: bool,
    pub root_path: PathBuf,
    pub running_type: RunningType,
    pub mode: ServerMode,
    pub rebuilder: Option<Arc<Rebuilder>>,
}

pub struct Server {
    configs: ServerConfigs,
}

impl Server {
    pub fn new(configs: ServerConfigs) -> Self {
        Self { configs }
    }

    pub async fn run(&self) -> Result<()> {
        let listener = match self.configs.mode {
            ServerMode::Development => bind_http_listener_with_fallback(self.configs.port).await?,
            ServerMode::Production => bind_listener("0.0.0.0", self.configs.port).await?,
        };

        let static_dir = format!("{}/static", self.configs.root_path.to_str().unwrap());
        let dist_dir = format!("{}/dist", self.configs.root_path.to_str().unwrap());
        let notfound_page = Box::new(format!(
            "{}/dist/pages/_notfound/index.html",
            self.configs.root_path.to_str().unwrap()
        ));

        let mut base_router = Router::new()
            .nest_service("/static", ServeDir::new(&static_dir))
            .nest_service("/dist", ServeDir::new(&dist_dir));

        if let ServerMode::Development = self.configs.mode {
            info!("Configuring server for development mode");
            let ws_port = self.configs.ws_port;
            // Inject the ws_port into the live-reload script at runtime
            let live_reload_script =
                include_str!("scripts/live-reload.js").replace("__WS_PORT__", &ws_port.to_string());
            base_router = base_router.route(
                "/livereload/script.js",
                get(move || {
                    let script = live_reload_script.clone();
                    async move {
                        info!("Serving live-reload.js");
                        axum::response::Response::builder()
                            .header("Content-Type", "application/javascript")
                            .body(script)
                            .unwrap()
                    }
                }),
            );
            // Apply live reload middleware
            base_router = base_router.layer(axum::middleware::from_fn(inject_live_reload_script));

            // Start the WebSocket server for live reload
            let ws_listener = bind_http_listener_with_fallback(ws_port)
                .await
                .map_err(|e| anyhow::anyhow!("WebSocket bind error: {}", e))?;
            info!(
                "WebSocket server listening on {:?}",
                ws_listener.local_addr()?
            );
            if let Some(rebuilder) = self.configs.rebuilder.clone() {
                tokio::spawn(async move {
                    while let Ok((stream, _)) = ws_listener.accept().await {
                        let live_reload = LiveReloadServer::new(rebuilder.subscribe());
                        // live_reload.handle_connection(socket).await;
                        tokio::spawn(live_reload.handle_connection(stream));
                    }
                });
            }
        }

        let mut app = RouterMut::from(base_router);

        match self.configs.running_type {
            RunningType::StaticSiteGeneration => {
                let fallback = move || async {
                    (
                        StatusCode::NOT_FOUND,
                        match Path::new(&*notfound_page).exists() {
                            true => Fallback::from_file(PathBuf::from(*notfound_page)).unwrap(),
                            false => Fallback::default(),
                        }
                        .to_html(),
                    )
                };
                app.fallback(fallback);
            }
            RunningType::ServerSideRendering => {
                app.fallback(|| async { Redirect::to("/_notfound") })
            }
        }

        // Register API routes from ./src/api/ directory
        // This scans for .js files and registers GET/POST handlers
        let src_path = self.configs.root_path.join("src");
        if src_path.join("api").exists() {
            match metassr_api_handler::register_api_routes(app.app(), &self.configs.root_path).await
            {
                Ok((router_with_api, Some(api_routes))) => {
                    app = RouterMut::from(router_with_api);
                    if let Some(rebuilder) = &self.configs.rebuilder {
                        rebuilder.set_api_routes(api_routes);
                    }
                    info!("API routes registered successfully");
                }
                Ok((router_with_api, None)) => {
                    app = RouterMut::from(router_with_api);
                    info!("API routes registered (no route files found)");
                }
                Err(e) => {
                    tracing::warn!("Failed to register API routes: {}", e);
                }
            }
        }

        PagesHandler::new(&mut app, &dist_dir, self.configs.running_type)?.build()?;

        // Apply middleware again after PagesHandler to catch dynamic HTML
        if let ServerMode::Development = self.configs.mode {
            debug!("Applying live reload middleware after PagesHandler");
            app = RouterMut::from(
                app.app()
                    .layer(axum::middleware::from_fn(inject_live_reload_script)),
            );
        }

        TracingLayer::setup(
            TracingLayerOptions {
                enable_http_logging: self.configs._enable_http_logging,
                // mode: self.configs.mode,
            },
            &mut app,
        );

        info!(
            message = format!("Listening on http://{}", listener.local_addr()?),
            mode = self.configs.mode.to_string()
        );

        axum::serve(listener, app.app()).await?;
        Ok(())
    }
}

async fn bind_listener(host: &str, port: u16) -> std::result::Result<TcpListener, Error> {
    TcpListener::bind((host, port)).await
}

async fn bind_http_listener_with_fallback(start_port: u16) -> Result<TcpListener> {
    let mut port = start_port;

    loop {
        match bind_listener("0.0.0.0", port).await {
            Ok(listener) => return Ok(listener),
            Err(error) if error.kind() == ErrorKind::AddrInUse => {
                let next_port = port.checked_add(1).ok_or_else(|| {
                    anyhow::anyhow!(
                        "HTTP port {port} is already in use and no higher port is available"
                    )
                })?;

                warn!("HTTP port {port} is already in use; trying port {next_port}");
                port = next_port;
            }
            Err(error) => return Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{bind_http_listener_with_fallback, bind_listener};
    use std::io::ErrorKind;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn binds_to_requested_port_when_available() {
        let listener = bind_http_listener_with_fallback(0).await.unwrap();

        assert_ne!(listener.local_addr().unwrap().port(), 0);
    }

    #[tokio::test]
    async fn increments_port_when_requested_port_is_in_use() {
        let occupied_listener = TcpListener::bind(("0.0.0.0", 0)).await.unwrap();
        let occupied_port = occupied_listener.local_addr().unwrap().port();

        if occupied_port == u16::MAX {
            return;
        }

        let listener = bind_http_listener_with_fallback(occupied_port)
            .await
            .unwrap();

        assert!(listener.local_addr().unwrap().port() > occupied_port);
    }

    #[tokio::test]
    async fn strict_bind_returns_addr_in_use_when_port_is_occupied() {
        let occupied_listener = TcpListener::bind(("0.0.0.0", 0)).await.unwrap();
        let occupied_port = occupied_listener.local_addr().unwrap().port();

        let error = bind_listener("0.0.0.0", occupied_port).await.unwrap_err();

        assert_eq!(error.kind(), ErrorKind::AddrInUse);
    }
}
