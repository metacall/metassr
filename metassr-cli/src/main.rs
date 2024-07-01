mod cli;
use clap::Parser;
use cli::Args;
use logger::LoggingLayer;

use metacall::{loaders, metacall, switch};
use metassr_core::server::{Server, ServerConfigs};
use std::{env::set_current_dir, path::Path};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

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

    Server::new(ServerConfigs {
        port: 8080,
        _enable_http_logging: true,
    })
    .run()
    .await
    .unwrap();
}
