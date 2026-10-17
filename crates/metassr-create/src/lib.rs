include!(concat!(env!("OUT_DIR"), "/templates.rs"));

use std::{
    fs::{create_dir_all, File},
    io::{self, Write},
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

/// Write `buf` to `writer` completely, retrying short writes.
fn write_generated_file(mut writer: impl Write, buf: &[u8]) -> io::Result<()> {
    writer.write_all(buf)
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

            write_generated_file(File::create(path)?, &buf)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{tags, templates::Template, write_generated_file, Creator};
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
        let templates = load_templates();

        dbg!(&from_utf8(
            templates
                .get("typescript")
                .unwrap()
                .get("src/_head.tsx")
                .unwrap()
        )
        .unwrap()
        .replace(tags::VERSION, "1.0.0")
        .replace(tags::NAME, "MetaSSR"));

        for (template, config) in [
            ("javascript", "jsconfig.json"),
            ("typescript", "tsconfig.json"),
        ] {
            let config = from_utf8(templates.get(template).unwrap().get(config).unwrap()).unwrap();
            assert!(
                config.contains(r#""jsx": "react-jsx""#),
                "{template} template must use React's automatic JSX runtime"
            );
        }
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

    /// Accepts only one byte per `write` call, like a short write would.
    struct ShortWriter {
        written: Vec<u8>,
    }

    impl std::io::Write for ShortWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            match buf.first() {
                Some(byte) => {
                    self.written.push(*byte);
                    Ok(1)
                }
                None => Ok(0),
            }
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn write_generated_file_writes_whole_buffer_on_short_writes() {
        let mut writer = ShortWriter {
            written: Vec::new(),
        };

        write_generated_file(&mut writer, b"hello").unwrap();

        assert_eq!(writer.written, b"hello");
    }

    #[test]
    fn generate_writes_complete_files() -> Result<()> {
        let dir = tempfile::TempDir::new()?;
        let project = dir.path().join("my-app");
        let project_name = project.to_str().unwrap();

        let creator = Creator::new(project_name, "1.0.0", "Hello World!", "ts");
        creator.generate()?;

        let expected = Template::from("ts").load(&creator)?;
        assert!(!expected.is_empty());

        for (relative, content) in expected {
            let generated = std::fs::read(project.join(&relative))?;
            assert_eq!(
                generated, content,
                "generated {relative} differs from the template"
            );
        }

        Ok(())
    }
}
