use std::path::{Path, PathBuf};

/// Properties used by [`HtmlBuilder`](crate::builder::HtmlBuilder) to
/// populate the HTML template.
#[derive(Debug, Clone, Default)]
pub struct HtmlProps {
    // TODO: read the HTML language from a config file in the web-application root.
    pub lang: String,
    pub head: String,
    pub body: String,
    pub scripts: Vec<PathBuf>,
    pub styles: Vec<PathBuf>,
}

/// Builder for [`HtmlProps`].
///
/// All string setters accept anything that implements `Into<String>` — this
/// includes both `&str` literals and owned `String` values without requiring
/// an explicit `.to_string()` call at the call site.
///
/// # Example
///
/// ```
/// use metassr_html::html_props::HtmlPropsBuilder;
///
/// let props = HtmlPropsBuilder::new()
///     .lang("en")
///     .head("<title>My App</title>")
///     .body("<div id=\"root\"></div>")
///     .scripts(vec!["main.js".to_owned()])
///     .styles(vec!["style.css".to_owned()])
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct HtmlPropsBuilder {
    lang: Option<String>,
    head: Option<String>,
    body: Option<String>,
    scripts: Option<Vec<String>>,
    styles: Option<Vec<String>>,
}

impl HtmlPropsBuilder {
    pub fn new() -> Self {
        Self {
            lang: None,
            head: None,
            body: None,
            scripts: None,
            styles: None,
        }
    }

    /// Set the `lang` attribute on the `<html>` element (e.g. `"en"`).
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = Some(lang.into());
        self
    }

    /// Set the contents of the `<head>` section.
    pub fn head(mut self, head: impl Into<String>) -> Self {
        self.head = Some(head.into());
        self
    }

    /// Set the contents of the `<body>` section.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set the list of JavaScript bundle paths to inject as `<script>` tags.
    pub fn scripts(mut self, scripts: Vec<String>) -> Self {
        self.scripts = Some(scripts);
        self
    }

    /// Set the list of CSS paths to inject as `<link rel="stylesheet">` tags.
    pub fn styles(mut self, styles: Vec<String>) -> Self {
        self.styles = Some(styles);
        self
    }

    pub fn build(&self) -> HtmlProps {
        HtmlProps {
            lang: self.lang.clone().unwrap_or_default(),
            head: self.head.clone().unwrap_or_default(),
            body: self.body.clone().unwrap_or_default(),
            scripts: self
                .scripts
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .map(|p| Path::new(p).to_path_buf())
                .collect(),
            styles: self
                .styles
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .map(|p| Path::new(p).to_path_buf())
                .collect(),
        }
    }
}

impl Default for HtmlPropsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::HtmlPropsBuilder;

    /// Builder defaults must produce empty strings and empty vecs — no panics.
    #[test]
    fn default_build_produces_empty_props() {
        let props = HtmlPropsBuilder::new().build();
        assert!(props.lang.is_empty());
        assert!(props.head.is_empty());
        assert!(props.body.is_empty());
        assert!(props.scripts.is_empty());
        assert!(props.styles.is_empty());
    }

    /// Builder setters must accept &str without an explicit .to_string() call.
    #[test]
    fn builder_accepts_str_slices() {
        let props = HtmlPropsBuilder::new()
            .lang("en")
            .head("<title>test</title>")
            .body("<div></div>")
            .build();
        assert_eq!(props.lang, "en");
        assert_eq!(props.head, "<title>test</title>");
        assert_eq!(props.body, "<div></div>");
    }

    /// Builder setters must also accept owned Strings.
    #[test]
    fn builder_accepts_owned_strings() {
        let props = HtmlPropsBuilder::new()
            .lang(String::from("fr"))
            .head(String::from("<title>bonjour</title>"))
            .build();
        assert_eq!(props.lang, "fr");
    }

    /// Scripts and styles are converted to PathBufs correctly.
    #[test]
    fn builder_converts_script_and_style_paths() {
        let props = HtmlPropsBuilder::new()
            .scripts(vec!["dist/main.js".to_owned()])
            .styles(vec!["dist/style.css".to_owned()])
            .build();
        assert_eq!(props.scripts.len(), 1);
        assert_eq!(props.styles.len(), 1);
    }
}
