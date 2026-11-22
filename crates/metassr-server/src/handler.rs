use anyhow::{anyhow, Result};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use metassr_build::{
    rebuilder::{RebuildType, Rebuilder},
    server::renderer::page::PageRenderer,
};
use metassr_fs_analyzer::{
    dist_dir::{DistDir, PageEntry},
    DirectoryAnalyzer,
};
use metassr_utils::routes::{resolve_duplicates, DuplicateRoutesPolicy, RouteCandidate, RouteKind};
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path as FsPath, PathBuf},
    sync::{Arc, RwLock},
};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info};

use crate::RunningType;

use super::router::RouterMut;

/// How a registered page route produces its HTML body.
#[derive(Clone)]
enum PageSource {
    /// Pre-rendered HTML read from disk (SSG).
    Static(String),
    /// Renderer re-invoked on every request (SSR).
    Dynamic(Arc<PageRenderer>),
}

pub struct PagesHandler<'a, S: Clone + Send + Sync + 'static> {
    pub app: &'a mut RouterMut<S>,
    pub pages: HashMap<String, PageEntry>,
    pub dist_dir: PathBuf,
    pub running_type: RunningType,
    pub duplicate_policy: DuplicateRoutesPolicy,
    pub root_path: PathBuf,
    pub rebuilder: Option<Arc<Rebuilder>>,
    rebuild_rx: Option<broadcast::Receiver<RebuildType>>,
    reload_tx: Option<mpsc::UnboundedSender<String>>,
}

/// A page route candidate that keeps both the manifest key (used by the
/// renderer) and the axum route (used for registration).
struct PageCandidate {
    key: String,
    route_path: String,
    entry: PageEntry,
}

impl RouteCandidate for PageCandidate {
    fn route_path(&self) -> &str {
        &self.route_path
    }

    fn file_path(&self) -> &FsPath {
        &self.entry.path
    }
}

/// Convert a page manifest key (`#root`, `blog/$article`) to an axum route.
fn page_route(key: &str) -> String {
    match key {
        "#root" => "/".to_string(),
        route => format!("/{}", route.replace('\\', "/").replace('$', ":")),
    }
}

