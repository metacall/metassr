use anyhow::{anyhow, Result};
use metassr_utils::{
    analyzer::dist_dir::{DistDirContainer, PageEntry},
    cache_dir::CacheDir,
};
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
            head: PathBuf::from(head).canonicalize()?,
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
}

impl ManifestGenerator {
    pub fn new(targets: Targets, cache: CacheDir, dist: DistDirContainer) -> Self {
        Self {
            targets,
            dist,
            cache,
        }
    }
    pub fn generate<H: AsRef<OsStr> + ?Sized>(&self, head: &H) -> Result<Manifest> {
        let global = GlobalEntry::new(head, &self.cache.dir_path())?;
        let mut manifest = Manifest::new(global);

        for (path, &id) in self.targets.iter() {
            let route = match path
                .strip_prefix(self.cache.dir_path().join("pages"))?
                .parent()
                .unwrap()
            {
                p if p == Path::new("") => "#root",
                p => p.to_str().unwrap(),
            };

            let page_entry = match self.dist.pages.get(route) {
                Some(e) => e,
                None => {
                    return Err(anyhow!("manifest: No Entries founded for: {:#?}", route));
                }
            };
            manifest.insert(route, id, page_entry, path.canonicalize()?);
            // dbg!(&route, &page_entry);
        }
        Ok(manifest)
    }
}
