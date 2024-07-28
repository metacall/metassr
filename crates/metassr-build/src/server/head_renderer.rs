use anyhow::{anyhow, Result};
use metacall::{loaders, metacall_no_arg};
use metassr_utils::cache_dir::CacheDir;
use std::{ffi::OsStr, fmt::format, path::PathBuf};
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

    pub fn render(&mut self) -> Result<String> {
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

        self.cache_dir.insert("head.tsx", script.as_bytes());
        dbg!(&script);
        if let Err(e) = loaders::from_single_file(
            "ts",
            format!("{}/head.tsx", self.cache_dir.dir_path().display()),
        ) {
            return Err(anyhow!("Couldn't load head rendering script: {e:?}"));
        }
        match metacall_no_arg::<String>("render_head") {
            Err(e) => Err(anyhow!("Couldn't render head: {e:?}")),
            Ok(out) => Ok(out),
        }
    }
}
