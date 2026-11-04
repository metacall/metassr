use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use crate::{
    client::ClientBuilder,
    server::{BuildingType, ServerSideBuilder},
};
use anyhow::{anyhow, Result};
use metassr_api_handler::{scanner::tag_for_path, ApiRoutes};
use metassr_bundler::WebBundler;
use metassr_utils::ansi::ansi_regex;
use metassr_watcher::utils::*;
use tokio::sync::broadcast;

use std::fmt;
use std::time::Instant;

use notify_debouncer_full::DebouncedEvent;

use tracing::{debug, error, info, warn};
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
    /// Reload a single API handler script in `src/api`. The script's path is provided.
    Api(PathBuf),
    // Rebuild a single Component.
    Component,
    // Reload Styles only.
    Style,
    Static,
    /// A compilation error occurred during rebuild, so the dev overlay can display it.
    BuildError {
        errors: Vec<String>,
    },
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
            RebuildType::BuildError { errors } => write!(f, "build_error:{}", errors.join(", ")),
        }
    }
}

pub struct Rebuilder {
    sender: broadcast::Sender<RebuildType>,
    root_path: PathBuf,
    out_dir: PathBuf,
    building_type: BuildingType,
    is_rebuilding: Arc<AtomicBool>,
    last_errors: Arc<Mutex<Option<Vec<String>>>>,
    /// The API routes loaded by the server, set once they are registered.
    api_routes: Mutex<Option<Arc<ApiRoutes>>>,
}

