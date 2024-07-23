pub mod render;
pub mod render_exec;

use crate::traits::{Build, Exec, Generate};
use html_generator::{builder::HtmlBuilder, html_props::HtmlProps, template::HtmlTemplate};
use metassr_utils::{
    cache_dir::CacheDir, dist_analyzer::DistDir, src_analyzer::SourceDir, traits::AnalyzeDir,
};
use render::ServerRender;
use render_exec::MultiRenderExec;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};

pub struct ServerSideBuilder {
    src_path: PathBuf,
    dist_path: PathBuf,
}

impl ServerSideBuilder {
    pub fn new<'a, S>(root: &'a S, dist_dir: &str) -> Result<Self>
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
        let app_path = src
            .specials
            .get("_app")
            .expect("Error: Cannot detect '_app' in src directory")
            .as_ref()
            .unwrap();

        let mut targets = HashMap::<i64, String>::new();

        for (page, page_path) in pages.iter() {
            let (func_id, render_script) = ServerRender::new(&app_path, &page_path).generate()?;

            // Page details
            let page = match Path::new(page) {
                path if path.file_stem() != Some(OsStr::new("index")) => {
                    let mut path = path.to_path_buf();
                    path.set_extension("");
                    path.join("index.server.tsx")
                }

                path => {
                    let mut path = path.to_path_buf();
                    path.set_extension("server.tsx");
                    path
                }
            };

            let pathname = cache_dir.insert(
                &format!("pages/{}", page.display()),
                render_script.as_bytes(),
            )?;
            targets.insert(func_id, pathname);
        }

        let output = MultiRenderExec::new(targets)?.exec()?;
        let dist_analyst = DistDir::new(&self.dist_path)?.analyze()?;
        // dbg!(&output);
        dbg!(&dist_analyst);

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
                Some(p) => {
                    let scripts: Vec<String> = p
                        .scripts
                        .iter()
                        .map(|p| p.to_str().unwrap().to_owned())
                        .collect();

                    let styles: Vec<String> = p
                        .styles
                        .iter()
                        .map(|p| p.to_str().unwrap().to_owned())
                        .collect();

                    let html_props = HtmlProps::new()
                        // TODO:  Get head content from `_head.tsx`
                        .head("")
                        .body(&html_body)
                        .lang("en")
                        .scripts(scripts)
                        .styles(styles);

                    let builder = HtmlBuilder::new(HtmlTemplate::default(), html_props.build());

                    builder.generate().write(p.path.join("index.html"))?;
                }
                None => return Err(anyhow!("No Entries founded for this page: {:#?}", path)),
            }
        }

        Ok(())
    }
}
