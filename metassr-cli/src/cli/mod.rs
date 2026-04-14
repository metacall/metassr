mod builder;
mod creator;
mod dev;
mod runner;
pub mod traits;

pub use builder::*;
pub use creator::*;
pub use dev::*;
pub use runner::*;

use clap::{Parser, Subcommand, ValueEnum};

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
    #[arg(long, default_value_t = String::from("."))]
    pub root: String,

    /// Enable debug mode to provide more detailed logs.
    #[arg(long)]
    pub debug_mode: Option<DebugMode>,

    /// Specify the file path where logs will be saved. If not provided, logs will be shown in the console only.
    #[arg(long)]
    pub log_file: Option<String>,

    #[command(subcommand)]
    pub commands: Commands,
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
        #[arg(long, default_value_t = String::from("dist"))]
        out_dir: String,

        /// The type of build to perform. Choose between Ssr (Server-Side Rendering) and Ssg (Static Site Generation).
        #[arg(short = 't', long = "type", default_value_t = BuildingType::Ssr)]
        build_type: BuildingType,
    },

    /// Runs the Server-Side Rendered (SSR) application.
    Run {
        /// The port number on which the HTTP server will run.
        #[arg(long, default_value_t = 8080)]
        port: u16,

        /// Serve the generated static site directly.
        #[arg(long)]
        serve: bool,
    },

    /// Creates a new MetaSSR project with the specified template.
    Create {
        /// The name of the new project. This is a required argument.
        #[arg(index = 1)]
        project_name: Option<String>,

        /// The version of your web application.
        #[arg(long, short)]
        version: Option<String>,

        /// A brief description of your web application.
        #[arg(long, short)]
        description: Option<String>,

        /// The template to use for creating the new project.
        #[arg(long, short)]
        template: Option<Template>,
    },

    Dev {
        /// port number on which the HTTP server will run
        #[arg(long, default_value_t = 8080)]
        port: u16,

        /// port number for the WebSocket live reload server
        #[arg(long, default_value_t = 3001)]
        ws_port: u16,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Result<Args, clap::Error> {
        let mut full = vec!["metassr"];
        full.extend_from_slice(args);
        Args::try_parse_from(full)
    }

    #[test]
    fn build_defaults() {
        let args = parse(&["build"]).unwrap();
        if let Commands::Build {
            out_dir,
            build_type,
        } = args.commands
        {
            assert_eq!(out_dir, "dist");
            assert_eq!(build_type, BuildingType::Ssr);
        } else {
            panic!("expected Build command");
        }
    }

    #[test]
    fn run_defaults() {
        let args = parse(&["run"]).unwrap();
        if let Commands::Run { port, serve } = args.commands {
            assert_eq!(port, 8080);
            assert!(!serve);
        } else {
            panic!("expected Run command");
        }
    }

    #[test]
    fn dev_defaults() {
        let args = parse(&["dev"]).unwrap();
        if let Commands::Dev { port, ws_port } = args.commands {
            assert_eq!(port, 8080);
            assert_eq!(ws_port, 3001);
        } else {
            panic!("expected Dev command");
        }
    }

    #[test]
    fn create_with_all_flags() {
        let args = parse(&[
            "create",
            "my-app",
            "--version",
            "2.0.0",
            "--description",
            "test project",
            "--template",
            "typescript",
        ])
        .unwrap();

        if let Commands::Create {
            project_name,
            version,
            description,
            template,
        } = args.commands
        {
            assert_eq!(project_name, Some("my-app".into()));
            assert_eq!(version, Some("2.0.0".into()));
            assert_eq!(description, Some("test project".into()));
            assert_eq!(template, Some(Template::Typescript));
        } else {
            panic!("expected Create command");
        }
    }

    #[test]
    fn debug_mode_flag() {
        let args = parse(&["--debug-mode", "all", "build"]).unwrap();
        assert_eq!(args.debug_mode, Some(DebugMode::All));
    }

    #[test]
    fn unknown_subcommand_is_rejected() {
        assert!(parse(&["deploy"]).is_err());
    }
}