impl Rebuilder {
    pub fn new(root_path: PathBuf, building_type: BuildingType, out_dir: PathBuf) -> Result<Self> {
        let (sender, _) = broadcast::channel(100);

        Ok(Self {
            sender,
            root_path,
            out_dir,
            building_type,
            is_rebuilding: Arc::new(AtomicBool::new(false)),
            last_errors: Arc::new(Mutex::new(None)),
            api_routes: Mutex::new(None),
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RebuildType> {
        self.sender.subscribe()
    }

    pub fn building_type(&self) -> BuildingType {
        self.building_type
    }

    pub fn last_errors(&self) -> Arc<Mutex<Option<Vec<String>>>> {
        Arc::clone(&self.last_errors)
    }

    pub fn set_last_errors(&self, errors: Vec<String>) {
        *self.last_errors.lock().unwrap() = Some(errors);
    }

    pub fn clear_last_errors(&self) {
        *self.last_errors.lock().unwrap() = None;
    }

    /// Called by the server after it registers the API routes, so that changes
    /// to scripts in `src/api` can be hot-reloaded.
    pub fn set_api_routes(&self, api_routes: Arc<ApiRoutes>) {
        *self.api_routes.lock().unwrap() = Some(api_routes);
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

                if let Err(e) = self.rebuild_page(path.clone()) {
                    let err_msg = e.to_string();
                    self.set_last_errors(vec![err_msg.clone()]);
                    let _ = self.sender.send(RebuildType::BuildError {
                        errors: vec![err_msg],
                    });
                    return Err(e);
                }

                self.clear_last_errors();
                match self.sender.send(rebuild_type.clone()) {
                    Ok(rec) => {
                        debug!("Sent to: {rec} receivers")
                    }
                    Err(e) => {
                        debug!("FULL CHANNEL: {e}");
                    }
                };
            }
            RebuildType::Api(ref path) => {
                debug!("entered rebuilding {:?}", rebuild_type);
                self.reload_api(path)?;
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
            RebuildType::BuildError { .. } => {
                // BuildError is emitted by rebuild() and should not trigger another rebuild.
            }
        }

        Ok(())
    }

    /// Reload an API script after it changed on disk.
    ///
    /// Files that are not API scripts, deleted files and files added after
    /// startup are skipped (the router is built once, so new routes need a
    /// restart).
    fn reload_api(&self, rel_path: &Path) -> Result<()> {
        let abs_path = self.root_path.join(rel_path);
        let Some(tag) = tag_for_path(&abs_path) else {
            debug!("Not an API script, skipping: {:?}", rel_path);
            return Ok(());
        };
        if !abs_path.is_file() {
            debug!("API script no longer exists, skipping: {:?}", rel_path);
            return Ok(());
        }

        let Some(api_routes) = self.api_routes.lock().unwrap().clone() else {
            warn!(
                "API routes are not registered; cannot reload {:?}",
                rel_path
            );
            return Ok(());
        };
        if !api_routes.is_registered(&abs_path) {
            info!(
                "New API script {:?} detected; restart the dev server to register its route.",
                rel_path
            );
            return Ok(());
        }

        api_routes.reload_script(&abs_path, tag)
    }

    fn rebuild_page(&self, path: PathBuf) -> Result<()> {
        debug!("Rebuilding page {:?}", path);

        let out_dir = self
            .out_dir
            .to_str()
            .ok_or_else(|| anyhow!("Invalid output path"))?;

        let client_builder = ClientBuilder::new("", out_dir, true)?;
        let server_builder = ServerSideBuilder::new("", out_dir, self.building_type, true)?;

        // Generate targets for both client and server
        let client_targets = client_builder.generate_targets()?;
        let server_state = server_builder.generate_targets()?;

        // Combine all targets into a single rspack compilation
        let mut combined_targets = client_targets;
        combined_targets.extend(server_state.bundling_targets.clone());

        {
            let instant = Instant::now();
            let bundler = WebBundler::new(&combined_targets, out_dir, true)?;
            if let Err(e) = bundler.exec() {
                let msg = format!("Bundling failed: {e}");
                let clean_log_msg = ansi_regex().replace_all(&msg, "").to_string();
                error!(target = "rebuilder", message = clean_log_msg);
                return Err(anyhow!(msg));
            }
            debug!(
                target = "rebuilder",
                message = "Bundling is completed",
                time = format!("{}ms", instant.elapsed().as_millis())
            );
        }

        // Server post-processing
        if let Err(e) = server_builder.finish_build(server_state) {
            error!(
                target = "rebuilder",
                message = format!("Failed to build server-side for {}: {}", path.display(), e)
            );
            return Err(anyhow!("Server-side build failed"));
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn rebuild_all_pages(&self) -> Result<()> {
        todo!("iterate entered rebuilding rebuild_page() on all pages")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::BuildingType;

    fn test_rebuilder() -> Rebuilder {
        Rebuilder::new(
            PathBuf::from("."),
            BuildingType::ServerSideRendering,
            PathBuf::from("dist"),
        )
        .unwrap()
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
    fn map_path_to_type_detects_api_paths() {
        let rebuilder = test_rebuilder();
        for path in ["src/api/hello.js", "src/api/users/list.py"] {
            match rebuilder.map_path_to_type(Path::new(path)).unwrap() {
                RebuildType::Api(api_path) => assert_eq!(api_path, PathBuf::from(path)),
                other => panic!("expected Api for {path}, got {other:?}"),
            }
        }
    }

    #[test]
    fn api_rebuild_without_registered_routes_is_a_no_op() {
        let rebuilder = test_rebuilder();
        // Nothing registered yet, a non-script and a missing file: all skipped.
        for path in [
            "src/api/hello.js",
            "src/api/README.md",
            "src/api/missing.py",
        ] {
            assert!(rebuilder
                .rebuild(RebuildType::Api(PathBuf::from(path)))
                .is_ok());
        }
    }

    #[tokio::test]
    async fn api_rebuild_hot_reloads_registered_handler() {
        use metassr_api_handler::types::ApiRequest;
        use std::collections::HashMap;

        fn handler(version: &str) -> String {
            format!(
                "function GET(req) {{ return JSON.stringify({{ status: 200, body: {{ version: '{version}' }} }}); }}\n\
                 module.exports = {{ GET }};\n"
            )
        }

        async fn version(routes: &ApiRoutes, script: &Path) -> String {
            let request = ApiRequest {
                url: "/api/version".to_string(),
                headers: HashMap::new(),
                method: "GET".to_string(),
                body: None,
                params: HashMap::new(),
                query: HashMap::new(),
            };
            let response = routes
                .call_handler(&script.to_string_lossy(), "GET", request)
                .await
                .unwrap();
            response.body["version"].as_str().unwrap().to_string()
        }

        let _metacall = metacall::initialize().unwrap();
        let root = tempfile::tempdir().unwrap();
        let api_dir = root.path().join("src").join("api");
        std::fs::create_dir_all(&api_dir).unwrap();
        let script = api_dir.join("version.js");
        std::fs::write(&script, handler("v1")).unwrap();

        let mut routes = ApiRoutes::new();
        routes.load_from_dir(&api_dir).await.unwrap();
        let routes = Arc::new(routes);

        let rebuilder = Rebuilder::new(
            root.path().to_path_buf(),
            BuildingType::ServerSideRendering,
            root.path().join("dist"),
        )
        .unwrap();
        rebuilder.set_api_routes(Arc::clone(&routes));
        let change = || RebuildType::Api(PathBuf::from("src/api/version.js"));

        assert_eq!(version(&routes, &script).await, "v1");

        std::fs::write(&script, handler("v2")).unwrap();
        rebuilder.rebuild(change()).unwrap();
        assert_eq!(version(&routes, &script).await, "v2");

        // Saving the same content again (editors often emit several events per
        // save) is a no-op.
        rebuilder.rebuild(change()).unwrap();
        assert_eq!(version(&routes, &script).await, "v2");

        // A broken edit is reported, and the last working handler keeps serving.
        std::fs::write(&script, "function GET(req) {").unwrap();
        assert!(rebuilder.rebuild(change()).is_err());
        assert_eq!(version(&routes, &script).await, "v2");
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
        let rebuilder = Rebuilder::new(
            tmp.path().to_path_buf(),
            BuildingType::ServerSideRendering,
            PathBuf::from("dist"),
        )
        .unwrap();
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
        let rebuilder = Rebuilder::new(
            tmp.path().to_path_buf(),
            BuildingType::ServerSideRendering,
            PathBuf::from("dist"),
        )
        .unwrap();
        let second = rebuilder.rebuild(RebuildType::Page(PathBuf::from(
            "src/pages/nonexistent.tsx",
        )));
        assert!(
            second.is_err(),
            "page rebuild after api rebuild should still attempt (not skipped)"
        );
    }
}
