use anyhow::Result;
use axum::{
    extract::{Path, Query},
    response::Html,
    routing::get,
};
use metassr_build::server::renderer::page::PageRenderer;
use metassr_utils::{
    dist_analyzer::{DistDir, PageEntry},
    traits::AnalyzeDir,
};
use std::{collections::HashMap, path::PathBuf};

use super::router::RouterMut;

pub struct PagesHandler<'a> {
    pub app: &'a mut RouterMut,
    pub pages: HashMap<String, PageEntry>,
    pub dist_dir: PathBuf,
}

impl<'a> PagesHandler<'a> {
    pub fn new(app: &'a mut RouterMut, dist_dir: &str) -> Result<Self> {
        Ok(Self {
            app,
            pages: DistDir::new(&dist_dir)?.analyze()?.pages,
            dist_dir: PathBuf::from(dist_dir),
        })
    }
    pub fn build(&mut self) -> Result<()> {
        for (route, _) in self.pages.iter() {
            let html = Box::new(PageRenderer::from_manifest(&self.dist_dir, route)?.render()?);

            let handler =
                move |Query(params): Query<HashMap<String, String>>,
                      Path(path): Path<HashMap<String, String>>| async move {
                    dbg!(&params, &path);
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
