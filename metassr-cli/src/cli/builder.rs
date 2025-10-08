use std::{fmt::Display, str::FromStr};

use super::traits::Exec;
use anyhow::{anyhow, Result};
use clap::ValueEnum;
use metacall::initialize;
use metassr_build::server;

use metassr_build::{
    client::{config::ClientConfig, ClientBuilder},
    server::{config::ServerConfig, ServerSideBuilder},
    traits::Build,
};

use std::time::Instant;

use tracing::{error, info};

pub struct Builder {
    root_dir: String,
    out_dir: String,
    _type: BuildingType,
}

impl Builder {
    pub fn new(_type: BuildingType, root_dir: String, out_dir: String) -> Self {
        Self { root_dir, out_dir, _type }
    }
}

impl Exec for Builder {
    fn exec(&self) -> anyhow::Result<()> {
        let _metacall = initialize().unwrap();
        let instant = Instant::now();

        {
            let instant = Instant::now();

            // Create client configuration
            let client_config = ClientConfig::new(".", &self.out_dir)?;
            let client_builder = ClientBuilder::new(client_config);
            
            if let Err(e) = client_builder.build() {
                error!(
                    target = "builder",
                    message = format!("Couldn't build for the client side: {e}"),
                );
                return Err(anyhow!("Couldn't continue building process."));
            }
            info!(
                target = "builder",
                message = "Client building is completed",
                time = format!("{}ms", instant.elapsed().as_millis())
            );
        }

        // Build server-side
        {
            let instant = Instant::now();

            // Create server configuration
            let server_config = ServerConfig::new(".", &self.out_dir)?
                .with_building_type(self._type.into());
            let server_builder = ServerSideBuilder::new(server_config);

            if let Err(e) = server_builder.build() {
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

impl Into<metassr_build::BuildingType> for BuildingType {
    fn into(self) -> metassr_build::BuildingType {
        match self {
            Self::SSG => metassr_build::BuildingType::StaticSiteGeneration,
            Self::SSR => metassr_build::BuildingType::ServerSideRendering,
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
            "ssr" | "server-side rendering" => Ok(BuildingType::Ssg),
            "ssg" | "static-site generation" => Ok(BuildingType::Ssr),
            _ => Err("unsupported option.".to_string()),
        }
    }
}
