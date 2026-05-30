mod cli;
use clap::Parser;
use cli::{
    traits::{AsyncExec, Exec},
    Args, Commands, DebugMode,
};
use logger::LoggingLayer;

use anyhow::Result;

use std::{
    env::{current_dir, set_current_dir, set_var},
    path::Path,
};

use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

pub fn build_filter_string(debug_mode: Option<DebugMode>) -> String {
    let level = match debug_mode {
        Some(DebugMode::All) => "debug",
        Some(DebugMode::Http) => "http=debug,info",
        _ => "info",
    };
    level.to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let allow_metacall_debug =
        [Some(DebugMode::All), Some(DebugMode::Metacall)].contains(&args.debug_mode);
    let allow_http_debug = [Some(DebugMode::All), Some(DebugMode::Http)].contains(&args.debug_mode);

    let tracing_level = build_filter_string(args.debug_mode);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(&tracing_level).add_directive("notify=off".parse().unwrap())
    });
    if let Commands::Create { .. } = args.commands {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .without_time()
            .compact()
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
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
    }
    match args.commands {
        Commands::Build {
            out_dir,
            build_type,
        } => {
            tracing::info!("command build Out dir: {:?}", out_dir);

            cli::Builder::new(build_type, out_dir).exec()?;
        }
        Commands::Run { port, serve } => {
            cli::Runner::new(port, serve, allow_http_debug)
                .exec()
                .await?;
        }
        Commands::Create {
            project_name,
            version,
            description,
            template,
        } => {
            cli::Creator::new(project_name, version, description, template)?.exec()?;
        }
        Commands::Dev { port, ws_port } => {
            cli::Dev::new(
                port,
                ws_port,
                current_dir()?,
                metassr_build::server::BuildingType::ServerSideRendering,
                allow_http_debug,
            )?
            .exec()
            .await?;
        }
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_flags_defaults_to_info() {
        assert_eq!(build_filter_string(None), "info");
    }

    #[test]
    fn http_flag_sets_http_debug() {
        assert_eq!(
            build_filter_string(Some(DebugMode::Http)),
            "http=debug,info"
        );
    }

    #[test]
    fn all_flag_sets_global_debug() {
        assert_eq!(build_filter_string(Some(DebugMode::All)), "debug");
    }

    #[test]
    fn rust_log_env_overrides_cli() {
        std::env::set_var("RUST_LOG", "debug");
        let result = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(build_filter_string(Some(DebugMode::Http))));
        std::env::remove_var("RUST_LOG");
        // assert that RUST_LOG overrides the cli flag
        assert_eq!(format!("{result}"), "debug");
    }
}
