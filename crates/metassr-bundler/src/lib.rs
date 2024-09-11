use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use metacall::{loaders, metacall, MetacallFuture, MetacallValue};
use metassr_utils::checker::CheckerState;
use std::{
    collections::HashMap,
    ffi::OsStr,
    marker::Sized,
    path::Path,
    sync::{Arc, Condvar, Mutex},
};
use tracing::error;

lazy_static! {
    /// A detector for if the bundling script `./bundle.js` is loaded or not. It is used to solve multiple loading script error in metacall.
    static ref IS_BUNDLING_SCRIPT_LOADED: Mutex<CheckerState> = Mutex::new(CheckerState::new());

    /// A simple checker to check if the bundling function is done or not. It is used to block the program until bundling done.
    static ref IS_COMPLIATION_WAIT: Arc<CompilationWait> = Arc::new(CompilationWait::default());
}
static BUILD_SCRIPT: &str = include_str!("./bundle.js");
const BUNDLING_FUNC: &str = "web_bundling";

/// A simple struct for compilation wait of the bundling function.
struct CompilationWait {
    checker: Mutex<CheckerState>,
    cond: Condvar,
}

impl Default for CompilationWait {
    fn default() -> Self {
        Self {
            checker: Mutex::new(CheckerState::with(false)),
            cond: Condvar::new(),
        }
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
        if !guard.is_true() {
            if let Err(e) = loaders::from_memory("node", BUILD_SCRIPT) {
                return Err(anyhow!("Cannot load bundling script: {e:?}"));
            }
            guard.make_true();
        }
        drop(guard);

        fn resolve(_: Box<dyn MetacallValue>, _: Box<dyn MetacallValue>) {
            let compilation_wait = &*Arc::clone(&IS_COMPLIATION_WAIT);
            let mut started = compilation_wait.checker.lock().unwrap();

            started.make_true();
            // We notify the condvar that the value has changed
            compilation_wait.cond.notify_one();
        }

        fn reject(err: Box<dyn MetacallValue>, _: Box<dyn MetacallValue>) {
            let compilation_wait = &*Arc::clone(&IS_COMPLIATION_WAIT);
            let mut started = compilation_wait.checker.lock().unwrap();

            error!("Bundling rejected: {err:?}");

            started.make_true();
            // We notify the condvar that the value has changed
            compilation_wait.cond.notify_one();
        }

        let future = metacall::<MetacallFuture>(
            BUNDLING_FUNC,
            [
                serde_json::to_string(&self.targets)?,
                self.dist_path.to_str().unwrap().to_owned(),
            ],
        )
        .unwrap();

        future.then(resolve).catch(reject).await_fut();

        // Wait for the thread to start up.
        let compilation_wait = Arc::clone(&IS_COMPLIATION_WAIT);

        let mut started = compilation_wait.checker.lock().unwrap();

        // Waiting till future done
        while !started.is_true() {
            started = Arc::clone(&IS_COMPLIATION_WAIT).cond.wait(started).unwrap();
        }
        // Reset checker
        started.make_false();
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
