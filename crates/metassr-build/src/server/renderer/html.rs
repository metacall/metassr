use std::path::Path;

use anyhow::Result;
use metassr_fs_analyzer::dist_dir::PageEntry;
use metassr_html::{
    builder::{HtmlBuilder, HtmlOutput},
    html_props::HtmlPropsBuilder,
    template::HtmlTemplate,
};

pub struct HtmlRenderer<'a> {
    head: String,
    body: String,
    page_entry: &'a PageEntry,
    props_json: Option<String>,
}

impl<'a> HtmlRenderer<'a> {
    pub fn new(head: &str, body: &str, page_entry: &'a PageEntry) -> Self {
        Self {
            head: head.to_string(),
            body: body.to_string(),
            page_entry,
            props_json: None,
        }
    }

    pub fn with_props(mut self, json: String) -> Self {
        self.props_json = Some(json);
        self
    }

    pub fn render(&self) -> Result<HtmlOutput> {
        let scripts: Vec<String> = self
            .page_entry
            .scripts
            .iter()
            .map(|p| Path::new("/").join(p).to_str().unwrap().to_owned())
            .collect();

        let styles: Vec<String> = self
            .page_entry
            .styles
            .iter()
            .map(|p| Path::new("/").join(p).to_str().unwrap().to_owned())
            .collect();

        let root_div = format!("<div id='root'>{}</div>", self.body);
        let body_content = match &self.props_json {
            Some(props) => {
                let escaped = escape_script_json(props);
                format!(
                    r#"{root_div}<script id="__METASSR_PROPS__" type="application/json">{escaped}</script>"#
                )
            }
            None => root_div,
        };

        let html_props = HtmlPropsBuilder::new()
            .head(&self.head)
            .body(&body_content)
            .lang("en")
            .scripts(scripts)
            .styles(styles);

        let builder = HtmlBuilder::new(HtmlTemplate::default(), html_props.build());

        Ok(builder.generate())
    }
}

/// Escapes untrusted JSON content for safe embedding inside a `<script>` tag.
///
/// Prevents breakout from the `<script>` tag by escaping `<`, `>`, `&`,
/// and Unicode line terminators U+2028 and U+2029.
fn escape_script_json(json: &str) -> String {
    let mut escaped = String::with_capacity(json.len());
    for c in json.chars() {
        match c {
            '<' => escaped.push_str(r"\u003c"),
            '>' => escaped.push_str(r"\u003e"),
            '&' => escaped.push_str(r"\u0026"),
            '\u{2028}' => escaped.push_str(r"\u2028"),
            '\u{2029}' => escaped.push_str(r"\u2029"),
            _ => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn mock_page_entry() -> PageEntry {
        PageEntry {
            scripts: vec![],
            styles: vec![],
            path: PathBuf::from("pages/test"),
        }
    }

    #[test]
    fn test_render_without_props() {
        let entry = mock_page_entry();
        let renderer = HtmlRenderer::new("<title>Test</title>", "<h1>Hello</h1>", &entry);
        let output = renderer.render().unwrap().to_string();
        assert!(output.contains("<div id='root'><h1>Hello</h1></div>"));
        assert!(!output.contains("__METASSR_PROPS__"));
    }

    #[test]
    fn test_render_with_props_appends_script_tag() {
        let entry = mock_page_entry();
        let props = r#"{"params":{"slug":"hello-world"},"query":{"x":"1"}}"#;
        let renderer = HtmlRenderer::new("", "content", &entry).with_props(props.to_string());
        let output = renderer.render().unwrap().to_string();
        assert!(output.contains(
            r#"<div id='root'>content</div><script id="__METASSR_PROPS__" type="application/json">{"params":{"slug":"hello-world"},"query":{"x":"1"}}</script>"#
        ));
    }

    #[test]
    fn test_escape_script_tag_injection() {
        let entry = mock_page_entry();
        let evil = r#"{"slug":"</script><script>alert(1)</script>"}"#;
        let renderer = HtmlRenderer::new("", "content", &entry).with_props(evil.to_string());
        let output = renderer.render().unwrap().to_string();
        assert!(!output.contains("</script><script>alert(1)</script>"));
        assert!(
            output.contains(r#"\u003c/script\u003e\u003cscript\u003ealert(1)\u003c/script\u003e"#)
        );
    }

    #[test]
    fn test_escape_script_json_all_special_chars() {
        let raw = format!("<>& \u{2028} \u{2029}");
        let escaped = escape_script_json(&raw);
        assert_eq!(escaped, r#"\u003c\u003e\u0026 \u2028 \u2029"#);
    }
}
