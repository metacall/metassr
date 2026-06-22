mod vendor;

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use metacall::{load, metacall};
use metassr_utils::{checker::CheckerState, js_path::to_js_path};
use std::{collections::HashMap, ffi::OsStr, path::Path, sync::Mutex};

lazy_static! {
    /// A detector for if the bundling script is loaded or not.
    static ref IS_BUNDLING_SCRIPT_LOADED: Mutex<CheckerState> = Mutex::new(CheckerState::default());
}
const BUNDLING_FUNC: &str = "web_bundling";

/// A web bundler that invokes the `web_bundling` function from the Node.js `bundle.js` script
/// using MetaCall. It uses esbuild for fast JavaScript/TypeScript bundling.
#[derive(Debug)]
pub struct WebBundler<'a> {
    pub targets: HashMap<String, &'a Path>,
    pub dist_path: &'a Path,
    pub dev_mode: bool,
}

impl<'a> WebBundler<'a> {
    pub fn new<S>(
        targets: &'a HashMap<String, String>,
        dist_path: &'a S,
        dev_mode: bool,
    ) -> Result<Self>
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

        if !non_found_files.is_empty() {
            return Err(anyhow!(
                "[bundler] Non Exist files found: {:?}",
                non_found_files
            ));
        }

        Ok(Self {
            targets,
            dist_path: Path::new(dist_path),
            dev_mode,
        })
    }

    /// Executes the bundling process by invoking the synchronous `web_bundling`
    /// function from `bundle.js` via MetaCall.
    pub fn exec(&self) -> Result<()> {
        let mut guard = IS_BUNDLING_SCRIPT_LOADED.lock().unwrap();
        if !guard.is_true() {
            let bundle_path = vendor::ensure_vendor_setup()?;

            if let Err(e) = load::from_single_file(
                load::Tag::NodeJS,
                bundle_path
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid vendor path"))?,
                None,
            ) {
                return Err(anyhow!("Cannot load bundling script: {e:?}"));
            }
            guard.make_true();
        }
        drop(guard);

        // esbuild's buildSync is synchronous — no async/promise handling needed
        match metacall::<f64>(
            BUNDLING_FUNC,
            [
                serde_json::to_string(&self.targets)?,
                to_js_path(self.dist_path),
                self.dev_mode.to_string(),
            ],
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Bundling failed: {e:?}")),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use metacall::initialize;

    fn clean() {
        let dist = Path::new("tests/dist");
        if dist.exists() {
            std::fs::remove_dir_all(dist).unwrap();
        }
    }

    #[test]
    fn bundling_works() {
        clean();
        let _metacall = initialize().unwrap();
        let targets = HashMap::from([("pages/home".to_owned(), "./tests/home.js".to_owned())]);

        match WebBundler::new(&targets, "tests/dist", false) {
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
        let targets = HashMap::from([("invalid_path.tsx".to_owned(), "invalid_path".to_owned())]);

        let bundler = WebBundler::new(&targets, "tests/dist", false);
        assert!(bundler.is_err());
    }
}
