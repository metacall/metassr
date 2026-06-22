use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use metassr_fs_analyzer::{
    dist_dir::{DistDir, DistDirContainer},
    DirectoryAnalyzer,
};
use metassr_utils::cache_dir::CacheDir;

use crate::traits::Exec;

use super::{
    render_exec::MultiRenderExec, renderer::head::HeadRenderer, renderer::html::HtmlRenderer,
    targets::Targets,
};

pub struct PagesGenerator {
    server_pages_dir: PathBuf,
    dist: DistDirContainer,
    head: String,
    output: HashMap<String, String>,
}

impl PagesGenerator {
    pub fn new<S: AsRef<OsStr> + ?Sized>(
        targets: Targets,
        head_path: &S,
        dist_path: &S,
        cache_dir: CacheDir,
        dev_mode: bool,
    ) -> Result<Self> {
        let dist_path_buf = PathBuf::from(dist_path.as_ref());
        let dist = DistDir::new(dist_path)?.analyze()?;
        let head = HeadRenderer::new(&head_path, cache_dir.clone(), dev_mode).render(false)?;
        let server_pages_dir = dist_path_buf.join("server").join("pages");

        let output = MultiRenderExec::new(targets.ready_for_exec(&dist_path_buf))?.exec()?;

        Ok(Self {
            dist,
            head,
            server_pages_dir,
            output,
        })
    }

    pub fn generate(&self) -> Result<()> {
        for (path, html_body) in &self.output {
            // Bundle path is e.g. dist/server/pages/home.js
            // File stem is the route key: "home", "root", "blog/article"
            let path = Path::new(&path);
            let rel = path.strip_prefix(&self.server_pages_dir)?;
            let route_key = rel.with_extension("");
            let route = match route_key.to_str().unwrap() {
                "root" => "#root",
                r => r,
            };

            let page_entry = self.dist.pages.get(route);
            match page_entry {
                Some(page_entry) => {
                    // dbg!(&path.join("index.html"));
                    HtmlRenderer::new(&self.head, html_body, page_entry)
                        .render()?
                        .write(page_entry.path.join("index.html"))?;
                }
                None => {
                    return Err(anyhow!(
                        "ssg: No entries found for this page: route = {route:#?}, path = {path:#?}",
                    ))
                }
            }
        }
        Ok(())
    }
}
