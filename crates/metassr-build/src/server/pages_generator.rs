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
    head_renderer::HeadRenderer, html_renderer::HtmlRenderer, render_exec::MultiRenderExec,
    targets::Targets,
};

pub struct PagesGenerator {
    cached_pages: PathBuf,
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
        let head = HeadRenderer::new(&head_path, cache_dir.clone()).render()?;
        let cached_pages = cache_dir.dir_path().join("pages");

        let output = MultiRenderExec::new(targets.ready_for_exec())?.exec()?;

        Ok(Self {
            dist,
            head,
            cached_pages,
            output,
        })
    }

    pub fn generate(&self) -> Result<()> {
        for (path, html_body) in &self.output {
            let path = Path::new(&path).parent().unwrap();
            let path_str = match path
                .strip_prefix(self.cached_pages.to_str().unwrap())?
                .to_str()
                .unwrap()
            {
                "" => "root",
                p => p,
            };

            let page_entry = self.dist.pages.get(path_str);
            match page_entry {
                Some(page_entry) => {
                    HtmlRenderer::new(&self.head, html_body, page_entry).render()?;
                }
                None => return Err(anyhow!("No Entries founded for this page: {:#?}", path)),
            }
        }
        Ok(())
    }
}
