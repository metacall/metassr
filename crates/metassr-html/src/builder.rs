use super::{html_props::HtmlProps, template::HtmlTemplate};
use anyhow::Result;
use std::{fmt::Display, fs::File, io::Write, path::PathBuf};

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

impl Display for HtmlOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
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
    use crate::{html_props::HtmlPropsBuilder, template::HtmlTemplate};

    use super::HtmlBuilder;

    #[test]
    fn generating_html() {
        let binding = HtmlPropsBuilder::new()
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
        let html = HtmlBuilder::new(HtmlTemplate::default(), props)
            .generate()
            .to_string();

        assert!(html.contains("<title>This is a static page</title>"));
        assert!(html.contains("<div id=\"root\"></div>"));
        assert!(html.contains("<script src=\"main.js\">"));
        assert!(html.contains("<script src=\"react.js\">"));
        assert!(html.contains("<link rel=\"stylesheet\" href=\"style.css\">"));
        assert!(html.contains("lang=\"en\""));
    }

    #[test]
    fn generating_html_without_scripts_or_styles() {
        // This previously panicked before the fix in HtmlPropsBuilder::build()
        let props = HtmlPropsBuilder::new()
            .lang("en")
            .body("<div id=\"root\"></div>")
            .head("<title>Test</title>")
            .build();
        let html = HtmlBuilder::new(HtmlTemplate::default(), props).generate();
        let output = html.to_string();
        assert!(output.contains("<title>Test</title>"));
        assert!(output.contains("<div id=\"root\"></div>"));
        assert!(!output.contains("<script")); // no scripts injected
        assert!(!output.contains("<link rel=\"stylesheet\"")); // no styles injected
    }

    #[test]
    fn generating_html_contains_correct_script_tags() {
        let props = HtmlPropsBuilder::new()
            .scripts(vec!["app.js".to_owned()])
            .styles(vec!["style.css".to_owned()])
            .build();
        let html = HtmlBuilder::new(HtmlTemplate::default(), props)
            .generate()
            .to_string();
        assert!(html.contains("<script src=\"app.js\">"));
        assert!(html.contains("<link rel=\"stylesheet\" href=\"style.css\">"));
    }

    #[test]
    fn generating_html_with_empty_lang_defaults_gracefully() {
        let props = HtmlPropsBuilder::new().build(); // all defaults
        let html = HtmlBuilder::new(HtmlTemplate::default(), props)
            .generate()
            .to_string();
        assert!(html.contains("<html lang=\"\">"));
        assert!(!html.contains("%LANG%"));
        assert!(!html.contains("%HEAD%"));
        assert!(!html.contains("%BODY%"));
        assert!(!html.contains("%SCRIPTS%"));
        assert!(!html.contains("%STYLES%"));
    }
}
