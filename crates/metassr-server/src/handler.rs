use anyhow::Result;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use metassr_build::server::renderer::page::PageRenderer;
use metassr_fs_analyzer::{
    dist_dir::{DistDir, PageEntry},
    DirectoryAnalyzer,
};
use metassr_utils::routes::{resolve_duplicates, DuplicateRoutesPolicy, RouteCandidate, RouteKind};
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path as FsPath, PathBuf},
    sync::Arc,
};
use tracing::error;

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

impl PageSource {
    fn respond(
        &self,
        params: &HashMap<String, String>,
        query: &HashMap<String, String>,
    ) -> Response {
        match self {
            PageSource::Static(html) => Html(html.clone()).into_response(),
            PageSource::Dynamic(renderer) => match renderer.render_with_context(params, query) {
                Ok(html) => Html(html).into_response(),
                Err(e) => {
                    error!("SSR render failed: {e:?}");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
                }
            },
        }
    }
}

/// Manifest key of the page served for unknown routes (`src/pages/_notfound`).
const NOT_FOUND_PAGE: &str = "_notfound";

pub struct PagesHandler<'a, S: Clone + Send + Sync + 'static> {
    pub app: &'a mut RouterMut<S>,
    pub pages: HashMap<String, PageEntry>,
    pub dist_dir: PathBuf,
    pub running_type: RunningType,
    pub duplicate_policy: DuplicateRoutesPolicy,
    pub root_path: PathBuf,
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
        })
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

        for candidate in resolved {
            let PageCandidate {
                key,
                route_path,
                entry: entries,
            } = candidate;

            let source = match self.running_type {
                RunningType::StaticSiteGeneration => {
                    PageSource::Static(read_to_string(entries.path.join("index.html"))?)
                }
                RunningType::ServerSideRendering => PageSource::Dynamic(Arc::new(
                    PageRenderer::from_manifest(&self.dist_dir, &key)?,
                )),
            };

            // Unknown routes render the `_notfound` page in place with a 404,
            // in both SSG and SSR, instead of redirecting to it (#239).
            if key == NOT_FOUND_PAGE {
                let source = source.clone();
                self.app.fallback(
                    move |Query(query): Query<HashMap<String, String>>| async move {
                        let mut response = source.respond(&HashMap::new(), &query);
                        // Keep a 500 from a failed render; only a rendered page becomes a 404.
                        if response.status() == StatusCode::OK {
                            *response.status_mut() = StatusCode::NOT_FOUND;
                        }
                        response
                    },
                );
            }

            let handler =
                move |Query(query): Query<HashMap<String, String>>,
                      Path(params): Path<HashMap<String, String>>| async move {
                    source.respond(&params, &query)
                };

            self.app.route(&route_path, get(handler));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
        Router,
    };
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

    #[tokio::test]
    async fn unknown_route_renders_notfound_page_with_404() {
        let tmp = tempfile::TempDir::new().unwrap();
        let pages = tmp.path().join("pages");
        fs::create_dir_all(&pages).unwrap();
        fs::write(pages.join("index.js"), "// root script").unwrap();
        fs::write(pages.join("index.html"), "<html><body>root</body></html>").unwrap();
        write_page(&pages, "_notfound", "custom not found");

        let mut router = RouterMut::from(Router::new());
        let mut handler = PagesHandler::new(
            &mut router,
            tmp.path().to_str().unwrap(),
            RunningType::StaticSiteGeneration,
            DuplicateRoutesPolicy::Error,
            tmp.path(),
        )
        .unwrap();
        handler.build().unwrap();

        let response = Service::call(
            &mut router.app(),
            Request::builder()
                .uri("/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(response.headers().get("location").is_none());
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(String::from_utf8_lossy(&body).contains("custom not found"));
    }
}
