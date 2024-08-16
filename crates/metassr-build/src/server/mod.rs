// TODO: Refactoring `ServerSideBuilder.build()`. It's very ugly!

pub mod renderer;

mod pages_generator;
mod render;
mod render_exec;
mod targets;

use crate::{
    bundler::{BundlingType, WebBundler},
    traits::{Build, Exec, Generate},
    utils::setup_page_path,
};
use metassr_utils::{
    cache_dir::CacheDir,
    src_analyzer::{special_entries, SourceDir},
    traits::AnalyzeDir,
};

use pages_generator::PagesGenerator;
use render::ServerRender;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};
use targets::TargetsGenerator;

use anyhow::{anyhow, Result};

pub struct ServerSideBuilder {
    src_path: PathBuf,
    dist_path: PathBuf,
}

impl ServerSideBuilder {
    pub fn new<S>(root: &S, dist_dir: &str) -> Result<Self>
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
        let (special_entries::App(app_path), special_entries::Head(head_path)) = src.specials()?;

        let targets = TargetsGenerator::new(app, pages, &mut cache_dir).generate()?;

        if let Err(e) = WebBundler::new(&targets.ready_for_bundling(), &self.dist_path).exec() {
            return Err(anyhow!("Bundling failed: {e}"));
        }

        // TODO: remove this
        sleep(Duration::from_secs(3));

        PagesGenerator::new(targets, &head_path, &self.dist_path, cache_dir)?.generate()?;
        Ok(())
    }
}
