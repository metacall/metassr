mod cli;
mod trace;
use anyhow::Result;
use clap::Parser;
use cli::Args;
use logger::LoggingLayer;
use metacall::{loaders, metacall, switch};
use std::{env::set_current_dir, path::Path};

use axum::{response::Html, routing::get, Router};
use tracing::info;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use crate::trace::http_trace;
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(LoggingLayer {
            logfile: args.log_file,
        })
        .init();

    let project_root = Path::new(&args.root);
    set_current_dir(&project_root)
        .map_err(|err| eprintln!("Cannot chdir: {err}"))
        .unwrap();

    let _metacall = switch::initialize().unwrap();
    loaders::from_single_file("ts", ["src/_app.tsx"].concat()).unwrap();

    server_runner(args.port, args.enable_http_logging)
        .await
        .unwrap();
}
// TODO: refactor server runner
async fn server_runner(port: u16, enable_http_logging: bool) -> Result<()> {
    let app = Router::new().route("/", get(root));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!("listening on http://{}", listener.local_addr().unwrap());

    // TODO: refactorging http tracing
    axum::serve(listener, http_trace(app, enable_http_logging))
        .await
        .unwrap();
    Ok(())
}

async fn root() -> Html<String> {
    Html(
        match metacall::<String>("__App__", ["Metacall".to_string()]) {
            Err(e) => {
                println!("{:?}", e);
                panic!();
            }
            Ok(ret) => ret,
        },
    )
}
