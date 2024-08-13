use std::path::Path;

use anyhow::Result;
use html_generator::{builder::HtmlBuilder, html_props::HtmlProps, template::HtmlTemplate};
use metassr_utils::dist_analyzer::PageEntry;

pub struct HtmlRenderer <'a>{
    head: String,
    body: String,
    page_entry: &'a PageEntry,
}

impl <'a>HtmlRenderer <'a>{
    pub fn new(head: &str, body: &str, page_entry: &'a PageEntry) -> Self {
        Self {
            head: head.to_string(),
            body: body.to_string(),
            page_entry,
        }
    }

    pub fn render(&self) -> Result<()> {
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

        let html_props = HtmlProps::new()
            // TODO:  Get head content from `_head.tsx`
            .head(&self.head)
            .body(&format!("<div id='root'>{}</div>", self.body))
            .lang("en")
            .scripts(scripts)
            .styles(styles);

        let builder = HtmlBuilder::new(HtmlTemplate::default(), html_props.build());

        builder
            .generate()
            .write(self.page_entry.path.join("index.html"))?;
        Ok(())
    }
}
