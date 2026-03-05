use std::sync::OnceLock;

static ANSI_REGEX: OnceLock<regex::Regex> = OnceLock::new();

// returns a compiled regex that matches ANSI escape sequences in both raw
/// (`\x1b[…`) and Rust debug escaped (`\\u{1b}[…`) forms
pub fn ansi_regex() -> &'static regex::Regex {
    ANSI_REGEX.get_or_init(|| {
        regex::Regex::new(
            r"(?:\u{1b}|\\u\{1b\})[\[\]()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]",
        )
        .unwrap()
    })
}
