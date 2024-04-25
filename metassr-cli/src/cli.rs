use clap::{command, Parser};
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
    /// the port where HTTP server running
    #[arg(long, default_value_t = 8080)]
    pub port: u16,
    /// Run with debug mode (add more information with logs)
    #[arg(long)]
    pub debug_mode: bool,

    /// The path where your logs want be saved
    #[arg[long]]
    pub log_file: Option<String>,
    /// Show HTTP logs (requests & responses)
    #[arg(long)]
    pub enable_http_logging: bool,
}
