include!(concat!(env!("OUT_DIR"), "/templates.rs"));

use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use templates::Template;

mod templates;

pub mod tags {
    pub const VERSION: &str = "%VER%";
    pub const NAME: &str = "%NAME%";
    pub const DESC: &str = "%DESC%";
}

pub struct Creator {
    project_name: String,
    version: String,
    description: String,
    template: Template,
}
impl Creator {
    pub fn new(project_name: &str, version: &str, desc: &str, template: &str) -> Self {
        Self {
            project_name: project_name.to_string(),
            version: version.to_string(),
            description: desc.to_string(),
            template: Template::from(template),
        }
    }
    pub fn generate(&self) -> Result<()> {
        let template = self.template.load(self)?;
        let root = PathBuf::from(&self.project_name);

        if root.exists() {
            return Err(anyhow!("Path already exists."));
        }
        for (file, buf) in template {
            let path = root.join(&file);
            create_dir_all(path.parent().unwrap())?;

            let _ = File::create(path)?.write(&buf)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{tags, Creator};
    use anyhow::Result;
    use std::{
        env::set_current_dir,
        fs::create_dir,
        path::Path,
        str::from_utf8,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn init_test_dir() -> Result<()> {
        let path = Path::new("tests");
        if !path.exists() {
            create_dir(path)?;
        }
        set_current_dir(path)?;
        Ok(())
    }

    include!(concat!(env!("OUT_DIR"), "/templates.rs"));
    #[test]
    fn load_template() {
        dbg!(&from_utf8(
            load_templates()
                .get("typescript")
                .unwrap()
                .get("src/_head.tsx")
                .unwrap()
        )
        .unwrap()
        .replace(tags::VERSION, "1.0.0")
        .replace(tags::NAME, "MetaSSR"));
    }
    #[test]
    fn generate_templates() -> Result<()> {
        init_test_dir()?;
        let project_name = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();
        dbg!(&project_name);
        Creator::new(
            &format!("{}-javascript", project_name),
            "1.0.0",
            "Hello World!",
            "js",
        )
        .generate()?;
        Creator::new(
            &format!("{}-typescript", project_name),
            "1.0.0",
            "Hello World!",
            "ts",
        )
        .generate()?;

        Ok(())
    }
}
