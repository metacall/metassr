use std::mem;

use super::traits::Exec;
use anyhow::anyhow;
use metacall::initialize;

use metassr_build::{client::ClientBuilder, server::BuildingType, server::ServerSideBuilder};
use metassr_bundler::WebBundler;

use std::time::Instant;

use tracing::{error, info};

pub struct Builder {
    out_dir: String,
    _type: BuildingType,
}

impl Builder {
    pub fn new(_type: BuildingType, out_dir: String) -> Self {
        Self { out_dir, _type }
    }
}

impl Exec for Builder {
    fn exec(&self) -> anyhow::Result<()> {
        let _metacall = initialize().unwrap();
        let instant = Instant::now();

        let client_builder = ClientBuilder::new("", &self.out_dir, false)?;
        let server_builder = ServerSideBuilder::new("", &self.out_dir, self._type, false)?;

        // Generate targets for both client and server
        let client_targets = client_builder.generate_targets().map_err(|e| {
            error!(
                target = "builder",
                message = format!("Client target generation failed: {e}")
            );
            anyhow!("Couldn't continue building process.")
        })?;

        let server_state = server_builder.generate_targets().map_err(|e| {
            error!(
                target = "builder",
                message = format!("Server target generation failed: {e}")
            );
            anyhow!("Couldn't continue building process.")
        })?;

        // Combine all targets into a single esbuild compilation
        let mut combined_targets = client_targets;
        combined_targets.extend(server_state.bundling_targets.clone());

        {
            let instant = Instant::now();
            let bundler = WebBundler::new(&combined_targets, &self.out_dir, false)?;
            if let Err(e) = bundler.exec() {
                error!(
                    target = "builder",
                    message = format!("Bundling failed: {e}")
                );
                return Err(anyhow!("Couldn't continue building process."));
            }
            info!(
                target = "builder",
                message = "Bundling is completed",
                time = format!("{}ms", instant.elapsed().as_millis())
            );
        }

        // Run server post-processing (manifest, head rendering, SSG pages)
        {
            let instant = Instant::now();
            server_builder.finish_build(server_state).map_err(|e| {
                error!(
                    target = "builder",
                    message = format!("Server post-processing failed: {e}")
                );
                anyhow!("Couldn't continue building process.")
            })?;
            info!(
                target = "builder",
                message = "Server post-processing is completed",
                time = format!("{}ms", instant.elapsed().as_millis())
            );
        }

        info!(
            target = "builder",
            message = "Building is completed",
            time = format!("{}ms", instant.elapsed().as_millis())
        );

        // Skip metacall_destroy() on drop. The node_loader shutdown hangs on macOS
        // because rspack's native addon leaves libuv handles alive that prevent the
        // event loop from draining. Since the build command exits immediately after
        // this point, the OS reclaims all resources h4.
        mem::forget(_metacall);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_building_type() {
        assert_eq!(
            "ssr".parse::<BuildingType>().unwrap(),
            BuildingType::ServerSideRendering
        );
        assert_eq!(
            "ssg".parse::<BuildingType>().unwrap(),
            BuildingType::StaticSiteGeneration
        );
    }

    #[test]
    fn parse_unsupported_option_returns_err() {
        assert!("csr".parse::<BuildingType>().is_err());
    }
}
