use super::{html_props::HtmlProps, template::HtmlTemplate};
use anyhow::Result;
use std::{fs::File, io::Write, path::PathBuf};

const LANG_TAG: &str = "%LANG%";
const HEAD_TAG: &str = "%HEAD%";
const BODY_TAG: &str = "%BODY%";
const SCRIPTS_TAG: &str = "%SCRIPTS%";
const STYLES_TAG: &str = "%STYLES%";

#[derive(Debug, Clone)]
pub struct HtmlOutput(String);

impl HtmlOutput {
    pub fn from(html: &str) -> Self {
        Self(html.to_string())
    }

    pub fn write(&self, path: PathBuf) -> Result<()> {
        let mut file = File::create(path)?;
        file.write_all(self.0.as_bytes())?;
        Ok(())
    }
}

pub struct HtmlBuilder {
    template: HtmlTemplate,
    props: HtmlProps,
}

impl HtmlBuilder {
    pub fn new(template: HtmlTemplate, props: HtmlProps) -> Self {
        Self { template, props }
    }

    pub fn generate(&self) -> HtmlOutput {
        let scripts = self
            .props
            .scripts
            .iter()
            .map(|path| format!("<script src=\"{}\"></script>", path.display()))
            .collect::<Vec<String>>()
            .join("");

        let styles = self
            .props
            .styles
            .iter()
            .map(|path| format!("<link rel=\"stylesheet\" href=\"{}\">", path.display()))
            .collect::<Vec<String>>()
            .join("");

        HtmlOutput::from(
            &self
                .template
                .to_string()
                .replace(LANG_TAG, &self.props.lang)
                .replace(HEAD_TAG, &self.props.head)
                .replace(BODY_TAG, &self.props.body)
                .replace(SCRIPTS_TAG, &scripts)
                .replace(STYLES_TAG, &styles),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{html_props::HtmlProps, template::HtmlTemplate};

    use super::HtmlBuilder;

    #[test]
    fn generating_html() {
        let binding = HtmlProps::new()
            .lang("en")
            .body("<div id=\"root\"></div>")
            .head(
                "
<meta charset=\"utf-8\" />
<meta name=\"viewport\" content=\"width=device-width\" />
<title>This is a static page</title>
        ",
            )
            .scripts(vec!["main.js".to_owned(), "react.js".to_owned()])
            .styles(vec!["style.css".to_owned()]);
        let props = binding.build();
        let html = HtmlBuilder::new(HtmlTemplate::default(), props).generate();
        println!("{html:?}")
    }
}