impl<'a, S: Clone + Send + Sync + 'static> PagesHandler<'a, S> {
    pub fn new(
        app: &'a mut RouterMut<S>,
        dist_dir: &str,
        running_type: RunningType,
        duplicate_policy: DuplicateRoutesPolicy,
        root_path: &FsPath,
    ) -> Result<Self> {
        Ok(Self {
            app,
            pages: DistDir::new(&dist_dir)?.analyze()?.pages,
            dist_dir: PathBuf::from(dist_dir),
            running_type,
            duplicate_policy,
            root_path: root_path.to_path_buf(),
            rebuilder: None,
            rebuild_rx: None,
            reload_tx: None,
        })
    }

    pub fn with_rebuilder(mut self, rebuilder: Option<Arc<Rebuilder>>) -> Self {
        self.rebuilder = rebuilder;
        self
    }

    pub fn with_rebuild_rx(mut self, rx: broadcast::Receiver<RebuildType>) -> Self {
        self.rebuild_rx = Some(rx);
        self
    }

    pub fn with_reload_tx(mut self, tx: mpsc::UnboundedSender<String>) -> Self {
        self.reload_tx = Some(tx);
        self
    }

    pub fn build(&mut self) -> Result<()> {
        let candidates: Vec<PageCandidate> = self
            .pages
            .iter()
            .map(|(key, entry)| PageCandidate {
                key: key.clone(),
                route_path: page_route(key),
                entry: entry.clone(),
            })
            .collect();

        let resolved = resolve_duplicates(
            candidates,
            self.duplicate_policy,
            RouteKind::Page,
            &self.root_path,
        )?;

        let mut sources = HashMap::new();

        for candidate in resolved {
            let PageCandidate {
                key,
                route_path,
                entry: entries,
            } = candidate;

            let initial_source = match self.running_type {
                RunningType::StaticSiteGeneration => {
                    PageSource::Static(read_to_string(entries.path.join("index.html"))?)
                }
                RunningType::ServerSideRendering => PageSource::Dynamic(Arc::new(
                    PageRenderer::from_manifest(&self.dist_dir, &key)?,
                )),
            };

            let source_cell = Arc::new(RwLock::new(initial_source));
            sources.insert(key.clone(), Arc::clone(&source_cell));

            let cell = Arc::clone(&source_cell);
            let handler =
                move |Query(query): Query<HashMap<String, String>>,
                      Path(params): Path<HashMap<String, String>>| {
                    let cell = Arc::clone(&cell);
                    async move {
                        let source = match cell.read() {
                            Ok(guard) => guard.clone(),
                            Err(poisoned) => poisoned.into_inner().clone(),
                        };

                        match source {
                            PageSource::Static(html) => Html(html).into_response(),
                            PageSource::Dynamic(renderer) => {
                                match renderer.render_with_context(&params, &query) {
                                    Ok(html) => Html(html).into_response(),
                                    Err(e) => {
                                        error!("SSR render failed: {e:?}");
                                        (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
                                            .into_response()
                                    }
                                }
                            }
                        }
                    }
                };

            self.app.route(&route_path, get(handler));
        }

        let rx_opt = self
            .rebuild_rx
            .take()
            .or_else(|| self.rebuilder.as_ref().map(|r| r.subscribe()));

        if let Some(mut rx) = rx_opt {
            let sources = Arc::new(sources);
            let dist_dir = self.dist_dir.clone();
            let running_type = self.running_type;
            let pages = self.pages.clone();
            let reload_tx = self.reload_tx.clone();

            tokio::spawn(async move {
                loop {
                    let rebuild_type = match rx.recv().await {
                        Ok(event) => event,
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(broadcast::error::RecvError::Closed) => break,
                    };

                    let page_path = match rebuild_type {
                        RebuildType::Page(ref path) => path,
                        _ => continue,
                    };

                    let route = match find_route_for_page_path(pages.keys(), page_path) {
                        Some(r) => r,
                        None => {
                            debug!("No route found matching page path: {page_path:?}");
                            continue;
                        }
                    };

                    let source_cell = match sources.get(&route) {
                        Some(cell) => Arc::clone(cell),
                        None => {
                            debug!("Route {route} not found in registered sources");
                            continue;
                        }
                    };

                    match reload_page_source(&dist_dir, &route, running_type, &pages) {
                        Ok(new_source) => {
                            let mut guard = match source_cell.write() {
                                Ok(g) => g,
                                Err(poisoned) => poisoned.into_inner(),
                            };
                            *guard = new_source;
                            info!("Reloaded page renderer for route: {route}");
                            if let Some(tx) = reload_tx.as_ref() {
                                let _ = tx.send(route);
                            }
                        }
                        Err(e) => {
                            error!("Failed to reload page renderer for route {route}: {e:?}");
                        }
                    }
                }
            });
        }

        Ok(())
    }
}

fn reload_page_source(
    dist_dir: &FsPath,
    route: &str,
    running_type: RunningType,
    pages: &HashMap<String, PageEntry>,
) -> Result<PageSource> {
    match running_type {
        RunningType::StaticSiteGeneration => {
            let entry = pages
                .get(route)
                .ok_or_else(|| anyhow!("No page entry found for route: {route}"))?;
            let html = read_to_string(entry.path.join("index.html"))?;
            Ok(PageSource::Static(html))
        }
        RunningType::ServerSideRendering => {
            let renderer = PageRenderer::from_manifest(dist_dir, route)?;
            Ok(PageSource::Dynamic(Arc::new(renderer)))
        }
    }
}

