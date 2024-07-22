use super::{html_props::HtmlProps, template::HtmlTemplate};

const LANG_TAG: &str = "%LANG%";
const HEAD_TAG: &str = "%HEAD%";
const BODY_TAG: &str = "%BODY%";
const SCRIPTS_TAG: &str = "%SCRIPTS%";
const STYLES_TAG: &str = "%STYLES%";
pub struct HtmlBuilder<'a> {
    template: HtmlTemplate,
    props: HtmlProps<'a>,
}

impl<'a> HtmlBuilder<'a> {
    pub fn new(template: HtmlTemplate, props: HtmlProps<'a>) -> Self {
        Self { template, props }
    }

    pub fn generate(&self) -> String {
        let scripts = self
            .props
            .scripts
            .iter()
            .map(|path| format!("<script src=\"{}\"></scripts>", path.display()))
            .collect::<Vec<String>>()
            .join("");

        let styles = self
            .props
            .styles
            .iter()
            .map(|path| format!("<link rel=\"stylesheet\" href=\"{}\">", path.display()))
            .collect::<Vec<String>>()
            .join("");
        self.template
            .to_string()
            .replace(LANG_TAG, &self.props.lang)
            .replace(HEAD_TAG, &self.props.head)
            .replace(BODY_TAG, &self.props.body)
            .replace(SCRIPTS_TAG, &scripts)
            .replace(STYLES_TAG, &styles)
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
            .scripts(vec!["main.js", "react.js"])
            .styles(vec!["style.css"]);
        let props = binding.build();
        let html = HtmlBuilder::new(HtmlTemplate::default(), props).generate();
        println!("{html}")
    }
}
