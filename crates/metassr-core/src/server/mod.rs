mod layers;
use layers::tracing::{LayerSetup, TracingLayer, TracingLayerOptions};

use anyhow::Result;
use axum::{response::Html, routing::get, Router};
use tracing::info;

pub struct ServerConfigs {
    pub port: u16,
    pub _enable_http_logging: bool,
}

pub struct Server {
    configs: ServerConfigs,
    app: Router,
}

impl Server {
    pub fn new(configs: ServerConfigs) -> Self {
        Self {
            configs,
            app: Router::new(),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.configs.port))
            .await
            .unwrap();

        info!("listening on http://{}", listener.local_addr().unwrap());
        let mut app = self.app.clone().route("/", get(root));

        // **Setting up layers**

        // Tracing layer
        TracingLayer::setup(
            TracingLayerOptions {
                enable_http_logging: self.configs._enable_http_logging,
            },
            &mut app,
        );

        axum::serve(listener, app).await.unwrap();
        Ok(())
    }
}

async fn root() -> Html<String> {
    Html("<div>Hello world</div>".to_owned())
}
