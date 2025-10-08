use std::fmt::Display;

static DEFAULT_TEMPLATE: &str = include_str!("./default.html");

pub struct HtmlTemplate(String);

impl HtmlTemplate {
    pub fn new(template: &str) -> Self {
        Self(template.to_owned())
    }
}

impl Default for HtmlTemplate {
    fn default() -> Self {
        Self(DEFAULT_TEMPLATE.to_string())
    }
}

impl Display for HtmlTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
