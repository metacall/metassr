mod cli;

use anyhow::Result;
use axum::{response::Html, routing::get, Router};
use clap::Parser;
use cli::Args;
use metacall::{loaders, metacall, switch};
use std::{env::set_current_dir, path::Path};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let project_root = Path::new(&args.root);
    set_current_dir(&project_root)
        .map_err(|err| eprintln!("Cannot chdir: {err}"))
        .unwrap();

    let _metacall = switch::initialize().unwrap();
    loaders::from_single_file("ts", ["App.tsx"].concat()).unwrap();

    server_runner(args.port).await.unwrap();
}

async fn server_runner(port: u16) -> Result<()> {
    let app = Router::new().route("/", get(root));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    println!(
        "listening on http://{}",
        listener.local_addr().unwrap().to_string()
    );

    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn root() -> Html<String> {
    Html(
        match metacall::<String>("Hello", ["Metacall!".to_string()]) {
            Err(e) => {
                println!("{:?}", e);
                panic!();
            }
            Ok(ret) => match ret {
                message => message,
                _ => "<h1>Not a Valid HTML</h1>".to_string(),
            },
        },
    )
}
