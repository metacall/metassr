use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::traits::Exec;
use anyhow::{anyhow, Result};
use metacall::{
    load::{self, Handle},
    metacall_handle_no_arg,
};

const RENDER_FUNC_PREFIX: &str = "render_";

pub struct RenderExec {
    id: i64,
    handle: Handle,
}

impl RenderExec {
    /// Loads the render script once and scopes it to this handle, so that
    /// [`call`](Self::call) can be repeated per request cheaply.
    pub fn new<S>(id: i64, path: &S) -> Result<Self>
    where
        S: AsRef<OsStr> + ?Sized,
    {
        let path = Path::new(path);
        if !path.exists() {
            return Err(anyhow!("Path not found: {path:#?}"));
        }

        let mut handle = Handle::new();
        if let Err(e) = load::from_single_file(load::Tag::NodeJS, path, Some(&mut handle)) {
            return Err(anyhow!(
                "Cannot load render script: {e:?} \n  path: {:#?}",
                path
            ));
        }

        Ok(Self { id, handle })
    }

    /// Runs the page render function within this script's handle.
    ///
    /// `Handle` is `Send + Sync`, so this is safe to call per request from the
    /// axum handlers. NOTE: the call is synchronous and runs directly on the
    /// tokio worker thread; wrapping it in `tokio::task::spawn_blocking` is a
    /// possible follow-up if render load becomes a bottleneck.
    pub fn call(&self) -> Result<String> {
        match metacall_handle_no_arg::<String>(
            &self.handle,
            format!("{}{}", RENDER_FUNC_PREFIX, self.id),
        ) {
            Err(e) => Err(anyhow!(
                "Cannot run {RENDER_FUNC_PREFIX}{}(): {e:?}",
                self.id
            )),
            Ok(out) => Ok(out),
        }
    }
}

impl Exec for RenderExec {
    type Output = String;
    fn exec(&self) -> Result<Self::Output> {
        self.call()
    }
}

#[derive(Debug, Clone)]
pub struct MultiRenderExec(HashMap<PathBuf, i64>);

impl MultiRenderExec {
    pub fn new(files: HashMap<String, i64>) -> Result<Self> {
        let mut self_ = Self(HashMap::new());

        for (path, id) in files {
            let path = Path::new(&path);
            if !path.exists() {
                return Err(anyhow!("Path not found: {path:#?}"));
            }
            self_.0.insert(path.to_path_buf(), id);
        }
        Ok(self_)
    }
}

impl Exec for MultiRenderExec {
    type Output = HashMap<String, String>;
    fn exec(&self) -> Result<Self::Output> {
        let mut result: Self::Output = HashMap::new();

        for (path, id) in self.0.iter() {
            let path = path.to_str().unwrap();
            let out = RenderExec::new(*id, &path)?.exec()?;
            result.insert(path.to_owned(), out);
        }
        Ok(result)
    }
}
