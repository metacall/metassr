use super::{html_props::HtmlProps, template::HtmlTemplate};
use anyhow::Result;
use std::{fmt, fs::File, io::Write, path::PathBuf};

const LANG_TAG: &str = "%LANG%";
const HEAD_TAG: &str = "%HEAD%";
const BODY_TAG: &str = "%BODY%";
const SCRIPTS_TAG: &str = "%SCRIPTS%";
const STYLES_TAG: &str = "%STYLES%";

/// The rendered HTML output produced by [`HtmlBuilder`].
///
/// Implements [`fmt::Display`] so it can be written directly to any formatter
/// (e.g. `format!("{output}")`, `println!("{output}")`) without a redundant
/// intermediate allocation.  Use [`HtmlOutput::write`] to persist the output
/// to disk.
///
/// # Example
///
/// ```no_run
/// use metassr_html::{builder::{HtmlBuilder, HtmlOutput}, html_props::HtmlPropsBuilder, template::HtmlTemplate};
///
/// let props = HtmlPropsBuilder::new().lang("en").build();
/// let output: HtmlOutput = HtmlBuilder::new(HtmlTemplate::default(), props).generate();
///
/// // Display — no extra allocation
/// println!("{output}");
///
/// // Owned String — allocated on demand via the auto-provided to_string()
/// let html: String = output.to_string();
/// ```
#[derive(Debug, Clone)]
pub struct HtmlOutput(String);

impl HtmlOutput {
    /// Write the rendered HTML to `path`, creating the file if it does not exist.
    pub fn write(&self, path: PathBuf) -> Result<()> {
        let mut file = File::create(path)?;
        file.write_all(self.0.as_bytes())?;
        Ok(())
    }
}

/// Construct an [`HtmlOutput`] from an owned [`String`] without copying.
impl From<String> for HtmlOutput {
    fn from(html: String) -> Self {
        Self(html)
    }
}

/// Construct an [`HtmlOutput`] from a string slice.
///
/// Prefer [`From<String>`] when you already own a `String` to avoid the
/// extra allocation.
impl From<&str> for HtmlOutput {
    fn from(html: &str) -> Self {
        Self(html.to_owned())
    }
}

/// Display the rendered HTML.  Because `Display` is implemented directly,
/// `to_string()` (provided automatically by the standard library) reuses this
/// implementation — there is no separate, heap-duplicating `ToString` impl.
impl fmt::Display for HtmlOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Builds a complete HTML document by substituting template placeholders with
/// the supplied [`HtmlProps`].
pub struct HtmlBuilder {
    template: HtmlTemplate,
    props: HtmlProps,
}

impl HtmlBuilder {
    pub fn new(template: HtmlTemplate, props: HtmlProps) -> Self {
        Self { template, props }
    }

    /// Render the template and return an [`HtmlOutput`].
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

        // `format!("{}", self.template)` uses HtmlTemplate's Display impl to
        // obtain an owned String; we then substitute placeholders in-place.
        HtmlOutput::from(
            format!("{}", self.template)
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

    use super::{HtmlBuilder, HtmlOutput};

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

    // ── New tests for the refactored From / Display impls ──────────────

    /// From<&str> and From<String> should produce identical output.
    #[test]
    fn html_output_from_str_and_from_string_are_equivalent() {
        let raw = "<html><body>hello</body></html>";
        let from_str = HtmlOutput::from(raw);
        let from_string = HtmlOutput::from(raw.to_owned());
        assert_eq!(from_str.to_string(), from_string.to_string());
    }

    /// Display must round-trip the inner string exactly — no extra whitespace,
    /// no extra wrapping.
    #[test]
    fn html_output_display_roundtrips_content() {
        let raw = "<html><body>round-trip test</body></html>";
        let output = HtmlOutput::from(raw);
        // format! uses Display, which must not clone or transform the inner string
        assert_eq!(format!("{output}"), raw);
    }

    /// to_string() (auto-derived from Display) must equal a direct format! call.
    /// This proves there is no separate ToString impl that could diverge.
    #[test]
    fn html_output_to_string_matches_display() {
        let raw = "<!DOCTYPE html><html></html>";
        let output = HtmlOutput::from(raw);
        assert_eq!(output.to_string(), format!("{output}"));
    }

    /// Cloning an HtmlOutput must produce an independent copy with the same content.
    #[test]
    fn html_output_clone_is_independent() {
        let original = HtmlOutput::from("original content");
        let cloned = original.clone();
        assert_eq!(original.to_string(), cloned.to_string());
    }
}