pub(crate) fn find_route_for_page_path<'a>(
    routes: impl IntoIterator<Item = &'a String>,
    path: &FsPath,
) -> Option<String> {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let after_pages = if let Some(idx) = normalized.find("src/pages/") {
        &normalized[idx + "src/pages/".len()..]
    } else if let Some(stripped) = normalized.strip_prefix("src/pages") {
        stripped.trim_start_matches('/')
    } else {
        normalized.trim_start_matches("./")
    };

    let without_ext = match after_pages.rsplit_once('.') {
        Some((base, _ext)) => base,
        None => after_pages,
    };

    let candidate = match without_ext {
        "" | "index" => "#root".to_string(),
        other => {
            if let Some(prefix) = other.strip_suffix("/index") {
                prefix.to_string()
            } else {
                other.to_string()
            }
        }
    };

    for r in routes {
        let norm_r = r.replace('\\', "/");
        if norm_r == candidate
            || (candidate != "#root" && norm_r == format!("{candidate}/index"))
            || (candidate == "#root" && (norm_r == "index" || norm_r == "#root"))
        {
            return Some(r.clone());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
        Router,
    };
    use metassr_build::server::BuildingType;
    use std::fs;
    use tower_service::Service;

    fn write_page(pages: &std::path::Path, route: &str, body: &str) {
        let page = pages.join(route);
        fs::create_dir_all(&page).unwrap();
        fs::write(page.join("index.js"), "// script").unwrap();
        fs::write(
            page.join("index.html"),
            format!("<html><body>{body}</body></html>"),
        )
        .unwrap();
    }

    #[tokio::test]
    async fn build_registers_root_and_nested_routes() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dist_dir = tmp.path();
        let root_pages = dist_dir.join("pages");
        let home_page = root_pages.join("home");

        fs::create_dir_all(&home_page).unwrap();
        fs::write(root_pages.join("index.js"), "// root script").unwrap();
        fs::write(
            root_pages.join("index.html"),
            "<html><body>root</body></html>",
        )
        .unwrap();
        fs::write(home_page.join("index.js"), "// home script").unwrap();
        fs::write(
            home_page.join("index.html"),
            "<html><body>home</body></html>",
        )
        .unwrap();

        let mut router = RouterMut::from(Router::new());
        let mut handler = PagesHandler::new(
            &mut router,
            dist_dir.to_str().unwrap(),
            RunningType::StaticSiteGeneration,
            DuplicateRoutesPolicy::Error,
            dist_dir,
        )
        .unwrap();
        handler.build().unwrap();

        let mut app = router.app();

        let root_response = Service::call(
            &mut app,
            Request::builder().uri("/").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(root_response.status(), StatusCode::OK);
        let root_body = to_bytes(root_response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(String::from_utf8_lossy(&root_body).contains("root"));

        let home_response = Service::call(
            &mut app,
            Request::builder().uri("/home").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(home_response.status(), StatusCode::OK);
        let home_body = to_bytes(home_response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(String::from_utf8_lossy(&home_body).contains("home"));
    }

    #[tokio::test]
    async fn build_rejects_parameter_name_collision_in_error_mode() {
        let tmp = tempfile::TempDir::new().unwrap();
        let pages = tmp.path().join("pages");
        write_page(&pages, "blog/$article", "article");
        write_page(&pages, "blog/$slug", "slug");

        let mut router: RouterMut<()> = RouterMut::from(Router::new());
        let mut handler = PagesHandler::new(
            &mut router,
            tmp.path().to_str().unwrap(),
            RunningType::StaticSiteGeneration,
            DuplicateRoutesPolicy::Error,
            tmp.path(),
        )
        .unwrap();

        let error = handler
            .build()
            .expect_err("colliding parameter names must fail the build");
        let message = error.to_string();
        assert!(message.contains("$article"), "{message}");
        assert!(message.contains("$slug"), "{message}");
    }

    #[tokio::test]
    async fn build_keeps_first_parameter_route_in_warn_mode() {
        let tmp = tempfile::TempDir::new().unwrap();
        let pages = tmp.path().join("pages");
        write_page(&pages, "blog/$article", "article");
        write_page(&pages, "blog/$slug", "slug");

        let mut router = RouterMut::from(Router::new());
        let mut handler = PagesHandler::new(
            &mut router,
            tmp.path().to_str().unwrap(),
            RunningType::StaticSiteGeneration,
            DuplicateRoutesPolicy::Warn,
            tmp.path(),
        )
        .unwrap();
        handler.build().unwrap();

        let mut app = router.app();
        let response = Service::call(
            &mut app,
            Request::builder()
                .uri("/blog/hello")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        // `:article` sorts before `:slug`, so the first route is kept.
        assert!(String::from_utf8_lossy(&body).contains("article"));
    }

    #[test]
    fn test_find_route_for_page_path() {
        let routes = vec![
            "#root".to_string(),
            "about".to_string(),
            "users/$id".to_string(),
            "blog/index".to_string(),
        ];

        assert_eq!(
            find_route_for_page_path(&routes, FsPath::new("src/pages/index.tsx")),
            Some("#root".to_string())
        );
        assert_eq!(
            find_route_for_page_path(&routes, FsPath::new("src/pages/index.jsx")),
            Some("#root".to_string())
        );
        assert_eq!(
            find_route_for_page_path(&routes, FsPath::new(r"src\pages\index.tsx")),
            Some("#root".to_string())
        );
        assert_eq!(
            find_route_for_page_path(&routes, FsPath::new("src/pages/about.tsx")),
            Some("about".to_string())
        );
        assert_eq!(
            find_route_for_page_path(&routes, FsPath::new("src/pages/about/index.tsx")),
            Some("about".to_string())
        );
        assert_eq!(
            find_route_for_page_path(&routes, FsPath::new("src/pages/users/$id.tsx")),
            Some("users/$id".to_string())
        );
        assert_eq!(
            find_route_for_page_path(&routes, FsPath::new("src/pages/blog/index.tsx")),
            Some("blog/index".to_string())
        );
        assert_eq!(
            find_route_for_page_path(&routes, FsPath::new("src/components/header.tsx")),
            None
        );
        assert_eq!(
            find_route_for_page_path(&routes, FsPath::new("src/pages/contact.tsx")),
            None
        );
    }

    #[tokio::test]
    async fn rebuild_reloads_page_dynamically_without_server_restart() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dist_dir = tmp.path();
        let root_pages = dist_dir.join("pages");
        let home_page = root_pages.join("home");

        fs::create_dir_all(&home_page).unwrap();
        fs::write(root_pages.join("index.js"), "// root script").unwrap();
        fs::write(
            root_pages.join("index.html"),
            "<html><body>root initial</body></html>",
        )
        .unwrap();
        fs::write(home_page.join("index.js"), "// home script").unwrap();
        fs::write(
            home_page.join("index.html"),
            "<html><body>home initial</body></html>",
        )
        .unwrap();

        let rebuilder = Arc::new(
            Rebuilder::new(
                dist_dir.to_path_buf(),
                BuildingType::StaticSiteGeneration,
                dist_dir.to_path_buf(),
            )
            .unwrap(),
        );

        let (reload_tx, mut reload_rx) = mpsc::unbounded_channel();

        let mut router = RouterMut::from(Router::new());
        let mut handler = PagesHandler::new(
            &mut router,
            dist_dir.to_str().unwrap(),
            RunningType::StaticSiteGeneration,
            DuplicateRoutesPolicy::Error,
            dist_dir,
        )
        .unwrap()
        .with_rebuilder(Some(Arc::clone(&rebuilder)))
        .with_reload_tx(reload_tx);
        handler.build().unwrap();

        let mut app = router.app();

        let root_res = Service::call(
            &mut app,
            Request::builder().uri("/").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();
        let root_body = to_bytes(root_res.into_body(), usize::MAX).await.unwrap();
        assert!(String::from_utf8_lossy(&root_body).contains("root initial"));

        let home_res = Service::call(
            &mut app,
            Request::builder().uri("/home").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();
        let home_body = to_bytes(home_res.into_body(), usize::MAX).await.unwrap();
        assert!(String::from_utf8_lossy(&home_body).contains("home initial"));

        fs::write(
            root_pages.join("index.html"),
            "<html><body>root updated</body></html>",
        )
        .unwrap();

        rebuilder
            .emit(RebuildType::Page(PathBuf::from("src/pages/index.tsx")))
            .unwrap();

        let reloaded_route = reload_rx.recv().await.expect("reload event not received");
        assert_eq!(reloaded_route, "#root");

        let root_res2 = Service::call(
            &mut app,
            Request::builder().uri("/").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();
        let root_body2 = to_bytes(root_res2.into_body(), usize::MAX).await.unwrap();
        assert!(
            String::from_utf8_lossy(&root_body2).contains("root updated"),
            "Subsequent request should render the updated page bundle without server restart"
        );

        let home_res2 = Service::call(
            &mut app,
            Request::builder().uri("/home").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();
        let home_body2 = to_bytes(home_res2.into_body(), usize::MAX).await.unwrap();
        assert!(
            String::from_utf8_lossy(&home_body2).contains("home initial"),
            "Unaffected routes must continue rendering their existing content"
        );
    }

    #[tokio::test]
    async fn failed_rebuild_preserves_existing_content() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dist_dir = tmp.path();
        let root_pages = dist_dir.join("pages");
        let home_page = root_pages.join("home");

        fs::create_dir_all(&home_page).unwrap();
        fs::write(root_pages.join("index.js"), "// root script").unwrap();
        fs::write(
            root_pages.join("index.html"),
            "<html><body>root initial</body></html>",
        )
        .unwrap();
        fs::write(home_page.join("index.js"), "// home script").unwrap();
        fs::write(
            home_page.join("index.html"),
            "<html><body>home initial</body></html>",
        )
        .unwrap();

        let (tx, rx) = broadcast::channel(16);
        let (reload_tx, mut reload_rx) = mpsc::unbounded_channel();

        let mut router = RouterMut::from(Router::new());
        let mut handler = PagesHandler::new(
            &mut router,
            dist_dir.to_str().unwrap(),
            RunningType::StaticSiteGeneration,
            DuplicateRoutesPolicy::Error,
            dist_dir,
        )
        .unwrap()
        .with_rebuild_rx(rx)
        .with_reload_tx(reload_tx);
        handler.build().unwrap();

        let mut app = router.app();

        // 1. Send BuildError, followed by a valid reload of home as a FIFO synchronization sentinel
        tx.send(RebuildType::BuildError {
            errors: vec!["syntax error in page".to_string()],
        })
        .unwrap();

        fs::write(
            home_page.join("index.html"),
            "<html><body>home reloaded</body></html>",
        )
        .unwrap();
        tx.send(RebuildType::Page(PathBuf::from("src/pages/home.tsx")))
            .unwrap();

        let reloaded = reload_rx.recv().await.expect("home reload expected");
        assert_eq!(reloaded, "home");

        let res = Service::call(
            &mut app,
            Request::builder().uri("/").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        assert!(
            String::from_utf8_lossy(&body).contains("root initial"),
            "Build error must not alter or break working page content"
        );

        // 2. Send rebuild for nonexistent route, followed by another reload of home as sentinel
        tx.send(RebuildType::Page(PathBuf::from("src/pages/missing.tsx")))
            .unwrap();

        fs::write(
            home_page.join("index.html"),
            "<html><body>home reloaded 2</body></html>",
        )
        .unwrap();
        tx.send(RebuildType::Page(PathBuf::from("src/pages/home.tsx")))
            .unwrap();

        let reloaded2 = reload_rx.recv().await.expect("home reload 2 expected");
        assert_eq!(reloaded2, "home");

        let res = Service::call(
            &mut app,
            Request::builder().uri("/").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        assert!(
            String::from_utf8_lossy(&body).contains("root initial"),
            "Rebuild for missing route must preserve current route content"
        );
    }

    #[tokio::test]
    async fn concurrent_requests_during_reload_do_not_fail() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dist_dir = tmp.path();
        let root_pages = dist_dir.join("pages");

        fs::create_dir_all(&root_pages).unwrap();
        fs::write(root_pages.join("index.js"), "// root script").unwrap();
        fs::write(
            root_pages.join("index.html"),
            "<html><body>root content</body></html>",
        )
        .unwrap();

        let (tx, rx) = broadcast::channel(16);
        let (reload_tx, mut reload_rx) = mpsc::unbounded_channel();

        let mut router = RouterMut::from(Router::new());
        let mut handler = PagesHandler::new(
            &mut router,
            dist_dir.to_str().unwrap(),
            RunningType::StaticSiteGeneration,
            DuplicateRoutesPolicy::Error,
            dist_dir,
        )
        .unwrap()
        .with_rebuild_rx(rx)
        .with_reload_tx(reload_tx);
        handler.build().unwrap();

        let app = Arc::new(tokio::sync::Mutex::new(router.app()));

        let mut handles = Vec::new();
        for _ in 0..10 {
            let app_clone = Arc::clone(&app);
            handles.push(tokio::spawn(async move {
                let mut app = app_clone.lock().await;
                let res = Service::call(
                    &mut *app,
                    Request::builder().uri("/").body(Body::empty()).unwrap(),
                )
                .await
                .unwrap();
                assert_eq!(res.status(), StatusCode::OK);
            }));
        }

        tx.send(RebuildType::Page(PathBuf::from("src/pages/index.tsx")))
            .unwrap();

        let reloaded = reload_rx.recv().await.expect("reload expected");
        assert_eq!(reloaded, "#root");

        for handle in handles {
            handle.await.unwrap();
        }
    }
}
