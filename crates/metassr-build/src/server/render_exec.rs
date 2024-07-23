use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::traits::Exec;
use anyhow::{anyhow, Result};
use metacall::{loaders, metacall_no_arg};

const RENDER_FUNC_PREFIX: &str = "render_";

pub struct RenderExec {
    id: i64,
    path: PathBuf,
}

impl RenderExec {
    pub fn new<S>(id: i64, path: &S) -> Result<Self>
    where
        S: AsRef<OsStr> + ?Sized,
    {
        let path = Path::new(path);
        if !path.exists() {
            return Err(anyhow!("Path not found: {path:#?}"));
        }
        Ok(Self {
            id,
            path: path.to_path_buf(),
        })
    }
}

impl Exec for RenderExec {
    type Output = String;
    fn exec(&self) -> Result<Self::Output> {
        // let path = self.0.strip_prefix(current_dir()?)?;
        if let Err(e) = loaders::from_single_file("ts", &self.path) {
            return Err(anyhow!(
                "Cannot load render script: {e:?} \n  path: {:#?}",
                self.path
            ));
        }

        match metacall_no_arg::<String>(format!("{}{}", RENDER_FUNC_PREFIX, self.id)) {
            Err(e) => Err(anyhow!(
                "Cannot running {RENDER_FUNC_PREFIX}{}(): {e:?}",
                self.id
            )),
            Ok(out) => Ok(out),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MultiRenderExec(HashMap<i64, PathBuf>);

impl MultiRenderExec {
    pub fn new(files: HashMap<i64, String>) -> Result<Self> {
        let mut self_ = Self(HashMap::new());

        for (id, path) in files {
            let path = Path::new(&path);
            if !path.exists() {
                return Err(anyhow!("Path not found: {path:#?}"));
            }
            self_.0.insert(id, path.to_path_buf());
        }
        Ok(self_)
    }
}

impl Exec for MultiRenderExec {
    type Output = HashMap<String, String>;
    fn exec(&self) -> Result<Self::Output> {
        let mut result: Self::Output = HashMap::new();

        for (id, path) in self.0.iter() {
            let path = path.to_str().unwrap();
            let out = RenderExec::new(*id, &path)?.exec()?;
            result.insert(path.to_owned(), out);
        }
        Ok(result)
    }
}
