use std::path::{Path, PathBuf};

use anyhow::Result;
use metassr_watcher::utils::*;
use notify::Event;
use tokio::sync::broadcast;

#[derive(Clone, Debug,)]
pub enum RebuildType {
    Page(PathBuf),
    Layout,
    Component,
    Style,
    Static,
}

pub struct Rebuilder {
    sender: broadcast::Sender<RebuildType>,
    root_path: PathBuf,
}

impl Rebuilder {
    pub fn new(root_path: PathBuf) -> Self {
        let (sender, _) = broadcast::channel(100);

        Self { sender, root_path }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RebuildType> {
        self.sender.subscribe()
    }

    pub fn handle_event(&self, event: Event) -> Result<()> {
        if !is_relevant_event(&event) {
            return Ok(());
        }

        Ok(())
    }

    fn map_path_to_type(path: &Path) -> Result<RebuildType> {
        let path_buf = path.to_path_buf();
        let path_str = path.to_string_lossy(); // make path a Cow. not all filenames are valid UTF-8

        let rebuild_type: RebuildType = match path_str {
            path if path.starts_with("src/pages") => RebuildType::Page(path_buf.clone()),
            path0 if path.starts_with("src/layout") => RebuildType::Layout,
            path if path.starts_with("src/components") => RebuildType::Component,
            path if path.starts_with("src/styles") => RebuildType::Style,
            path if path.starts_with("static") => RebuildType::Static,
            // rebuilding everything if we're not surue of rebuilding kind
            _ => RebuildType::Layout,
        };

        Ok(rebuild_type)
    }

    pub fn rebuild(&self, rebuild_type: RebuildType) -> Result<()> {
        match rebuild_type {
            RebuildType::Page(ref path) => {
                // todo
                println!("rebuilding {:?} in {:?}", rebuild_type, path);
            }
            RebuildType::Layout => {
                // todo
                println!("rebuilding {:?}", rebuild_type);
            }
            RebuildType::Component => {
                // todo
                println!("rebuilding {:?}", rebuild_type);
            }
            RebuildType::Style => {
                // todo
                println!("rebuilding {:?}", rebuild_type);
            }
            RebuildType::Static => {
                // todo
                println!("rebuilding {:?}", rebuild_type);
            }
        }

        Ok(())
    }
}
