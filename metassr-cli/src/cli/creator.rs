use clap::ValueEnum;
use metassr_create::Creator as MetassrCreator;
use std::{fmt::Display, str::FromStr};
use tracing::{error, info};

use super::traits::Exec;

pub struct Creator {
    project_name: String,
    version: String,
    description: String,
    template: Template,
}

impl Creator {
    pub fn new(
        project_name: String,
        version: String,
        description: String,
        template: Template,
    ) -> Self {
        Self {
            project_name,
            version,
            description,
            template,
        }
    }
}

impl Exec for Creator {
    fn exec(&self) -> anyhow::Result<()> {
        match MetassrCreator::new(
            &self.project_name,
            &self.version,
            &self.description,
            &self.template.to_string(),
        )
        .generate()
        {
            Ok(_) => info!("Project has been created."),
            Err(e) => error!("Couldn't create your project: {e}"),
        };
        Ok(())
    }
}

#[derive(Debug, ValueEnum, PartialEq, Eq, Clone, Copy)]
pub enum Template {
    Javascript,
    Typescript,
}

impl Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            Self::Javascript => "javascript",
            Self::Typescript => "typescript",
        })
    }
}

impl FromStr for Template {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "js" | "javascript" => Ok(Self::Javascript),
            "ts" | "typescript" => Ok(Self::Typescript),
            _ => unreachable!("Template isn't found."),
        }
    }
}
