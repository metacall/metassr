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

    /// Executes the bundling process by invoking the `web_bundling` function from `bundle.js` via MetaCall.
    ///
    /// It checks if the bundling script has been loaded, then calls the function and waits for the
    /// bundling to complete, either resolving successfully or logging an error.
    ///
    /// # Errors
    ///
    /// This function returns an `Err` if the bundling script cannot be loaded or if bundling fails.
    pub fn exec(&self) -> Result<()> {
        // Lock the mutex to check if the bundling script is already loaded
        let mut guard = IS_BUNDLING_SCRIPT_LOADED.lock().unwrap();
        if !guard.is_true() {
            // If not loaded, attempt to load the script into MetaCall
            if let Err(e) = loaders::from_memory("node", BUILD_SCRIPT) {
                return Err(anyhow!("Cannot load bundling script: {e:?}"));
            }
            // Mark the script as loaded
            guard.make_true();
        }
        // Drop the lock on the mutex as it's no longer needed
        drop(guard);

        // Resolve callback when the bundling process is completed successfully
        fn resolve(_: Box<dyn MetacallValue>, _: Box<dyn MetacallValue>) {
            let compilation_wait = &*Arc::clone(&IS_COMPLIATION_WAIT);
            let mut started = compilation_wait.checker.lock().unwrap();

            // Mark the process as completed and notify waiting threads
            started.make_true();
            compilation_wait.cond.notify_one();
        }

        // Reject callback for handling errors during the bundling process
        fn reject(err: Box<dyn MetacallValue>, _: Box<dyn MetacallValue>) {
            let compilation_wait = &*Arc::clone(&IS_COMPLIATION_WAIT);
            let mut started = compilation_wait.checker.lock().unwrap();

            // Log the bundling error and mark the process as completed
            error!("Bundling rejected: {err:?}");
            started.make_true();
            compilation_wait.cond.notify_one();
        }

        // Call the `web_bundling` function in the MetaCall script with targets and output path
        let future = metacall::<MetacallFuture>(
            BUNDLING_FUNC,
            [
                // Serialize the targets map to a string format
                serde_json::to_string(&self.targets)?,
                // Get the distribution path as a string
                self.dist_path.to_str().unwrap().to_owned(),
            ],
        )
        .unwrap();

        // Set the resolve and reject handlers for the bundling future
        future.then(resolve).catch(reject).await_fut();

        // Lock the mutex and wait for the bundling process to complete
        let compilation_wait = Arc::clone(&IS_COMPLIATION_WAIT);
        let mut started = compilation_wait.checker.lock().unwrap();

        // Block the current thread until the bundling process signals completion
        while !started.is_true() {
            started = Arc::clone(&IS_COMPLIATION_WAIT).cond.wait(started).unwrap();
        }

        // Reset the checker state to false after the process completes
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
