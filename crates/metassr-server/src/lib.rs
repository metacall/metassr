mod fallback;
mod handler;
mod layers;
mod router;

use fallback::Fallback;
use handler::PagesHandler;
use layers::tracing::{LayerSetup, TracingLayer, TracingLayerOptions};

use anyhow::Result;
use axum::{http::StatusCode, response::Redirect, Router};
use router::RouterMut;
use std::path::{Path, PathBuf};
use tower_http::services::ServeDir;
use tracing::info;

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
}

pub struct Server {
    configs: ServerConfigs,
}

impl Server {
    pub fn new(configs: ServerConfigs) -> Self {
        Self { configs }
    }

    pub async fn run(&self) -> Result<()> {
        let listener =
            tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.configs.port)).await?;

        let static_dir = format!("{}/static", self.configs.root_path.to_str().unwrap());
        let dist_dir = format!("{}/dist", self.configs.root_path.to_str().unwrap());
        let notfound_page = Box::new(format!(
            "{}/dist/pages/_notfound/index.html",
            self.configs.root_path.to_str().unwrap()
        ));

        let mut app = RouterMut::from(
            Router::new()
                .nest_service("/static", ServeDir::new(&static_dir))
                .nest_service("/dist", ServeDir::new(&dist_dir)),
        );

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
                app.fallback(fallback)
            }
            RunningType::SSR => app.fallback(|| async { Redirect::to("/_notfound") }),
        }

        PagesHandler::new(&mut app, &dist_dir, self.configs.running_type)?.build()?;

        // **Setting up layers**

        // Tracing layer
        TracingLayer::setup(
            TracingLayerOptions {
                enable_http_logging: self.configs._enable_http_logging,
            },
            &mut app,
        );

        info!("Listening on http://{}", listener.local_addr()?);
        axum::serve(listener, app.app()).await?;
        Ok(())
    }
}
