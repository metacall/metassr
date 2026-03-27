pub mod renderer;

pub mod manifest;
mod pages_generator;
mod render;
mod render_exec;
pub mod targets;

use crate::traits::Build;
use manifest::ManifestGenerator;

use metassr_bundler::WebBundler;
use metassr_fs_analyzer::{
    dist_dir::DistDir,
    src_dir::{special_entries, SourceDir},
    DirectoryAnalyzer,
};
use metassr_utils::cache_dir::CacheDir;

use pages_generator::PagesGenerator;
use renderer::head::HeadRenderer;

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use targets::{Targets, TargetsGenerator};

use anyhow::{anyhow, Result};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BuildingType {
    ServerSideRendering,
    StaticSiteGeneration,
}

pub struct ServerSideBuilder {
    src_path: PathBuf,
    dist_path: PathBuf,
    building_type: BuildingType,
    dev_mode: bool,
}

impl ServerSideBuilder {
    pub fn new<S>(
        root: &S,
        dist_dir: &str,
        building_type: BuildingType,
        dev_mode: bool,
    ) -> Result<Self>
    where
        S: AsRef<OsStr> + ?Sized,
    {
        let root = Path::new(root);
        let src_path = root.join("src");
        let dist_path = root.join(dist_dir);

        if !src_path.exists() {
            return Err(anyhow!("src directory not found."));
        }
        if !dist_path.exists() {
            fs::create_dir(dist_path.clone())?;
        }
        Ok(Self {
            src_path,
            dist_path,
            building_type,
            dev_mode,
        })
    }
}

/// State produced by target generation, needed for post-bundle processing.
pub struct ServerBuildState {
    pub bundling_targets: HashMap<String, String>,
    pub targets: Targets,
    pub cache_dir: CacheDir,
    pub head: PathBuf,
}

impl ServerSideBuilder {
    /// Generates server-side render scripts (including head) and returns bundling targets
    /// without running the bundler. Use with `WebBundler` to combine with client targets.
    pub fn generate_targets(&self) -> Result<ServerBuildState> {
        let mut cache_dir = CacheDir::new(&format!("{}/cache", self.dist_path.display()))?;

        let src = SourceDir::new(&self.src_path).analyze()?;
        let pages = src.clone().pages;
        let (special_entries::App(app), special_entries::Head(head)) = src.specials()?;

        let targets = match TargetsGenerator::new(app, pages, &mut cache_dir).generate() {
            Ok(t) => t,
            Err(e) => return Err(anyhow!("Couldn't generate targets: {e}")),
        };

        let mut bundling_targets = targets.ready_for_bundling(&self.dist_path);

        // Include head component target so it gets bundled in the same rspack call.
        // Entry name "cache/head" outputs to dist/cache/head.js.
        let head_targets = HeadRenderer::generate_target(&head, &mut cache_dir)?;
        bundling_targets.extend(head_targets);

        Ok(ServerBuildState {
            bundling_targets,
            targets,
            cache_dir,
            head,
        })
    }

    /// Runs post-bundle processing: manifest generation, head loading, and SSG pages.
    /// Head is already bundled by the combined build, so we just load it (bundler=false).
    pub fn finish_build(&self, state: ServerBuildState) -> Result<()> {
        let dist = DistDir::new(&self.dist_path)?.analyze()?;

        let manifest = ManifestGenerator::new(
            state.targets.clone(),
            state.cache_dir.clone(),
            dist,
            self.dist_path.clone(),
        )
        .generate(&state.head)?;
        manifest.write(&self.dist_path.clone())?;

        // Head was already bundled in the combined rspack call, just load it
        if let Err(e) = HeadRenderer::new(
            &manifest.global.head,
            state.cache_dir.clone(),
            self.dev_mode,
        )
        .render(false)
        {
            return Err(anyhow!("Couldn't render head: {e}"));
        }

        if self.building_type == BuildingType::StaticSiteGeneration {
            if let Err(e) = PagesGenerator::new(
                state.targets,
                &state.head,
                &self.dist_path,
                state.cache_dir,
                self.dev_mode,
            )?
            .generate()
            {
                return Err(anyhow!("Couldn't generate pages: {e}"));
            }
        }
        Ok(())
    }
}

impl Build for ServerSideBuilder {
    type Output = ();
    fn build(&self) -> Result<Self::Output> {
        let state = self.generate_targets()?;

        let bundler = WebBundler::new(&state.bundling_targets, &self.dist_path, self.dev_mode)?;
        if let Err(e) = bundler.exec() {
            return Err(anyhow!("Bundling failed: {e}"));
        }

        self.finish_build(state)
    }
}
