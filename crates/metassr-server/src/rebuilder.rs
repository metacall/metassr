use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use metacall::initialize;
use metassr_build::{
    client::ClientBuilder,
    server::{BuildingType, ServerSideBuilder},
    traits::{Build, Generate},
};
use metassr_watcher::utils::*;
use notify::Event;
use tokio::sync::broadcast;

use std::time::Instant;

use tracing::{error, info};

#[derive(Clone, Debug)]
pub enum RebuildType {
    Page(PathBuf), // this only is done
    Layout,
    Component,
    Style,
    Static,
}

pub struct Rebuilder {
    sender: broadcast::Sender<RebuildType>,
    root_path: PathBuf,
    out_dir: PathBuf,
    building_type: BuildingType,
}

impl Rebuilder {
    pub fn new(root_path: PathBuf, building_type: BuildingType) -> Result<Self> {
        let (sender, _) = broadcast::channel(100);
        let out_dir = PathBuf::from("dist");
        println!("Out dir: {:?}", out_dir);

        Ok(Self {
            sender,
            root_path,
            out_dir,
            building_type,
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RebuildType> {
        self.sender.subscribe()
    }

    pub fn handle_event(&self, event: Event) -> Result<RebuildType> {
        if !is_relevant_event(&event) {
            anyhow::bail!("Not a relevant event");
        }

        let path = event
            .paths
            .first()
            .ok_or_else(|| anyhow::anyhow!("No path"))?;

        println!("Path: {path:#?}");

        let rel_path: &Path = path.strip_prefix(&self.root_path)?;

        println!("Rel path: {:?}", rel_path);

        let rebuild_type: RebuildType = self.map_path_to_type(rel_path)?;

        // Log what we're entered rebuilding
        info!("Rebuilding due to changes in: {:?}", rebuild_type);

        // Send rebuild notification
        let _ = self.sender.send(rebuild_type.clone()); // ?? validate the borrowing/ownership model here

        Ok(rebuild_type)
    }

    fn map_path_to_type(&self, path: &Path) -> Result<RebuildType> {
        let path_buf = path.to_path_buf();
        let path_str = path.to_string_lossy(); // make path a Cow. not all filenames are valid UTF-8

        let rebuild_type: RebuildType = match path_str {
            path if path.starts_with("src/pages") => RebuildType::Page(path_buf.clone()),
            path if path.starts_with("src/layout") => RebuildType::Layout,
            path if path.starts_with("src/components") => RebuildType::Component,
            path if path.starts_with("src/styles") => RebuildType::Style,
            path if path.starts_with("static") => RebuildType::Static,
            // entered rebuilding everything if we're not surue of entered rebuilding kind
            _ => RebuildType::Layout,
        };

    Ok(rebuild_type)
    }

    pub async fn rebuild(&self, rebuild_type: RebuildType) -> Result<()> {
        match rebuild_type {
            RebuildType::Page(ref path) => {
                info!("entered rebuilding {:?} in {:?}", rebuild_type, path);
                self.rebuild_page(path.clone()).await?;
                let _ = self.sender.send(rebuild_type.clone());
            }
            RebuildType::Layout => {
                // todo
                info!("entered rebuilding {:?}", rebuild_type);
            }
            RebuildType::Component => {
                // todo
                info!("entered rebuilding {:?}", rebuild_type);
            }
            RebuildType::Style => {
                // todo
                info!("entered rebuilding {:?}", rebuild_type);
            }
            RebuildType::Static => {
                // todo
                info!("entered rebuilding {:?}", rebuild_type);
            }
        }

        Ok(())
    }

    async fn rebuild_page(&self, path: PathBuf) -> Result<()> {
        info!("Rebuilding page {:?}", path);
        // let _metacall = switch::initialize().unwrap();
        let instant = Instant::now();

        println!("Rebuilding page Rel path: {:?} Rebuilding page ", path);
        println!(
            "{:?}",
            path.to_str().ok_or_else(|| anyhow!("couldn't find path"))?,
        );

        println!("{:?}", self.out_dir);
        // Build client-side bundle
        {
            let instant = Instant::now();
            let client_builder = ClientBuilder::new(
                "",
                self.out_dir
                    .to_str()
                    .ok_or_else(|| anyhow!("couldn't find out dir path"))?,
            )?
            .build();

            println!("{:?}", client_builder);

            if let Err(e) = client_builder {
                error!(
                    target = "rebuilder",
                    message = format!("Couldn't build for the client side:  {e}"),
                );
                return Err(anyhow!("Couldn't continue rebuilding process."));
            }

            info!(
                target = "rebuilder",
                message = "Client building is completed",
                time = format!("{}ms", instant.elapsed().as_millis())
            );
        }

        // Build server-side bundle
        {
            let instant = Instant::now();

            let server_builder = ServerSideBuilder::new(
                "",
                self.out_dir
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid output path"))?,
                self.building_type,
            )?;

            if let Err(e) = server_builder.build() {
                error!(
                    target = "rebuilder",
                    message = format!("Failed to build server-side for {}: {}", path.display(), e)
                );
                return Err(anyhow!("Server-side build failed"));
            }

            info!(
                target = "rel_path",
                message = "Server building is completed",
                time = format!("{}ms", instant.elapsed().as_millis())
            );
        }

        Ok(())
    }

    async fn rebuild_all_pages(&self) -> Result<()> {
        // todo: itereate entered rebuilding "rebuild_page fn-" on all pages
        Ok(())
    }
}
