// TODO: Refactoring `ServerSideBuilder.build()`. It's very ugly!

mod head_renderer;
mod html_renderer;
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
    dist_analyzer::DistDir,
    src_analyzer::{special_entries, SourceDir},
    traits::AnalyzeDir,
};

use head_renderer::HeadRenderer;
use html_renderer::HtmlRenderer;
use render::ServerRender;
use render_exec::MultiRenderExec;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};
use targets::Targets;

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

        let mut targets = Targets::new();

        for (page, page_path) in pages.iter() {
            let (func_id, render_script) = ServerRender::new(&app_path, page_path).generate()?;

            let page = setup_page_path(page, "server.js");
            let path = cache_dir.insert(
                PathBuf::from("pages").join(&page).to_str().unwrap(),
                render_script.as_bytes(),
            )?;

            targets.insert(func_id, &path);
        }

        if let Err(e) = WebBundler::new(
            &targets.ready_for_bundling(),
            &self.dist_path,
            BundlingType::Library,
        )
        .exec()
        {
            return Err(anyhow!("Bundling failed: {e}"));
        }
        // TODO: remove this
        sleep(Duration::from_secs(3));

        let output = MultiRenderExec::new(targets.ready_for_exec())?.exec()?;
        let dist_analyst = DistDir::new(&self.dist_path)?.analyze()?;

        // dbg!(&dist_analyst);

        let head_content = HeadRenderer::new(&head_path, cache_dir.clone()).render()?;

        let cached_pages = cache_dir.dir_path().join("pages");
        for (path, html_body) in output {
            let path = Path::new(&path).parent().unwrap();
            let path_str = match path
                .strip_prefix(cached_pages.to_str().unwrap())?
                .to_str()
                .unwrap()
            {
                "" => "root",
                p => p,
            };

            let page_entry = dist_analyst.pages.get(path_str);
            match page_entry {
                Some(page_entry) => {
                    HtmlRenderer::new(&head_content, &html_body, page_entry).render()?;
                }
                None => return Err(anyhow!("No Entries founded for this page: {:#?}", path)),
            }
        }

        Ok(())
    }
}
