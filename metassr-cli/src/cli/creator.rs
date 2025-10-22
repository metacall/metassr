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
        project_name: Option<String>,
        version: Option<String>,
        description: Option<String>,
        template: Option<Template>,
    ) -> anyhow::Result<Self> {
        let project_name = match project_name {
            Some(name) => name,
            None => inquire::Text::new("Project name:")
                .with_help_message("Enter the name of your new project")
                .prompt()?,
        };

        let template = match template {
            Some(template) => template,
            None => {
                let options = vec![Template::Javascript, Template::Typescript];
                inquire::Select::new("Template:", options)
                    .with_help_message("Choose a template for your new project")
                    .with_starting_cursor(0)
                    .prompt()?
            }
        };

        let version = match version {
            Some(version) => version,
            None => inquire::Text::new("Version:")
                .with_help_message("Enter the version of your application")
                .with_default("1.0.0")
                .prompt()?,
        };

        let description = match description {
            Some(desc) => desc,
            None => inquire::Text::new("Description:")
                .with_default("A web application built with MetaSSR framework")
                .with_help_message("Enter a brief description of your application")
                .prompt()?,
        };

        Ok(Self {
            project_name,
            version,
            description,
            template,
        })
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
