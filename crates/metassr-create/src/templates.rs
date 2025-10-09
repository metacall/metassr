use crate::{load_templates, tags, Creator};
use anyhow::Result;
use std::{collections::HashMap, fmt::Display, str::from_utf8};

pub enum Template {
    Javascript,
    Typescript,
}

impl From<&str> for Template {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "javascript" | "js" => Self::Javascript,
            "typescript" | "ts" => Self::Typescript,
            _ => unreachable!("Template isn't detected."),
        }
    }
}

impl Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Javascript => "javascript",
            Self::Typescript => "typescript",
        };
        f.write_fmt(format_args!("{}", s))
    }
}

impl Template {
    pub fn load(&self, creator: &Creator) -> Result<HashMap<String, Vec<u8>>> {
        let template = load_templates();
        let template = template.get(&self.to_string()).unwrap();
        let mut template = template.clone();
        let package_json = from_utf8(template.get("package.json").unwrap())?
            .replace(tags::NAME, &creator.project_name)
            .replace(tags::VERSION, &creator.version)
            .replace(tags::DESC, &creator.description)
            .as_bytes()
            .to_vec();
        template.insert("package.json".to_string(), package_json);
        let ext = match *self {
            Template::Javascript => "jsx",
            Template::Typescript => "tsx",
        };
        let head = from_utf8(template.get(&format!("src/_head.{ext}")).unwrap())?
            .replace(tags::NAME, &creator.project_name)
            .replace(tags::VERSION, &creator.version)
            .as_bytes()
            .to_vec();
        template.insert(format!("src/_head.{ext}"), head);

        Ok(template)
    }
}
