use anyhow::Result;
use axum::{
    extract::{Path, Query},
    response::Html,
    routing::get,
};
use metassr_utils::{
    dist_analyzer::{DistDir, PageEntry},
    traits::AnalyzeDir,
};
use std::{collections::HashMap, fs::read_to_string};

use super::router::RouterMut;

pub struct PagesHandler<'a> {
    pub app: &'a mut RouterMut,
    pub pages: HashMap<String, PageEntry>,
}

impl<'a> PagesHandler<'a> {
    pub fn new(app: &'a mut RouterMut, dist_dir: &str) -> Result<Self> {
        Ok(Self {
            app,
            pages: DistDir::new(&dist_dir)?.analyze()?.pages,
        })
    }
    pub fn build(&mut self) -> Result<()> {
        for (route, entries) in self.pages.iter() {
            let route = format!(
                "/{}",
                match route {
                    e if e == &String::from("root") => "".to_string(),
                    _ => route.replace('$', ":"),
                }
            );

            let path = entries.path.join("index.html");
            let html = Box::new(read_to_string(path)?);

            let handler =
                move |Query(params): Query<HashMap<String, String>>,
                      Path(path): Path<HashMap<String, String>>| async move {
                    dbg!(&params, &path);
                    Html(*html)
                };

            self.app.route(&route, get(handler));
        }
        Ok(())
    }
}
