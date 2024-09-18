use anyhow::Result;
use axum::{
    extract::{Path, Query},
    response::Html,
    routing::get,
};
use metassr_build::server::renderer::page::PageRenderer;
use metassr_utils::analyzer::{
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
                RunningType::SSG => Box::new(read_to_string(entries.path.join("index.html"))?),
                RunningType::SSR => {
                    Box::new(PageRenderer::from_manifest(&self.dist_dir, route)?.render()?)
                }
            };

            let handler =
                move |Query(params): Query<HashMap<String, String>>,
                      Path(path): Path<HashMap<String, String>>| async move {
                    // dbg!(&params, &path);
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
