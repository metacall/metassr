use anyhow::Result;
use axum::{
    extract::{Path, Query},
    response::Html,
    routing::get,
};
use metassr_build::server::renderer::page::PageRenderer;
use metassr_fs_analyzer::{
    dist_dir::{DistDir, PageEntry},
    DirectoryAnalyzer,
};
use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use crate::RunningType;

use super::router::RouterMut;

pub struct PagesHandler<'a, S: Clone + Send + Sync + 'static> {
    pub app: &'a mut RouterMut<S>,
    pub pages: HashMap<String, PageEntry>,
    pub dist_dir: PathBuf,
    pub running_type: RunningType,
}

impl<'a, S: Clone + Send + Sync + 'static> PagesHandler<'a, S> {
    pub fn new(
        app: &'a mut RouterMut<S>,
        dist_dir: &str,
        running_type: RunningType,
    ) -> Result<Self> {
        Ok(Self {
            app,
            pages: DistDir::new(&dist_dir)?.analyze()?.pages,
            dist_dir: PathBuf::from(dist_dir),
            running_type,
        })
    }
    pub fn build(&mut self) -> Result<()> {
        for (route, entries) in self.pages.iter() {
            let html = match self.running_type {
                RunningType::StaticSiteGeneration => {
                    Box::new(read_to_string(entries.path.join("index.html"))?)
                }
                RunningType::ServerSideRendering => {
                    Box::new(PageRenderer::from_manifest(&self.dist_dir, route)?.render()?)
                }
            };

            let handler =
                move |Query(_params): Query<HashMap<String, String>>,
                      Path(_path): Path<HashMap<String, String>>| async move {
                    Html(*html)
                };

            let route = format!(
                "/{}",
                match route {
                    e if e == &String::from("#root") => "".to_string(),
                    _ => route.replace('$', ":"),
                }
            );
            self.app.route(&route, get(handler));
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
}
