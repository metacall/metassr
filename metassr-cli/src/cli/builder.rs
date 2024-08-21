use std::{fmt::Display, str::FromStr};

use super::traits::Exec;
use anyhow::{anyhow, Result};
use clap::ValueEnum;
use metacall::switch;
use metassr_build::server;

use metassr_build::{client::ClientBuilder, server::ServerSideBuilder, traits::Build};

use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use tracing::{debug, error};

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
        let _metacall = switch::initialize().unwrap();
        let instant = Instant::now();
        if let Err(e) = ClientBuilder::new("", &self.out_dir)?.build() {
            error!(
                target = "builder",
                message = format!("Couldn't build for the client side:  {e}"),
            );
            return Err(anyhow!("Couldn't continue building process."));
        }

        // TODO: find a solution to remove this
        sleep(Duration::from_secs(1));

        if let Err(e) = ServerSideBuilder::new("", &self.out_dir, self._type.into())?.build() {
            error!(
                target = "builder",
                message = format!("Couldn't build for the server side: {e}"),
            );
            return Err(anyhow!("Couldn't continue building process."));
        }

        if (_metacall.0)() == 0 {
            debug!(
                target = "builder",
                message = "Building is completed",
                time = format!("{}ms", instant.elapsed().as_millis())
            );
        }
        Ok(())
    }
}

#[derive(Debug, ValueEnum, PartialEq, Eq, Clone, Copy)]
pub enum BuildingType {
    /// Static-Site Generation.
    SSG,
    /// Server-Side Rendering.
    SSR,
}

impl Into<server::BuildingType> for BuildingType {
    fn into(self) -> server::BuildingType {
        match self {
            Self::SSG => server::BuildingType::StaticSiteGeneration,
            Self::SSR => server::BuildingType::ServerSideRendering,
        }
    }
}

impl Display for BuildingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            Self::SSG => "ssg",
            Self::SSR => "ssr",
        })
    }
}

impl FromStr for BuildingType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ssr" | "server-side rendering" => Ok(BuildingType::SSR),
            "ssg" | "static-site generation" => Ok(BuildingType::SSG),
            _ => Err("unsupported option.".to_string()),
        }
    }
}
