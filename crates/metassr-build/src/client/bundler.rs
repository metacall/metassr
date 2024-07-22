use anyhow::{anyhow, Result};
use metacall::{loaders, metacall, MetacallNull};
use std::{collections::HashMap, ffi::OsStr, marker::Sized, path::Path};

static BUILD_SCRIPT: &str = include_str!("./scripts/bundle.js");
const BUNDLING_FUNC: &str = "bundling_client";

pub struct ClientBundler<'a> {
    pub targets: HashMap<String, &'a Path>,
    pub dist_path: &'a Path,
}

impl<'a> ClientBundler<'a> {
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

    pub fn bundling(&self) -> Result<()> {
        if let Err(e) = loaders::from_memory("node", BUILD_SCRIPT) {
            return Err(anyhow!("Cannot load bundling script: {e:?}"));
        }

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
        ClientBundler::new(
            &HashMap::from([(
                "pages/homes.tsx".to_owned(),
                "../../tests/web-app/src/pages/home.tsx".to_owned(),
            )]),
            "../../tests/web-app/dist",
        )
        .bundling()
        .unwrap()
    }
}
