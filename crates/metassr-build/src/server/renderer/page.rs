use std::ffi::OsStr;

use anyhow::Result;
use metassr_utils::{cache_dir::CacheDir, dist_analyzer::PageEntry};

use crate::{
    server::{manifest::Manifest, render_exec::RenderExec},
    traits::Exec,
};

use super::{head::HeadRenderer, html::HtmlRenderer};

pub struct PageRenderer {
    head: String,
    body: String,
    entries: PageEntry,
}

impl PageRenderer {
    pub fn from_manifest<S: AsRef<OsStr> + ?Sized>(
        manifest_parent: &S,
        route: &str,
    ) -> Result<Self> {
        let manifest = Manifest::from(manifest_parent);

        let cache = CacheDir::new(&manifest.global.cache)?;
        let entry = manifest.get(route).unwrap().clone();

        let exec = RenderExec::new(entry.id, &entry.renderer)?;
        let body = exec.exec()?;
        let head = HeadRenderer::new(&manifest.global.head, cache).render()?;

        Ok(Self {
            head,
            body,
            entries: entry.page_entry,
        })
    }
    pub fn render(&self) -> Result<String> {
        Ok(HtmlRenderer::new(&self.head, &self.body, &self.entries)
            .render()?
            .to_string())
    }
}
