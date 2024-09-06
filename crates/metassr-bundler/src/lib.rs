use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use metacall::{loaders, metacall, MetacallNull};
use std::{collections::HashMap, ffi::OsStr, marker::Sized, path::Path, sync::Mutex};

lazy_static! {
    static ref IS_BUNDLING_SCRIPT_LOADED: Mutex<BundleSciptLoadingState> =
        Mutex::new(BundleSciptLoadingState::new());
}
static BUILD_SCRIPT: &str = include_str!("./bundle.js");
const BUNDLING_FUNC: &str = "web_bundling";

/// A detector for if the bundling script `./bundle.js` is loaded or not.
#[derive(Debug)]
pub struct BundleSciptLoadingState(bool);

impl BundleSciptLoadingState {
    pub fn new() -> Self {
        Self(false)
    }
    pub fn loaded(&mut self) {
        self.0 = true
    }
    pub fn is_loaded(&self) -> bool {
        self.0
    }
}

impl Default for BundleSciptLoadingState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]

pub struct WebBundler<'a> {
    pub targets: HashMap<String, &'a Path>,
    pub dist_path: &'a Path,
}

impl<'a> WebBundler<'a> {
    pub fn new<S>(targets: &'a HashMap<String, String>, dist_path: &'a S) -> Self
    where
        S: AsRef<OsStr> + ?Sized,
    {
        let targets: HashMap<String, &Path> = targets
            .iter()
            .map(|(k, v)| (k.into(), Path::new(v)))
            .collect();

        Self {
            targets,
            dist_path: Path::new(dist_path),
        }
    }
    pub fn exec(&self) -> Result<()> {
        let mut guard = IS_BUNDLING_SCRIPT_LOADED.lock().unwrap();
        if !guard.is_loaded() {
            if let Err(e) = loaders::from_memory("node", BUILD_SCRIPT) {
                return Err(anyhow!("Cannot load bundling script: {e:?}"));
            }
            guard.loaded();
        }
        drop(guard);
    
        if let Err(e) = metacall::<MetacallNull>(
            BUNDLING_FUNC,
            [
                serde_json::to_string(&self.targets)?,
                self.dist_path.to_str().unwrap().to_owned(),
            ],
        ) {
            return Err(anyhow!("Cannot running {BUNDLING_FUNC}(): {e:?}"));
        }
    
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metacall::switch;
    #[test]
    fn it_works() {
        let _metacall = switch::initialize().unwrap();
        WebBundler::new(
            &HashMap::from([(
                "pages/homes.tsx".to_owned(),
                "../../tests/web-app/src/pages/home.tsx".to_owned(),
            )]),
            "../../tests/web-app/dist",
        )
        .exec()
        .unwrap()
    }
}
