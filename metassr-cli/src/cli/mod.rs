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
Command line interface application for MetaSSR framework. This CLI tool helps you manage and deploy your MetaSSR projects.
"
)]
pub struct Args {
    /// The path of the project root directory.
    #[arg(long)]
    pub root: String,

    /// Enable debug mode to provide more detailed logs.
    #[arg(long)]
    pub debug_mode: Option<DebugMode>,

    /// Specify the file path where logs will be saved. If not provided, logs will be shown in the console only.
    #[arg(long)]
    pub log_file: Option<String>,

    #[command(subcommand)]
    pub commands: Option<Commands>,
}

#[derive(Debug, ValueEnum, PartialEq, Eq, Clone)]
pub enum DebugMode {
    /// Enables all available debug logs.
    All,
    /// Logs related specifically to MetaCall operations.
    Metacall,
    /// Logs HTTP request and response details.
    Http,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum Commands {
    /// Builds your web application into a deployable format.
    Build {
        /// The output directory where build files will be saved.
        #[arg(long)]
        out_dir: String,

        /// The type of build to perform. Choose between SSR (Server-Side Rendering) and SSG (Static Site Generation).
        #[arg(short = 't', long = "type")]
        build_type: BuildingType,
    },

    /// Runs the Server-Side Rendered (SSR) application.
    Run {
        /// The port number on which the HTTP server will run.
        #[arg(long)]
        port: u16,

        /// Serve the generated static site directly.
        #[arg(long)]
        serve: bool,
    },

    /// Creates a new MetaSSR project with the specified template.
    Create {
        /// The name of the new project. This is a required argument.
        #[arg(index = 1)]
        project_name: String,

        /// The version of your web application.
        #[arg(long, short)]
        version: String,

        /// A brief description of your web application.
        #[arg(long, short)]
        description: String,

        /// The template to use for creating the new project.
        #[arg(long, short)]
        template: Template,
    },
}
