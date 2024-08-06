mod fallback;
mod layers;
use fallback::Fallback;
use layers::tracing::{LayerSetup, TracingLayer, TracingLayerOptions};

use anyhow::Result;
use axum::{http::StatusCode, Router};
use tower_http::services::ServeDir;
use tracing::info;

pub struct ServerConfigs<'a> {
    pub port: u16,
    pub _enable_http_logging: bool,
    pub root_path: &'a Path,
}

pub struct Server<'a> {
    configs: &'a ServerConfigs<'a>,
    app: Router,
}

impl<'a> Server<'a> {
    pub fn new(configs: &'a ServerConfigs) -> Self {
        Self {
            configs,
            app: Router::new(),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let listener =
            tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.configs.port)).await?;

        let notfound_page = Box::new(format!(
            "{}/dist/pages/_notfound/index.html",
            self.configs.root_path.to_str().unwrap()
        ));

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
            .nest_service("/static", ServeDir::new(&static_dir))
            .fallback_service(get(default_not_found));

        // **Setting up layers**

        // Tracing layer
        TracingLayer::setup(
            TracingLayerOptions {
                enable_http_logging: self.configs._enable_http_logging,
            },
            &mut app,
        );

        info!("listening on http://{}", listener.local_addr()?);
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn default_not_found() -> Html<String> {
    Html("<div>Hello world</div>".to_owned())
}
