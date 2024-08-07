// TODO: Refactoring `ServerSideBuilder.build()`. It's very ugly!

mod head_renderer;
mod html_renderer;
mod render;
mod render_exec;

use crate::{
    bundler::{BundlingType, WebBundler},
    traits::{Build, Exec, Generate},
};
use metassr_utils::{
    cache_dir::CacheDir,
    dist_analyzer::DistDir,
    src_analyzer::{special_entries, SourceDir},
    traits::AnalyzeDir,
};

use html_renderer::HtmlRenderer;
use head_renderer::HeadRenderer;
use render::ServerRender;
use render_exec::MultiRenderExec;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};

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
        dbg!(&src);

        let mut targets = HashMap::<i64, String>::new();
        let mut bundling_targets = HashMap::<String, String>::new();

        for (page, page_path) in pages.iter() {
            let (func_id, render_script) = ServerRender::new(&app_path, page_path).generate()?;

            // Page details
            let mut page = match Path::new(page) {
                path if path.file_stem() != Some(OsStr::new("index")) => {
                    let mut path = path.to_path_buf();
                    path.set_extension("");
                    path.join("index.server.js")
                }

                path => {
                    let mut path = path.to_path_buf();
                    path.set_extension("server.js");
                    path
                }
            };

            // TODO: refactor this part
            let cached_pages = cache_dir.dir_path().join("pages");

            let pathname = cache_dir.insert(
                PathBuf::from("pages").join(&page).to_str().unwrap(),
                render_script.as_bytes(),
            )?;
            page.set_extension("");

            targets.insert(func_id, pathname.clone());
            bundling_targets.insert(
                cached_pages
                    .strip_prefix("dist")?
                    .join(&page)
                    .to_str()
                    .unwrap()
                    .to_owned(),
                PathBuf::from(&pathname)
                    .canonicalize()?
                    .to_str()
                    .unwrap()
                    .to_owned(),
            );
        }

        let bundler = WebBundler::new(&bundling_targets, &self.dist_path, BundlingType::Library);
        // dbg!(&bundler, &targets);
        if let Err(e) = bundler.exec() {
            return Err(anyhow!("Bundling failed: {e}"));
        }
        sleep(Duration::from_secs(3));

        let output = MultiRenderExec::new(targets)?.exec()?;
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
