use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use anyhow::{anyhow, Result};
use metassr_api_handler::ApiRoutes;
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

use tracing::{debug, error, warn};
struct RebuildGuard<'a> {
    flag: &'a AtomicBool,
}
impl<'a> RebuildGuard<'a> {
    fn new(flag: &'a AtomicBool) -> Option<Self> {
        if flag.swap(true, Ordering::SeqCst) {
            None
        } else {
            Some(Self { flag })
        }
    }
}
impl Drop for RebuildGuard<'_> {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::SeqCst);
    }
}

#[derive(Clone, Debug)]
pub enum RebuildType {
    /// Rebuild a single page. page's path is provided
    Page(PathBuf),
    Layout,
    /// Reload a single API handler script.
    Api(PathBuf),
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
            RebuildType::Api(path) => write!(f, "api:{}", path.to_string_lossy()),
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
    /// Shared handle to the loaded API routes, set after the server registers them.
    api_routes: Mutex<Option<Arc<Mutex<ApiRoutes>>>>,
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
            api_routes: Mutex::new(None),
        })
    }

    /// Called by the server after API routes are loaded to enable hot-reloading.
    pub fn set_api_routes(&self, api_routes: Arc<Mutex<ApiRoutes>>) {
        *self.api_routes.lock().unwrap() = Some(api_routes);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RebuildType> {
        self.sender.subscribe()
    }

    pub fn handle_event(&self, event: DebouncedEvent) -> Result<Option<RebuildType>> {
        if !is_relevant_event(&event) {
            return Ok(None);
        }

        let path = event
            .paths
            .first()
            .ok_or_else(|| anyhow::anyhow!("No path in event"))?;

        let rel_path: &Path = path.strip_prefix(&self.root_path)?;

        let rebuild_type = self.map_path_to_type(rel_path)?;

        Ok(Some(rebuild_type))
    }

    fn map_path_to_type(&self, path: &Path) -> Result<RebuildType> {
        let path_buf = path.to_path_buf();
        let path_str = path.to_string_lossy(); // make path a Cow. not all filenames are valid UTF-8

        let rebuild_type: RebuildType = match path_str {
            path if path.starts_with("src/pages") => RebuildType::Page(path_buf.clone()),
            path if path.starts_with("src/api") => RebuildType::Api(path_buf.clone()),
            path if path.starts_with("src/layout") => RebuildType::Layout,
            path if path.starts_with("src/components") => RebuildType::Component,
            path if path.starts_with("src/styles") => RebuildType::Style,
            path if path.starts_with("static") => RebuildType::Static,
            // entered rebuilding everything if we're not sure of entered rebuilding kind
            _ => RebuildType::Layout,
        };

        Ok(rebuild_type)
    }

    pub fn rebuild(&self, rebuild_type: RebuildType) -> Result<()> {
        let _guard = match RebuildGuard::new(&self.is_rebuilding) {
            Some(guard) => guard,
            None => {
                debug!("rebuilding in progress, skipping");
                return Ok(());
            }
        };

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
            RebuildType::Api(ref rel_path) => {
                debug!("Reloading API handler: {:?}", rel_path);
                self.rebuild_api(rel_path.clone())?;
            }
            RebuildType::Layout => {
                // todo: implement granular layout rebuild
                debug!("entered rebuilding {:?}", rebuild_type);
                tracing::warn!("Layout rebuild is not yet implemented; skipping.");
            }
            RebuildType::Component => {
                // todo: implement granular component rebuild
                debug!("entered rebuilding {:?}", rebuild_type);
                tracing::warn!("Component rebuild is not yet implemented; skipping.");
            }
            RebuildType::Style => {
                // todo: implement granular style rebuild
                debug!("entered rebuilding {:?}", rebuild_type);
                tracing::warn!("Style rebuild is not yet implemented; skipping.");
            }
            RebuildType::Static => {
                // todo: implement static asset rebuild
                debug!("entered rebuilding {:?}", rebuild_type);
                tracing::warn!("Static asset rebuild is not yet implemented; skipping.");
            }
        }

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

    fn rebuild_api(&self, rel_path: PathBuf) -> Result<()> {
        let abs_path = self.root_path.join(&rel_path);

        let api_routes = self.api_routes.lock().unwrap().clone();
        match api_routes {
            Some(api_routes) => {
                api_routes.lock().unwrap().reload_script(&abs_path)?;
                Ok(())
            }
            None => {
                warn!(
                    "API routes not registered; cannot hot-reload {:?}",
                    rel_path
                );
                Ok(())
            }
        }
    }

    #[allow(dead_code)]
    fn rebuild_all_pages(&self) -> Result<()> {
        todo!("iterate entered rebuilding rebuild_page() on all pages")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metassr_build::server::BuildingType;

    fn test_rebuilder() -> Rebuilder {
        Rebuilder::new(PathBuf::from("."), BuildingType::ServerSideRendering).unwrap()
    }

    #[test]
    fn map_path_to_type_detects_page_paths() {
        let rebuilder = test_rebuilder();
        let result = rebuilder
            .map_path_to_type(Path::new("src/pages/blog/index.tsx"))
            .unwrap();

        match result {
            RebuildType::Page(path) => {
                assert_eq!(path, PathBuf::from("src/pages/blog/index.tsx"));
            }
            other => panic!("expected page rebuild type, got {}", other),
        }
    }

    #[test]
    fn map_path_to_type_detects_component_paths() {
        let rebuilder = test_rebuilder();
        let result = rebuilder
            .map_path_to_type(Path::new("src/components/button.tsx"))
            .unwrap();

        assert!(matches!(result, RebuildType::Component));
    }

    #[test]
    fn map_path_to_type_detects_static_paths() {
        let rebuilder = test_rebuilder();
        let result = rebuilder
            .map_path_to_type(Path::new("static/assets/logo.svg"))
            .unwrap();

        assert!(matches!(result, RebuildType::Static));
    }

    #[test]
    fn map_path_to_type_falls_back_to_layout_for_unknown_paths() {
        let rebuilder = test_rebuilder();
        let result = rebuilder
            .map_path_to_type(Path::new("scripts/rebuild-helper.ts"))
            .unwrap();

        assert!(matches!(result, RebuildType::Layout));
    }

    #[cfg(windows)]
    #[test]
    fn map_path_to_type_handles_windows_separators() {
        let rebuilder = test_rebuilder();
        let result = rebuilder
            .map_path_to_type(Path::new(r"src\components\button.tsx"))
            .unwrap();

        assert!(matches!(result, RebuildType::Component));
    }

    #[test]
    fn rebuild_flag_resets_after_error() {
        let tmp = tempfile::TempDir::new().unwrap();
        let rebuilder =
            Rebuilder::new(tmp.path().to_path_buf(), BuildingType::ServerSideRendering).unwrap();
        let first = rebuilder.rebuild(RebuildType::Page(PathBuf::from(
            "src/pages/nonexistent.tsx",
        )));
        assert!(
            first.is_err(),
            "first rebuild should fail with invalid path"
        );
        let second = rebuilder.rebuild(RebuildType::Page(PathBuf::from(
            "src/pages/nonexistent.tsx",
        )));
        assert!(
            second.is_err(),
            "second rebuild should attempt to rebuild (return Err), not return Ok(())"
        );
    }
    #[test]
    fn rebuild_flag_resets_after_successful_variant() {
        let tmp = tempfile::TempDir::new().unwrap();
        let rebuilder =
            Rebuilder::new(tmp.path().to_path_buf(), BuildingType::ServerSideRendering).unwrap();
        let first = rebuilder.rebuild(RebuildType::Api(PathBuf::from("src/api/test.js")));
        assert!(
            first.is_ok(),
            "api rebuild with no routes should succeed with a warning"
        );
        let second = rebuilder.rebuild(RebuildType::Page(PathBuf::from(
            "src/pages/nonexistent.tsx",
        )));
        assert!(
            second.is_err(),
            "page rebuild after api rebuild should still attempt (not skipped)"
        );
    }
}
