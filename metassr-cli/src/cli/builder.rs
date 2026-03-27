use std::{fmt::Display, mem, str::FromStr};

use super::traits::Exec;
use anyhow::{anyhow, Result};
use clap::ValueEnum;
use metacall::initialize;
use metassr_build::server;

use metassr_build::{client::ClientBuilder, server::ServerSideBuilder};
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
        let server_builder = ServerSideBuilder::new("", &self.out_dir, self._type.into(), false)?;

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

        // Combine all targets into a single rspack compilation
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

#[derive(Debug, ValueEnum, PartialEq, Eq, Clone, Copy)]
pub enum BuildingType {
    /// Static Site Generation
    Ssg,
    /// Server Side Rendering
    Ssr,
}

impl From<BuildingType> for server::BuildingType {
    fn from(val: BuildingType) -> Self {
        match val {
            BuildingType::Ssg => server::BuildingType::StaticSiteGeneration,
            BuildingType::Ssr => server::BuildingType::ServerSideRendering,
        }
    }
}

impl Display for BuildingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            Self::Ssg => "ssg",
            Self::Ssr => "ssr",
        })
    }
}

impl FromStr for BuildingType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ssr" | "server-side rendering" => Ok(BuildingType::Ssr),
            "ssg" | "static-site generation" => Ok(BuildingType::Ssg),
            _ => Err("unsupported option.".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_building_type() {
        assert_eq!("ssr".parse::<BuildingType>().unwrap(), BuildingType::Ssr);
        assert_eq!("ssg".parse::<BuildingType>().unwrap(), BuildingType::Ssg);
    }

    #[test]
    fn parse_unsupported_option_returns_err() {
        assert!("csr".parse::<BuildingType>().is_err());
    }

    #[test]
    fn convert_to_server_building_type() {
        let ssr: server::BuildingType = BuildingType::Ssr.into();
        let ssg: server::BuildingType = BuildingType::Ssg.into();
        assert_eq!(ssr, server::BuildingType::ServerSideRendering);
        assert_eq!(ssg, server::BuildingType::StaticSiteGeneration);
    }
}
