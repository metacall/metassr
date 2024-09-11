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

        let compilation_wait = Arc::new((Mutex::new(false), Condvar::new()));
        let compilation_wait_clone = Arc::clone(&compilation_wait);

        fn resolve(result: impl MetacallValue, data: impl MetacallValue) {
            let (lock, cvar) = &*compilation_wait_clone;
            let mut started = lock.lock().unwrap();
            println!("Result of the compilation: {result}");
            *started = true;
            // We notify the condvar that the value has changed
            cvar.notify_one();
        }

        fn reject(result: impl MetacallValue, data: impl MetacallValue) {
            let (lock, cvar) = &*compilation_wait_clone;
            let mut started = lock.lock().unwrap();
            println!("Error with compilation: {result}");
            *started = true;
            // We notify the condvar that the value has changed
            cvar.notify_one();
        }

        let future = metacall::<MetacallFuture>(BUNDLING_FUNC, [
            serde_json::to_string(&self.targets)?,
            self.dist_path.to_str().unwrap().to_owned(),
        ]).unwrap();

        future.then(resolve).catch(reject).await_fut();

        // Wait for the thread to start up.
        let (lock, cvar) = &*compilation_wait;
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap();
        }

        /*
        if let Err(e) = metacall::<MetacallNull>(
            BUNDLING_FUNC,
            [
                serde_json::to_string(&self.targets)?,
                self.dist_path.to_str().unwrap().to_owned(),
            ],
        ) {
            return Err(anyhow!("Cannot running {BUNDLING_FUNC}(): {e:?}"));
        }
        */
    
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
