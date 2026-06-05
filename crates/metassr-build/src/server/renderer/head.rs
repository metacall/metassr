use anyhow::{anyhow, Result};
use dunce;
use lazy_static::lazy_static;
use metacall::{load, metacall_no_arg};
use metassr_utils::{cache_dir::CacheDir, checker::CheckerState, js_path::to_js_path};
use std::{collections::HashMap, ffi::OsStr, path::PathBuf, sync::Mutex};

use metassr_bundler::WebBundler;

lazy_static! {
    static ref IS_HEAD_SCRIPT_LOADED: Mutex<CheckerState> = Mutex::new(CheckerState::default());
}

pub struct HeadRenderer {
    path: PathBuf,
    cache_dir: CacheDir,
    dev_mode: bool,
}

impl HeadRenderer {
    pub fn new<S>(path: &S, cache_dir: CacheDir, dev_mode: bool) -> Self
    where
        S: AsRef<OsStr> + ?Sized,
    {
        Self {
            path: PathBuf::from(path),
            cache_dir,
            dev_mode,
        }
    }

    pub fn render(&mut self, bundler: bool) -> Result<String> {
        let mut guard = IS_HEAD_SCRIPT_LOADED.lock().unwrap();
        if !guard.is_true() {
            if bundler {
                self.bundle()?;
            }

            // Load the bundled head from dist/server/head.js (esbuild output location)
            let bundle_path = self
                .cache_dir
                .path()
                .parent()
                .ok_or_else(|| anyhow!("Cannot resolve dist path from cache dir"))?
                .join("server")
                .join("head.js");

            let _ = load::from_single_file(load::Tag::NodeJS, &bundle_path, None);
            guard.make_true()
        }
        drop(guard);

        match metacall_no_arg::<String>("render_head") {
            Err(e) => Err(anyhow!("Couldn't render head: {e:?}")),
            Ok(out) => Ok(out),
        }
    }

    fn bundle(&mut self) -> Result<()> {
        let bundling_targets = self.bundling_target()?;
        let bundler = WebBundler::new(&bundling_targets, self.cache_dir.path(), self.dev_mode)?;

        if let Err(e) = bundler.exec() {
            return Err(anyhow!("Cannot bundling head: {e}"));
        }
        Ok(())
    }

    fn script(&self) -> Result<String> {
        let script = format!(
            r#"
import Head from "{}"
import {{ renderToString }} from "react-dom/server"
import React from "react"

export function render_head() {{
    return renderToString(<Head />);
}}

                "#,
            to_js_path(&dunce::canonicalize(&self.path)?)
        );
        Ok(script)
    }

    fn bundling_target(&mut self) -> Result<HashMap<String, String>> {
        let path = self
            .cache_dir
            .insert("head.js", self.script()?.as_bytes())?;
        let name = PathBuf::from(path.clone().file_name().unwrap())
            .with_extension("")
            .to_str()
            .unwrap()
            .to_string();
        let fullpath = to_js_path(&dunce::canonicalize(&path)?);

        Ok(HashMap::from([(name, fullpath)]))
    }

    /// Generates the head bundling target for inclusion in a combined build.
    /// The returned entry uses a `cache/` prefix so it outputs to `dist/cache/head.js`
    /// when bundled with the main dist output directory.
    pub fn generate_target(
        head_path: &PathBuf,
        cache_dir: &mut CacheDir,
    ) -> Result<HashMap<String, String>> {
        let script = format!(
            r#"
import Head from "{}"
import {{ renderToString }} from "react-dom/server"
import React from "react"

export function render_head() {{
    return renderToString(<Head />);
}}
                "#,
            to_js_path(&dunce::canonicalize(head_path)?)
        );

        let path = cache_dir.insert("head.js", script.as_bytes())?;
        let fullpath = to_js_path(&dunce::canonicalize(&path)?);

        // Entry name "server/head" => esbuild outputs to dist/server/head.js (no collision)
        Ok(HashMap::from([("server/head".to_string(), fullpath)]))
    }
}
