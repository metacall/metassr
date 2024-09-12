pub mod renderer;

pub mod manifest;
mod pages_generator;
mod render;
mod render_exec;
mod targets;

use crate::traits::Build;
use manifest::ManifestGenerator;

use metassr_bundler::WebBundler;
use metassr_utils::{
    cache_dir::CacheDir,
    dist_analyzer::DistDir,
    src_analyzer::{special_entries, SourceDir},
    traits::AnalyzeDir,
};
use pages_generator::PagesGenerator;
use renderer::head::HeadRenderer;

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use targets::TargetsGenerator;

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
}

impl ServerSideBuilder {
    pub fn new<S>(root: &S, dist_dir: &str, building_type: BuildingType) -> Result<Self>
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
        })
    }
}
// TODO: refactoring build function
impl Build for ServerSideBuilder {
    type Output = ();
    fn build(&self) -> Result<Self::Output> {
        let mut cache_dir = CacheDir::new(&format!("{}/cache", self.dist_path.display()))?;

        let src = SourceDir::new(&self.src_path).analyze()?;
        let pages = src.clone().pages;
        let (special_entries::App(app), special_entries::Head(head)) = src.specials()?;

        let targets = match TargetsGenerator::new(app, pages, &mut cache_dir).generate() {
            Ok(t) => t,
            Err(e) => return Err(anyhow!("Couldn't generate targets: {e}")),
        };

        if let Err(e) = WebBundler::new(
            &targets.ready_for_bundling(&self.dist_path),
            &self.dist_path,
        )
        .exec()
        {
            return Err(anyhow!("Bundling failed: {e}"));
        }

        let dist = DistDir::new(&self.dist_path)?.analyze()?;

        let manifest =
            ManifestGenerator::new(targets.clone(), cache_dir.clone(), dist).generate(&head)?;
        manifest.write(&self.dist_path.clone())?;

        if let Err(e) = HeadRenderer::new(&manifest.global.head, cache_dir.clone()).render(true) {
            return Err(anyhow!("Coludn't render head: {e}"));
        }

        if self.building_type == BuildingType::StaticSiteGeneration {
            if let Err(e) =
                PagesGenerator::new(targets, &head, &self.dist_path, cache_dir)?.generate()
            {
                return Err(anyhow!("Couldn't generate pages: {e}"));
            }
        }
        Ok(())
    }
}
