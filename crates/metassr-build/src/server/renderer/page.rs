use std::{collections::HashMap, ffi::OsStr};

use anyhow::Result;
use serde_json::json;

use metassr_fs_analyzer::dist_dir::PageEntry;
use metassr_utils::cache_dir::CacheDir;

use crate::server::{manifest::Manifest, render_exec::RenderExec};

use super::{head::HeadRenderer, html::HtmlRenderer};

pub struct PageRenderer {
    head: String,
    exec: RenderExec,
    entries: PageEntry,
}

impl PageRenderer {
    /// Loads the page render script once. Pair with [`render`](Self::render) to
    /// re-render the page on every request.
    pub fn from_manifest<S: AsRef<OsStr> + ?Sized>(
        manifest_parent: &S,
        route: &str,
    ) -> Result<Self> {
        let manifest = Manifest::from(manifest_parent);

        let cache = CacheDir::new(&manifest.global.cache)?;
        let entry = manifest.get(route).unwrap().clone();

        let exec = RenderExec::new(entry.id, &entry.renderer)?;
        let head = HeadRenderer::new(&manifest.global.head, cache, false).render(false)?;

        Ok(Self {
            head,
            exec,
            entries: entry.page_entry,
        })
    }
    pub fn render(&self) -> Result<String> {
        let body = self.exec.call()?;
        Ok(HtmlRenderer::new(&self.head, &body, &self.entries)
            .render()?
            .to_string())
    }

    pub fn render_with_context(
        &self,
        params: &HashMap<String, String>,
        query: &HashMap<String, String>,
    ) -> Result<String> {
        let ctx = json!({
            "params": params,
            "query": query,
        });
        let ctx_json = serde_json::to_string(&ctx)?;
        let body = self.exec.call_with_context(&ctx_json)?;
        Ok(HtmlRenderer::new(&self.head, &body, &self.entries)
            .with_props(ctx_json)
            .render()?
            .to_string())
    }
}
