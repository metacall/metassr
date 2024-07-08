use anyhow::{anyhow, Result};
use metacall::{loaders, metacall, MetacallNull};
use std::{ffi::OsStr, marker::Sized, path::Path};

static BUILD_SCRIPT: &str = include_str!("./scripts/bundle.js");
const BUNDLING_FUNC: &str = "bundling_client";

pub struct ClientBundler<'a> {
    pub target: &'a Path,
    pub dist_path: &'a Path,
}

impl<'a> ClientBundler<'a> {
    pub fn new<S>(target: &'a S, dist_path: &'a S) -> Self
    where
        S: AsRef<OsStr> + ?Sized,
    {
        Self {
            target: Path::new(target),
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
                self.target.to_str().unwrap().to_owned(),
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
            "../../tests/web-app/src/pages/home.ts",
            "../../tests/web-app/dist",
        )
        .bundling()
        .unwrap()
    }
}
