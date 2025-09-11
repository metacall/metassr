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
    static ref IS_BUNDLING_SCRIPT_LOADED: Mutex<CheckerState> = Mutex::new(CheckerState::default());

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
            checker: Mutex::new(CheckerState::default()),
            cond: Condvar::new(),
        }
    }
}

/// A web bundler that invokes the `web_bundling` function from the Node.js `bundle.js` script
/// using MetaCall. It is designed to bundle web resources like JavaScript and TypeScript files
/// by calling a custom `rspack` configuration.
///
/// The `exec` function blocks the execution until the bundling process completes.
#[derive(Debug)]
pub struct WebBundler<'a> {
    /// A map containing the source entry points for bundling.
    /// The key represents the entry name, and the value is the file path.
    pub targets: HashMap<String, &'a Path>,
    /// The output directory where the bundled files will be stored.
    pub dist_path: &'a Path,
}

impl<'a> WebBundler<'a> {
    /// Creates a new `WebBundler` instance.
    ///
    /// - `targets`: A HashMap where the key is a string representing an entry point, and the value is the file path.
    /// - `dist_path`: The path to the directory where the bundled output should be saved.
    ///
    /// Returns a `WebBundler` struct.
    pub fn new<S>(targets: &'a HashMap<String, String>, dist_path: &'a S) -> Result<Self>
    where
        S: AsRef<OsStr> + ?Sized,
    {
        let mut non_found_files = vec![];
        let targets: HashMap<String, &Path> = targets
            .iter()
            .map(|(k, path)| {
                let path = Path::new(path);
                if !path.exists() {
                    non_found_files.push(path.to_str().unwrap());
                }
                (k.into(), path)
            })
            .collect();

        if non_found_files.len() > 0 {
            return Err(anyhow!(
                "[bundler] Non Exist files found: {:?}",
                non_found_files
            ));
        }

        Ok(Self {
            targets,
            dist_path: Path::new(dist_path),
        })
    }

    /// Executes the bundling process by invoking the `web_bundling` function from `bundle.js` via MetaCall.
    ///
    /// It checks if the bundling script has been loaded, then calls the function and waits for the
    /// bundling to complete, either resolving successfully or logging an error.
    ///
    /// # Errors
    ///
    /// This function returns an `Err` if the bundling script cannot be loaded or if bundling fails.
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
            compilation_wait.cond.notify_one();
        }

        fn reject(err: Box<dyn MetacallValue>, _: Box<dyn MetacallValue>) {
            let compilation_wait = &*Arc::clone(&IS_COMPLIATION_WAIT);
            let mut started = compilation_wait.checker.lock().unwrap();

            error!("Bundling rejected: {err:?}");
            started.make_true();
            compilation_wait.cond.notify_one();
        }

        let future = metacall::<MetacallFuture>(
            BUNDLING_FUNC,
            [
                serde_json::to_string(&self.targets)?,       // entry
                self.dist_path.to_str().unwrap().to_owned(), // dist
            ],
        )
        .unwrap();

        future.then(resolve).catch(reject).await_fut();

        let compilation_wait = Arc::clone(&IS_COMPLIATION_WAIT);
        let mut started = compilation_wait.checker.lock().unwrap();

        while !started.is_true() {
            started = Arc::clone(&IS_COMPLIATION_WAIT).cond.wait(started).unwrap();
        }

        started.make_false();
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use metacall::switch;

    fn clean() {
        let dist = Path::new("test/dist");
        if dist.exists() {
            std::fs::remove_dir_all(dist).unwrap();
        }
    }

    #[test]
    fn bundling_works() {
        clean();
        let _metacall = switch::initialize().unwrap();
        let targets = HashMap::from([("pages/home".to_owned(), "./tests/home.js".to_owned())]);

        match WebBundler::new(&targets, "tests/dist") {
            Ok(bundler) => {
                assert!(bundler.exec().is_ok());
                assert!(Path::new("tests/dist/pages/home.js").exists());
            }
            Err(err) => {
                panic!("BUNDLING TEST FAILED: {err:?}",)
            }
        }
        clean();
    }

    #[test]
    fn invalid_target_fails() {
        clean();
        let _metacall = switch::initialize().unwrap();
        let targets = HashMap::from([("invalid_path.tsx".to_owned(), "invalid_path".to_owned())]);

        let bundler = WebBundler::new(&targets, "tests/dist");
        assert!(bundler.is_err());
    }
}
