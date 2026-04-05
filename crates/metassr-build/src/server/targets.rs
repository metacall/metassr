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

    pub fn ready_for_bundling(&self, dist_path: &PathBuf) -> HashMap<String, String> {
        self.0
            .keys()
            .map(|path| {
                let mut name = match path.strip_prefix(dist_path) {
                    Ok(p) => p,
                    Err(e) => panic!(
                        "Couldn't \"{}\".strip_prefix(\"{}\"): {e}",
                        dist_path.display(),
                        path.display()
                    ),
                }
                .to_path_buf();
                name.set_extension("");
                (
                    to_js_path(&name),
                    to_js_path(&dunce::canonicalize(path).unwrap()),
                )
            })
            .collect()
    }

    pub fn ready_for_exec(&self) -> HashMap<String, i64> {
        self.0
            .iter()
            .map(|(path, &id)| (path.to_str().unwrap().to_string(), id))
            .collect()
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
