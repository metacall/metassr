mod cli;
use clap::Parser;
use cli::{Args, Commands, DebugMode};
use logger::LoggingLayer;

use anyhow::{anyhow, Result};
use metacall::switch;

use metassr_build::{client::ClientBuilder, server::ServerSideBuilder, traits::Build};
use metassr_server::{Server, ServerConfigs};

use std::{
    env::{current_dir, set_current_dir, set_var},
    path::Path,
    time::Instant,
};

use tracing::{error, info};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let allow_metacall_debug =
        [Some(DebugMode::All), Some(DebugMode::Metacall)].contains(&args.debug_mode);
    let allow_http_debug = [Some(DebugMode::All), Some(DebugMode::Http)].contains(&args.debug_mode);

    tracing_subscriber::registry()
        .with(LoggingLayer {
            logfile: args.log_file,
        })
        .init();

    let project_root = Path::new(&args.root);

    set_current_dir(project_root)
        .map_err(|err| eprintln!("Cannot chdir: {err}"))
        .unwrap();
    if allow_metacall_debug {
        set_var("METACALL_DEBUG", "1");
    }

    if allow_metacall_debug {
        set_var("METACALL_DEBUG", "1");
    }

    match args.commands {
        Some(Commands::Build { out_dir }) => {
            let instant = Instant::now();
            let _metacall = switch::initialize().unwrap();

            if let Err(e) = ClientBuilder::new("", &out_dir)?.build() {
                error!(
                    target = "builder",
                    message = format!("Couldn't build for the client side:  {e}"),
                );
                return Err(anyhow!("Couldn't continue building process."));
            };

            if let Err(e) = ServerSideBuilder::new("", &out_dir)?.build() {
                error!(
                    target = "builder",
                    message = format!("Couldn't build for the server side: {e}"),
                );
                return Err(anyhow!("Couldn't continue building process."));
            }

            // Waiting till metacall destroyed
            if (_metacall.0)() == 0 {
                info!(
                    target = "builder",
                    message = "Building is completed",
                    time = format!("{}ms", instant.elapsed().as_millis())
                );
            }
        }
        Some(Commands::Run { port }) => {
            let server_configs = ServerConfigs {
                port,
                _enable_http_logging: allow_http_debug,
                root_path: current_dir()?,
            };

            Server::new(server_configs).run().await?;
        }
        _ => {}
    };

    Ok(())
}
