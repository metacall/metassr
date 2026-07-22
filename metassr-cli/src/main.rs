mod cli;
use clap::Parser;
use cli::{
    traits::{AsyncExec, Exec},
    Args, BuildingType, Commands, DebugMode,
};
use logger::LoggingLayer;

use anyhow::Result;
use metassr_config::{BuildConfig, MetaSSRConfig};

use std::{
    env::{current_dir, set_current_dir, set_var},
    path::Path,
};

use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_DEV_WS_PORT: u16 = 3001;
const DEFAULT_OUT_DIR: &str = "dist";

fn debug_mode_from_config(config: &Option<MetaSSRConfig>) -> Option<DebugMode> {
    let debug = config.as_ref()?.debug.as_ref()?;
    match debug.mode.as_deref()? {
        "all" => Some(DebugMode::All),
        "metacall" => Some(DebugMode::Metacall),
        "http" => Some(DebugMode::Http),
        _ => None,
    }
}

pub fn build_filter_string(debug_mode: Option<DebugMode>) -> String {
    match debug_mode {
        Some(DebugMode::All) => "debug",
        Some(DebugMode::Http) => "http=debug,info",
        _ => "info",
    }
    .to_string()
}

fn resolve_build(config: &Option<MetaSSRConfig>) -> (String, BuildingType) {
    let build: Option<&BuildConfig> = config.as_ref().and_then(|c| c.build.as_ref());

    let out_dir = build
        .and_then(|b| b.out_dir.as_deref())
        .unwrap_or(DEFAULT_OUT_DIR)
        .to_string();

    let build_type = build
        .and_then(|b| b.r#type.as_deref())
        .map(|t| match t {
            "ssg" => cli::BuildingType::Ssg,
            _ => cli::BuildingType::Ssr,
        })
        .unwrap_or(cli::BuildingType::Ssr);

    (out_dir, build_type)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let is_create = matches!(args.commands, Commands::Create { .. });

    let config = if !is_create {
        MetaSSRConfig::load(Path::new(&args.root))?
    } else {
        None
    };

    let debug_mode = args.debug_mode.or_else(|| debug_mode_from_config(&config));

    let allow_metacall_debug = matches!(debug_mode, Some(DebugMode::All | DebugMode::Metacall));
    let allow_http_debug = matches!(debug_mode, Some(DebugMode::All | DebugMode::Http));

    let log_file = args.log_file.or_else(|| {
        config
            .as_ref()
            .and_then(|c| c.debug.as_ref()?.log_file.clone())
    });

    let tracing_level = build_filter_string(debug_mode);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(tracing_level).add_directive("notify=off".parse().unwrap())
    });

    if is_create {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .without_time()
            .compact()
            .init();

        if let Commands::Create {
            project_name,
            version,
            description,
            template,
        } = args.commands
        {
            cli::Creator::new(project_name, version, description, template)?.exec()?;
        }
        return Ok(());
    }

    tracing_subscriber::registry()
        .with(filter)
        .with(LoggingLayer { logfile: log_file })
        .init();

    let project_root = Path::new(&args.root);
    set_current_dir(project_root)
        .map_err(|err| eprintln!("Cannot chdir: {err}"))
        .unwrap();

    if allow_metacall_debug {
        set_var("METACALL_DEBUG", "1");
    }

    match args.commands {
        Commands::Build {
            out_dir,
            build_type,
        } => {
            let (config_out_dir, config_build_type) = resolve_build(&config);
            let out_dir = out_dir.unwrap_or(config_out_dir);
            let build_type = build_type.unwrap_or(config_build_type);

            tracing::info!("command build Out dir: {:?}", out_dir);

            cli::Builder::new(build_type, out_dir).exec()?;
        }
        Commands::Start { port, serve } => {
            let port = port
                .or_else(|| config.as_ref().and_then(|c| c.server.as_ref()?.port))
                .unwrap_or(DEFAULT_PORT);

            cli::Runner::new(port, serve, allow_http_debug)
                .exec()
                .await?;
        }
        Commands::Dev {
            port,
            ws_port,
            out_dir,
            build_type,
        } => {
            let (config_out_dir, config_build_type) = resolve_build(&config);
            let port = port
                .or_else(|| config.as_ref().and_then(|c| c.dev.as_ref()?.server_port))
                .unwrap_or(DEFAULT_PORT);
            let ws_port = ws_port
                .or_else(|| config.as_ref().and_then(|c| c.dev.as_ref()?.ws_port))
                .unwrap_or(DEFAULT_DEV_WS_PORT);
            let out_dir = out_dir.unwrap_or(config_out_dir);
            let build_type = build_type.unwrap_or(config_build_type);

            cli::Dev::new(
                port,
                ws_port,
                current_dir()?,
                out_dir,
                build_type,
                allow_http_debug,
            )?
            .exec()
            .await?;
        }
        _ => {} // Create is handled above
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
