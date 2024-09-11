use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use metacall::{loaders, metacall_no_arg};
use metassr_utils::{cache_dir::CacheDir, checker::CheckerState};
use std::{collections::HashMap, ffi::OsStr, path::PathBuf, sync::Mutex};

use metassr_bundler::WebBundler;

lazy_static! {
    static ref IS_HEAD_SCRIPT_LOADED: Mutex<CheckerState> = Mutex::new(CheckerState::default());
}

pub struct HeadRenderer {
    path: PathBuf,
    cache_dir: CacheDir,
}

impl HeadRenderer {
    pub fn new<S>(path: &S, cache_dir: CacheDir) -> Self
    where
        S: AsRef<OsStr> + ?Sized,
    {
        Self {
            path: PathBuf::from(path),
            cache_dir,
        }
    }

    pub fn render(&mut self, bundler: bool) -> Result<String> {
        let mut guard = IS_HEAD_SCRIPT_LOADED.lock().unwrap();
        if !guard.is_true() {
            if bundler {
                self.bundle()?;
                // TODO: remove this line
                sleep(Duration::from_millis(500));
            }

            let _ = loaders::from_single_file(
                "node",
                format!("{}/head.js", self.cache_dir.dir_path().display()),
            );
            guard.make_true()
        }
        drop(guard);

        match metacall_no_arg::<String>("render_head") {
            Err(e) => Err(anyhow!("Couldn't render head: {e:?}")),
            Ok(out) => Ok(out),
        }
    }

    fn bundle(&mut self) -> Result<()> {
        if let Err(e) = WebBundler::new(&self.bundling_target()?, &self.cache_dir.dir_path()).exec()
        {
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
            self.path.canonicalize()?.display()
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
        let fullpath = path.canonicalize()?.to_str().unwrap().to_string();

        Ok(HashMap::from([(name, fullpath)]))
    }
}
