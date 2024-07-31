use std::{
    ffi::OsStr,
    marker::Sized,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct HtmlProps {
    // TODO: getting html language from a config file stored in web application root.
    pub lang: String,
    pub head: String,
    pub body: String,
    pub scripts: Vec<PathBuf>,
    pub styles: Vec<PathBuf>,
}

impl HtmlProps {
    pub fn new() -> HtmlPropsBuilder {
        HtmlPropsBuilder::new()
    }
}

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
    pub fn lang<S: ToString + ?Sized>(mut self, lang: &S) -> Self {
        self.lang = Some(lang.to_string());
        self
    }
    pub fn head<S: ToString + ?Sized>(mut self, head: &S) -> Self {
        self.head = Some(head.to_string());
        self
    }
    pub fn body<S: ToString + ?Sized>(mut self, body: &S) -> Self {
        self.body = Some(body.to_string());
        self
    }
    pub fn scripts(mut self, scripts: Vec<String>) -> Self {
        self.scripts = Some(scripts);
        self
    }
    pub fn styles(mut self, styles: Vec<String>) -> Self {
        self.styles = Some(styles);
        self
    }
    pub fn build(&self) -> HtmlProps {
        HtmlProps {
            lang: self.lang.as_ref().unwrap_or(&String::new()).to_owned(),
            head: self.head.as_ref().unwrap_or(&String::new()).to_owned(),
            body: self.body.as_ref().unwrap_or(&String::new()).to_owned(),
            scripts: self
                .scripts
                .as_ref()
                .unwrap()
                .iter()
                .map(|p| Path::new(p).to_path_buf())
                .collect(),
            styles: self
                .styles
                .as_ref()
                .unwrap()
                .iter()
                .map(|p| Path::new(p).to_path_buf())
                .collect(),
        }
    }
}
