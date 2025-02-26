// crates/metassr-server/src/lib.rs
mod fallback;
mod handler;
mod layers;
pub mod live_reload;
pub mod rebuilder;
mod router;

use fallback::Fallback;
use handler::PagesHandler;
use layers::tracing::{LayerSetup, TracingLayer, TracingLayerOptions};

use anyhow::Result;
use axum::{http::StatusCode, response::Redirect, Router};
use axum::{response::IntoResponse, routing::get};
use live_reload::LiveReloadServer;
use rebuilder::Rebuilder;
use router::RouterMut;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;
use tracing::info;

#[derive(Debug, Clone, Copy)]
pub enum ServerMode {
    Development,
    Production,
}

#[derive(Debug, Clone, Copy)]
pub enum RunningType {
    SSG,
    SSR,
}

pub struct ServerConfigs {
    pub port: u16,
    pub _enable_http_logging: bool,
    pub root_path: PathBuf,
    pub running_type: RunningType,
    pub mode: ServerMode,
}

pub struct Server {
    configs: ServerConfigs,
}

impl Server {
    pub fn new(configs: ServerConfigs) -> Self {
        Self { configs }
    }

    pub async fn run(&self, rebuilder: Option<Arc<Rebuilder>>) -> Result<()> {
        let listener =
            tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.configs.port)).await?;

        let static_dir = format!("{}/static", self.configs.root_path.to_str().unwrap());
        let dist_dir = format!("{}/dist", self.configs.root_path.to_str().unwrap());
        let notfound_page = Box::new(format!(
            "{}/dist/pages/_notfound/index.html",
            self.configs.root_path.to_str().unwrap()
        ));

        let mut base_router: Router<()> = Router::new()
            .nest_service("/static", ServeDir::new(&static_dir))
            .nest_service("/dist", ServeDir::new(&dist_dir));

        if let ServerMode::Development = self.configs.mode {
            let live_reload_script = include_str!("scripts/live-reload.js");
            base_router = base_router.route(
                "/livereload/script.js", //todo check this path
                get(|| async {
                    axum::response::Response::builder()
                        .header("Content-Type", "application/javascript")
                        .body(live_reload_script.to_string())
                        .unwrap()
                }),
            );
            // Start the WebSocket server for live reload
            let ws_listener = TcpListener::bind("127.0.0.1:3001").await?;
            println!("0000000000000 {:?}", ws_listener);
            if let Some(rebuilder) = rebuilder {
                tokio::spawn(async move {
                    while let Ok((stream, addr)) = ws_listener.accept().await {
                        let live_reload = LiveReloadServer::new(rebuilder.subscribe());
                        tokio::spawn(live_reload.handle_connection(stream, addr));
                    }
                });
            }
        }

        let mut app: RouterMut<()> = RouterMut::from(base_router);

        match self.configs.running_type {
            RunningType::SSG => {
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
            RunningType::SSR => app.fallback(|| async { Redirect::to("/_notfound") }),
        }

        PagesHandler::new(&mut app, &dist_dir, self.configs.running_type)?.build()?;

        TracingLayer::setup(
            TracingLayerOptions {
                enable_http_logging: self.configs._enable_http_logging,
                // mode: self.configs.mode,
            },
            &mut app,
        );

        info!(
            "Listening on http://{} in {} mode",
            listener.local_addr()?,
            match self.configs.mode {
                ServerMode::Development => "development",
                ServerMode::Production => "production",
            }
        );

        axum::serve(listener, app.app()).await?;
        Ok(())
    }
}
