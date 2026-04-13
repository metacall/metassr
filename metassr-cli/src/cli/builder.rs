use std::{fmt::Display, mem, str::FromStr};

use super::traits::Exec;
use anyhow::{anyhow, Result};
use clap::ValueEnum;
use metacall::initialize;
use metassr_build::server;

use metassr_build::{client::ClientBuilder, server::ServerSideBuilder, traits::Build};

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

        {
            let instant = Instant::now();

            if let Err(e) = ClientBuilder::new("", &self.out_dir)?.build() {
                error!(
                    target = "builder",
                    message = format!("Couldn't build for the client side:  {e}"),
                );
                return Err(anyhow!("Couldn't continue building process."));
            }
            info!(
                target = "builder",
                message = "Client building is completed",
                time = format!("{}ms", instant.elapsed().as_millis())
            );
        }

        {
            let instant = Instant::now();

            if let Err(e) = ServerSideBuilder::new("", &self.out_dir, self._type.into())?.build() {
                error!(
                    target = "builder",
                    message = format!("Couldn't build for the server side: {e}"),
                );
                return Err(anyhow!("Couldn't continue building process."));
            }

            info!(
                target = "builder",
                message = "Server building is completed",
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
