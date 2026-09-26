use clap::ValueEnum;
use metassr_create::Creator as MetassrCreator;
use std::{collections::HashMap, fmt::Display, io::IsTerminal, process::Command, str::FromStr};
use tracing::{error, info};

use super::traits::Exec;

// ANSI color codes
pub const RESET: &str = "\x1b[0m";
pub const YELLOW: &str = "\x1b[93m";
pub const BLUE: &str = "\x1b[94m";
pub struct Creator {
    project_name: String,
    version: String,
    description: String,
    template: Template,
    install: bool,
}

impl Creator {
    pub fn new(
        project_name: Option<String>,
        version: Option<String>,
        description: Option<String>,
        template: Option<Template>,
        install: bool,
        yes: bool,
    ) -> anyhow::Result<Self> {
        // Prompts require an interactive terminal. In scripts and agent shells
        // stdin is not a TTY, so fall back to defaults instead of dying on an
        // unanswered prompt; `-y/--yes` forces the same behaviour in a TTY.
        let interactive = std::io::stdin().is_terminal() && !yes;

        let project_name = match project_name {
            Some(name) => name,
            None if interactive => inquire::Text::new("Project name:")
                .with_help_message("Enter the name of your new project")
                .prompt()?,
            None => anyhow::bail!(
                "project name is required when not running interactively (pass it as an argument)"
            ),
        };

        let template = match template {
            Some(template) => template,
            None if interactive => {
                let options = vec![Template::Javascript, Template::Typescript];
                inquire::Select::new("Template:", options)
                    .with_help_message("Choose a template for your new project")
                    .with_starting_cursor(0)
                    .prompt()?
            }
            None => Template::Javascript,
        };

        let version = match version {
            Some(version) => version,
            None if interactive => inquire::Text::new("Version:")
                .with_help_message("Enter the version of your application")
                .with_default("1.0.0")
                .prompt()?,
            None => "1.0.0".to_string(),
        };

        let description = match description {
            Some(desc) => desc,
            None if interactive => inquire::Text::new("Description:")
                .with_default("A web application built with MetaSSR framework")
                .with_help_message("Enter a brief description of your application")
                .prompt()?,
            None => "A web application built with MetaSSR framework".to_string(),
        };

        let install = if install {
            true
        } else if interactive {
            inquire::Select::new("Install dependencies with npm?", vec!["Yes", "No"])
                .with_starting_cursor(0)
                .prompt()?
                == "Yes"
        } else {
            false
        };

        Ok(Self {
            project_name,
            version,
            description,
            template,
            install,
        })
    }
}

impl Exec for Creator {
    fn exec(&self) -> anyhow::Result<()> {
        match MetassrCreator::new(
            &self.project_name,
            &self.version,
            &self.description,
            self.template.as_str(),
        )
        .generate()
        {
            Ok(_) => info!("Project has been created."),
            Err(e) => error!("Couldn't create your project: {e}"),
        };

        if self.install {
            info!("Installing dependencies with npm...");
            let status = Command::new("npm")
                .arg("install")
                .current_dir(&self.project_name)
                .status()
                .map_err(|e| anyhow::anyhow!("failed to run npm install: {e}"))?;

            if !status.success() {
                anyhow::bail!("npm install failed in {}", self.project_name);
            }
        }

        Ok(())
    }
}

#[derive(Debug, ValueEnum, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Template {
    Javascript,
    Typescript,
}
impl Template {
    pub fn as_str(&self) -> &'static str {
        match self {
            Template::Javascript => "javascript",
            Template::Typescript => "typescript",
        }
    }
}

impl Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let templates =
            HashMap::from([(Template::Javascript, YELLOW), (Template::Typescript, BLUE)]);
        write!(
            f,
            "{}{}{RESET}",
            templates.get(self).unwrap(),
            match self {
                Template::Javascript => "javascript",
                Template::Typescript => "typescript",
            }
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_template() {
        assert_eq!("js".parse::<Template>().unwrap(), Template::Javascript);
        assert_eq!("ts".parse::<Template>().unwrap(), Template::Typescript);
    }

    #[test]
    fn as_str_round_trips() {
        assert_eq!(Template::Javascript.as_str(), "javascript");
        assert_eq!(Template::Typescript.as_str(), "typescript");
    }

    #[test]
    fn non_interactive_uses_defaults() {
        let creator = Creator::new(Some("my-app".into()), None, None, None, false, true).unwrap();
        assert_eq!(creator.version, "1.0.0");
        assert_eq!(
            creator.description,
            "A web application built with MetaSSR framework"
        );
        assert_eq!(creator.template, Template::Javascript);
        assert!(!creator.install);
    }

    #[test]
    fn non_interactive_requires_name() {
        assert!(Creator::new(None, None, None, None, false, true).is_err());
    }

    #[test]
    fn explicit_flags_win_over_defaults() {
        let creator = Creator::new(
            Some("my-app".into()),
            Some("2.0.0".into()),
            Some("custom".into()),
            Some(Template::Typescript),
            true,
            true,
        )
        .unwrap();
        assert_eq!(creator.version, "2.0.0");
        assert_eq!(creator.description, "custom");
        assert_eq!(creator.template, Template::Typescript);
        assert!(creator.install);
    }
}
