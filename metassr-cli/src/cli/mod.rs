mod builder;
mod creator;
mod runner;
pub mod traits;

pub use builder::*;
pub use creator::*;
pub use runner::*;

use clap::{command, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "
Command line interface application for MetaSSR framework
"
)]

pub struct Args {
    /// The path of project root
    #[arg(long, default_value_t = String::from("."))]
    pub root: String,
    /// Run with debug mode (add more information with logs)
    #[arg(long)]
    pub debug_mode: Option<DebugMode>,

    /// The path where your logs want be saved
    #[arg[long]]
    pub log_file: Option<String>,
    /// Show HTTP logs (requests & responses)
    #[arg(long)]
    pub enable_http_logging: bool,

    #[command(subcommand)]
    pub commands: Option<Commands>,
}

#[derive(Debug, ValueEnum, PartialEq, Eq, Clone)]
pub enum DebugMode {
    All,
    Metacall,
    Http,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum Commands {
    /// Building your web application.
    Build {
        #[arg(long, default_value_t = String::from("dist"))]
        out_dir: String,
        #[arg(short = 't', long = "type", default_value_t = BuildingType::SSR)]
        build_type: BuildingType,
    },
    /// Run your Server-side rendered web application.
    Run {
        /// the port where HTTP server running.
        #[arg(long, default_value_t = 8080)]
        port: u16,
        /// Serve your generated static-site
        #[arg(long)]
        serve: bool,
    },

    /// Create a new metassr project.
    Create {
        /// The name of project
        #[arg(index = 1)]
        project_name: String,

        /// The version of your web application
        #[arg(long, short, default_value_t = String::from("1.0.0"))]
        version: String,

        /// The description of your web application
        #[arg(long, short, default_value_t = String::from("A web application built with MetaSSR framework."))]
        description: String,

        /// Template of your new project
        #[arg(long, short, default_value_t = Template::Javascript)]
        template: Template,
    },
}
