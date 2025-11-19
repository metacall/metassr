use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::{anyhow, Result};
use metassr_build::{
    client::ClientBuilder,
    server::{BuildingType, ServerSideBuilder},
    traits::Build,
};
use metassr_watcher::utils::*;
use tokio::sync::broadcast;

use std::fmt;
use std::time::Instant;

use notify_debouncer_full::DebouncedEvent;

use tracing::{debug, error};

#[derive(Clone, Debug)]
pub enum RebuildType {
    /// Rebuild a single page. page's path is providied
    Page(PathBuf), // this only is done
    Layout,
    // Rebuild a single Component.
    Component,
    // Reload Styles only.
    Style,
    Static,
}

impl fmt::Display for RebuildType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RebuildType::Page(path) => {
                write!(f, "page:{}", path.to_string_lossy())
            }
            RebuildType::Layout => write!(f, "layout"),
            RebuildType::Component => write!(f, "component"),
            RebuildType::Style => write!(f, "style"),
            RebuildType::Static => write!(f, "static"),
        }
    }
}

pub struct Rebuilder {
    sender: broadcast::Sender<RebuildType>,
    root_path: PathBuf,
    out_dir: PathBuf,
    building_type: BuildingType,
    is_rebuilding: Arc<AtomicBool>,
}

impl Rebuilder {
    pub fn new(root_path: PathBuf, building_type: BuildingType) -> Result<Self> {
        let (sender, _) = broadcast::channel(100);
        let out_dir = PathBuf::from("dist");

        Ok(Self {
            sender,
            root_path,
            out_dir,
            building_type,
            is_rebuilding: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RebuildType> {
        self.sender.subscribe()
    }

    pub fn handle_event(&self, event: DebouncedEvent) -> Result<RebuildType> {
        if !is_relevant_event(&event) {
            anyhow::bail!("Not a relevant event");
        }

        let path = event
            .paths
            .first()
            .ok_or_else(|| anyhow::anyhow!("No path"))?;

        let rel_path: &Path = path.strip_prefix(&self.root_path)?;

        let rebuild_type = self.map_path_to_type(rel_path)?;

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

    pub fn rebuild(&self, rebuild_type: RebuildType) -> Result<()> {
        if self.is_rebuilding.swap(true, Ordering::SeqCst) {
            return Ok(()); // Already rebuilding, skip
        }

        match rebuild_type {
            RebuildType::Page(ref path) => {
                debug!("entered rebuilding {:?} in {:?}", rebuild_type, path);

                self.rebuild_page(path.clone())?;
                match self.sender.send(rebuild_type.clone()) {
                    Ok(rec) => {
                        debug!("Sent to: {rec} receivers")
                    }
                    Err(e) => {
                        debug!("FULL CHANNEL: {e}");
                    }
                };
            }
            RebuildType::Layout => {
                // todo
                debug!("entered rebuilding {:?}", rebuild_type);
            }
            RebuildType::Component => {
                // todo
                debug!("entered rebuilding {:?}", rebuild_type);
            }
            RebuildType::Style => {
                // todo
                debug!("entered rebuilding {:?}", rebuild_type);
            }
            RebuildType::Static => {
                // todo
                debug!("entered rebuilding {:?}", rebuild_type);
            }
        }

        self.is_rebuilding.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn rebuild_page(&self, path: PathBuf) -> Result<()> {
        debug!("Rebuilding page {:?}", path);

        debug!("Rebuilding page Rel path: {:?} Rebuilding page ", path);

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

            if let Err(e) = client_builder {
                error!(
                    target = "rebuilder",
                    message = format!("Couldn't build for the client side:  {e}"),
                );
                return Err(anyhow!("Couldn't continue rebuilding process."));
            }

            debug!(
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

            debug!(
                target = "rel_path",
                message = "Server building is completed",
                time = format!("{}ms", instant.elapsed().as_millis())
            );
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn rebuild_all_pages(&self) -> Result<()> {
        todo!("iterate entered rebuilding rebuild_page() on all pages")
    }
}
