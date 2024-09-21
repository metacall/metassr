//! Manifest generation and management for the `metassr` framework.
//!
//! This module defines the structures and functions for generating, serializing, and managing
//! the `manifest.json` file, which maps routes to their associated page entries and renderers
//! in the SSR (server-side rendering) process.

use anyhow::{anyhow, Result};
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

/// Represents a single entry in the manifest for a specific route.
///
/// This struct contains details about the page's assets (scripts and styles)
/// and the path to the server-side renderer responsible for rendering the page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestRouteEntry {
    /// Unique identifier for the entry (usually a hash or numeric ID).
    pub id: i64,
    /// The page entry containing scripts, styles, and path for the route's static assets.
    pub page_entry: PageEntry,
    /// Path to the server-side renderer script for this route.
    pub renderer: PathBuf,
}

impl ManifestRouteEntry {
    /// Creates a new `ManifestRouteEntry`.
    ///
    /// # Parameters
    /// - `id`: Unique identifier for the entry.
    /// - `page_entry`: Information about the page's static assets.
    /// - `renderer`: Path to the server-side renderer.
    ///
    /// # Returns
    /// A new `ManifestRouteEntry` instance.
    pub fn new(id: i64, page_entry: PageEntry, renderer: PathBuf) -> Self {
        Self {
            id,
            page_entry,
            renderer,
        }
    }
}

/// Holds global information for the manifest, such as the HTML `<head>` file and the cache directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalEntry {
    /// Path to the global HTML `<head>` file.
    pub head: PathBuf,
    /// Path to the cache directory where SSR renderers are stored.
    pub cache: PathBuf,
}

impl GlobalEntry {
    /// Creates a new `GlobalEntry`.
    ///
    /// # Parameters
    /// - `head`: Path to the global `<head>` file.
    /// - `cache`: Path to the cache directory.
    ///
    /// # Returns
    /// A new `GlobalEntry` instance.
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

/// The manifest that maps routes to their respective page entries and SSR renderers.
///
/// It also contains a `GlobalEntry` which stores information shared across all routes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// The global entry shared across all routes.
    pub global: GlobalEntry,
    /// A map of route paths to `ManifestRouteEntry` objects.
    routes: HashMap<String, ManifestRouteEntry>,
}

impl Manifest {
    /// Creates a new empty `Manifest` with a given global entry.
    ///
    /// **Parameters**
    /// - `global`: The global entry (head and cache paths).
    ///
    /// **Returns**
    /// A new `Manifest` instance with no routes.
    pub fn new(global: GlobalEntry) -> Self {
        Self {
            global,
            routes: HashMap::new(),
        }
    }

    /// Inserts a new route into the manifest.
    ///
    /// **Parameters**
    /// - `route`: The route string (e.g., "blog", "home", "#root").
    /// - `id`: The unique identifier for the page entry.
    /// - `page_entry`: A reference to the `PageEntry` containing static asset information.
    /// - `renderer`: The path to the server-side renderer for this route.
    ///
    /// **Returns**
    /// Optionally returns the previous `ManifestRouteEntry` if the route already existed.
    pub fn insert(
        &mut self,
        route: &str,
        id: i64,
        page_entry: &PageEntry,
        renderer: PathBuf,
    ) -> Option<ManifestRouteEntry> {
        let entry = ManifestRouteEntry::new(id, page_entry.clone(), renderer);
        self.routes.insert(route.to_string(), entry)
    }

    /// Serializes the manifest into a pretty-printed JSON string.
    ///
    /// **Returns**
    /// A `Result` containing the JSON string or an error if serialization fails.
    pub fn to_json(&self) -> Result<String> {
        let json = to_string_pretty(&self)?;
        Ok(json)
    }

    /// Writes the manifest to a file as `manifest.json`.
    ///
    /// **Parameters**
    /// - `path`: The path where the manifest file should be written.
    ///
    /// **Returns**
    /// A `Result` containing the path to the manifest file or an error if writing fails.
    pub fn write<S: AsRef<OsStr> + ?Sized>(&self, path: &S) -> Result<PathBuf> {
        let manifest_filename = "manifest.json";
        let path = PathBuf::from(path);
        let mut file = File::create(path.join(manifest_filename))?;

        file.write_all(self.to_json()?.as_bytes())?;
        Ok(path)
    }

    /// Retrieves a `ManifestRouteEntry` for a given route.
    ///
    /// **Parameters**
    /// - `route`: The route string.
    ///
    /// **Returns**
    /// An `Option` containing a reference to the `ManifestRouteEntry` if it exists.
    pub fn get(&self, route: &str) -> Option<&ManifestRouteEntry> {
        self.routes.get(route)
    }
}

impl<S: AsRef<OsStr> + ?Sized> From<&S> for Manifest {
    /// Loads a `Manifest` from a JSON file located at the specified path.
    ///
    /// **Parameters**
    /// - `path`: The directory where `manifest.json` is located.
    ///
    /// **Returns**
    /// A `Manifest` instance deserialized from the file.
    fn from(path: &S) -> Self {
        let manifest_filename = "manifest.json";
        let path = PathBuf::from(path).join(manifest_filename);
        let content = read_to_string(path).unwrap();

        serde_json::from_str(&content).unwrap()
    }
}

/// Generates a `Manifest` by analyzing the distribution and cache directories.
///
/// This struct is responsible for matching routes with their corresponding page assets
/// and SSR renderers.
pub struct ManifestGenerator {
    targets: Targets,
    dist: DistDirContainer,
    cache: CacheDir,
}

impl ManifestGenerator {
    /// Creates a new `ManifestGenerator`.
    ///
    /// **Parameters**
    /// - `targets`: A collection of route targets and their identifiers.
    /// - `cache`: The cache directory containing SSR renderers.
    /// - `dist`: The distribution directory containing page assets.
    ///
    /// **Returns**
    /// A new `ManifestGenerator` instance.
    pub fn new(targets: Targets, cache: CacheDir, dist: DistDirContainer) -> Self {
        Self {
            targets,
            dist,
            cache,
        }
    }

    /// Generates the manifest by iterating over the route targets, matching them with
    /// their assets in the distribution directory, and assigning SSR renderers from the cache.
    ///
    /// **Parameters**
    /// - `head`: The path to the global HTML `<head>` file.
    ///
    /// **Returns**
    /// A `Result` containing the generated `Manifest` or an error if generation fails.
    pub fn generate<H: AsRef<OsStr> + ?Sized>(&self, head: &H) -> Result<Manifest> {
        let cache_path = self.cache.path();
        let global = GlobalEntry::new(head, cache_path)?;
        let mut manifest = Manifest::new(global);

        for (path, &id) in self.targets.iter() {
            // Derive route name from the path.
            // Example:
            // - Input:  "dist/cache/pages/blog/$article/index.server.js"
            // - Output: "blog/$article"
            //
            // If the parent of the stripped path is an empty string, the route will be "#root"
            let route = match path
                .strip_prefix(cache_path.join("pages"))?
                .parent()
                .unwrap()
            {
                p if p == Path::new("") => "#root",
                p => p.to_str().unwrap(),
            };

            let page_entry = match self.dist.pages.get(route) {
                Some(e) => e,
                None => {
                    return Err(anyhow!("manifest: No entries found for: {:#?}", route));
                }
            };
            manifest.insert(route, id, page_entry, path.canonicalize()?);
        }
        Ok(manifest)
    }
}
