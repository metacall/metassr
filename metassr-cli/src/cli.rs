use clap::{command, Parser};
use std::fmt::Display;
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
    #[arg(long, default_value_t = 8080)]
    pub port: u16,
}
