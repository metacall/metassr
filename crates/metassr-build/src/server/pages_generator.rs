use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use metassr_utils::{
    cache_dir::CacheDir,
    dist_analyzer::{DistDir, DistDirContainer},
    traits::AnalyzeDir,
};

use crate::traits::Exec;

use super::{
    render_exec::MultiRenderExec, renderer::head::HeadRenderer, renderer::html::HtmlRenderer,
    targets::Targets,
};

pub struct PagesGenerator {
    cache: PathBuf,
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
    ) -> Result<Self> {
        let dist = DistDir::new(dist_path)?.analyze()?;
        let head = HeadRenderer::new(&head_path, cache_dir.clone()).render(true)?;
        let cache = cache_dir.dir_path();

        let output = MultiRenderExec::new(targets.ready_for_exec())?.exec()?;

        Ok(Self {
            dist,
            head,
            cache,
            output,
        })
    }

    pub fn generate(&self) -> Result<()> {
        for (path, html_body) in &self.output {
            let path = Path::new(&path).parent().unwrap();
            // dbg!(&path, &self.cache.join(""));
            let route = match path.strip_prefix(self.cache.join("pages"))? {
                p if p == Path::new("") => "#root",
                p => p.to_str().unwrap(),
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
                    "ssg: No Entries founded for this page: route = {route:#?}, path = {path:#?}",
                ))
                }
            }
        }
        Ok(())
    }
}
