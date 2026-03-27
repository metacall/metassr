use std::{
    collections::{hash_map::Iter, HashMap},
    path::{Path, PathBuf},
};

use anyhow::Result;
use dunce;

use metassr_fs_analyzer::src_dir::PagesEntriesType;
use metassr_utils::{cache_dir::CacheDir, js_path::to_js_path};

use crate::{traits::Generate, utils::setup_page_path};

use super::render::ServerRender;

#[derive(Debug, Clone)]
pub struct Targets(HashMap<PathBuf, i64>);

impl Targets {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, func_id: i64, path: &Path) {
        self.0.insert(path.to_path_buf(), func_id);
    }

    /// Returns bundling targets with entry names that output to `dist/server/pages/`.
    /// e.g. source `dist/cache/pages/home/index.server.js` => entry `server/pages/home`
    ///      => esbuild output `dist/server/pages/home.js` (no collision with source)
    pub fn ready_for_bundling(&self, dist_path: &Path) -> HashMap<String, String> {
        let cache_pages = dist_path.join("cache").join("pages");
        self.0
            .keys()
            .map(|path| {
                let route = self.route_from_source(path, &cache_pages);
                let entry = format!("server/pages/{route}");
                (entry, to_js_path(&dunce::canonicalize(path).unwrap()))
            })
            .collect()
    }

    /// Returns the expected bundle output paths for execution (SSG/SSR).
    /// e.g. route `home` => `dist/server/pages/home.js`
    pub fn ready_for_exec(&self, dist_path: &Path) -> HashMap<String, i64> {
        let cache_pages = dist_path.join("cache").join("pages");
        self.0
            .iter()
            .map(|(path, &id)| {
                let route = self.route_from_source(path, &cache_pages);
                let bundle = dist_path
                    .join("server")
                    .join("pages")
                    .join(format!("{route}.js"));
                (bundle.to_str().unwrap().to_string(), id)
            })
            .collect()
    }

    fn route_from_source(&self, source: &Path, cache_pages: &Path) -> String {
        let rel = source.strip_prefix(cache_pages).unwrap();
        match rel.parent().unwrap() {
            p if p == Path::new("") => "root".to_string(),
            p => p.to_str().unwrap().to_string(),
        }
    }

    pub fn iter(&self) -> Iter<'_, PathBuf, i64> {
        self.0.iter()
    }
}

impl Default for Targets {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TargetsGenerator<'a> {
    app: PathBuf,
    pages: PagesEntriesType,
    cache: &'a mut CacheDir,
}

impl<'a> TargetsGenerator<'a> {
    pub fn new(app: PathBuf, pages: PagesEntriesType, cache: &'a mut CacheDir) -> Self {
        Self { app, pages, cache }
    }
    pub fn generate(&mut self) -> Result<Targets> {
        let mut targets = Targets::new();
        for (page, page_path) in self.pages.iter() {
            let (func_id, render_script) = ServerRender::new(&self.app, page_path).generate()?;

            let page = setup_page_path(page, "server.js");
            let path = self.cache.insert(
                PathBuf::from("pages").join(&page).to_str().unwrap(),
                render_script.as_bytes(),
            )?;

            targets.insert(func_id, &path);
        }
        Ok(targets)
    }
}
