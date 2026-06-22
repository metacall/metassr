use anyhow::{anyhow, Result};
use dunce;

use metassr_fs_analyzer::dist_dir::{DistDirContainer, PageEntry};
use metassr_utils::cache_dir::CacheDir;

use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{read_to_string, File},
    io::Write,
    path::{Path, PathBuf},
};

use super::targets::Targets;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub id: i64,
    pub page_entry: PageEntry,
    pub renderer: PathBuf,
}

impl ManifestEntry {
    pub fn new(id: i64, page_entry: PageEntry, renderer: PathBuf) -> Self {
        Self {
            id,
            page_entry,
            renderer,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalEntry {
    pub head: PathBuf,
    pub cache: PathBuf,
}

impl GlobalEntry {
    pub fn new<H, C>(head: &H, cache: &C) -> Result<Self>
    where
        H: AsRef<OsStr> + ?Sized,
        C: AsRef<OsStr> + ?Sized,
    {
        Ok(Self {
            head: dunce::canonicalize(PathBuf::from(head))?,
            cache: PathBuf::from(cache),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub global: GlobalEntry,
    routes: HashMap<String, ManifestEntry>,
}

impl Manifest {
    pub fn new(global: GlobalEntry) -> Self {
        Self {
            global,
            routes: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        route: &str,
        id: i64,
        page_entry: &PageEntry,
        renderer: PathBuf,
    ) -> Option<ManifestEntry> {
        let entry = ManifestEntry::new(id, page_entry.clone(), renderer);
        self.routes.insert(route.to_string(), entry)
    }

    pub fn to_json(&self) -> Result<String> {
        let json = to_string_pretty(&self)?;
        Ok(json)
    }

    pub fn write<S: AsRef<OsStr> + ?Sized>(&self, path: &S) -> Result<PathBuf> {
        let manifest_filename = "manifest.json";
        let path = PathBuf::from(path);
        let mut file = File::create(path.join(manifest_filename))?;

        file.write_all(self.to_json()?.as_bytes())?;
        Ok(path)
    }
    pub fn get(&self, route: &str) -> Option<&ManifestEntry> {
        self.routes.get(route)
    }
}

impl<S: AsRef<OsStr> + ?Sized> From<&S> for Manifest {
    fn from(path: &S) -> Self {
        let manifest_filename = "manifest.json";
        let path = PathBuf::from(path).join(manifest_filename);
        let content = read_to_string(path).unwrap();

        serde_json::from_str(&content).unwrap()
    }
}

pub struct ManifestGenerator {
    targets: Targets,
    dist: DistDirContainer,
    cache: CacheDir,
    dist_path: PathBuf,
}

impl ManifestGenerator {
    pub fn new(
        targets: Targets,
        cache: CacheDir,
        dist: DistDirContainer,
        dist_path: PathBuf,
    ) -> Self {
        Self {
            targets,
            dist,
            cache,
            dist_path,
        }
    }
    pub fn generate<H: AsRef<OsStr> + ?Sized>(&self, head: &H) -> Result<Manifest> {
        let cache_path = self.cache.path();
        let global = GlobalEntry::new(head, cache_path)?;
        let mut manifest = Manifest::new(global);

        let cache_pages = cache_path.join("pages");
        for (path, &id) in self.targets.iter() {
            let rel = path.strip_prefix(&cache_pages)?;
            let route = match rel.parent().unwrap() {
                p if p == Path::new("") => "#root",
                p => p.to_str().unwrap(),
            };
            let route_key = if route == "#root" { "root" } else { route };

            // Point to the esbuild bundle output, not the source file
            let renderer = self
                .dist_path
                .join("server")
                .join("pages")
                .join(format!("{route_key}.js"));

            let page_entry = match self.dist.pages.get(route) {
                Some(e) => e,
                None => {
                    return Err(anyhow!("manifest: No entries found for: {:#?}", route));
                }
            };
            manifest.insert(route, id, page_entry, renderer);
        }
        Ok(manifest)
    }
}
