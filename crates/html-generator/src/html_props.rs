use std::{ffi::OsStr, marker::Sized, path::Path};

#[derive(Debug, Clone)]
pub struct HtmlProps<'a> {
    // TODO: getting html language from a config file stored in web application root.
    pub lang: String,
    pub head: String,
    pub body: String,
    pub scripts: Vec<&'a Path>,
    pub styles: Vec<&'a Path>,
}

impl<'a> HtmlProps<'a> {
    pub fn new<R: AsRef<OsStr> + ?Sized>() -> HtmlPropsBuilder<'a, R> {
        HtmlPropsBuilder::new()
    }
}

#[derive(Debug, Clone)]
pub struct HtmlPropsBuilder<'a, R: AsRef<OsStr> + ?Sized> {
    lang: Option<String>,
    head: Option<String>,
    body: Option<String>,
    scripts: Option<Vec<&'a R>>,
    styles: Option<Vec<&'a R>>,
}

impl<'a, R: AsRef<OsStr> + ?Sized> HtmlPropsBuilder<'a, R> {
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
    pub fn scripts(mut self, scripts: Vec<&'a R>) -> Self {
        self.scripts = Some(scripts);
        self
    }
    pub fn styles(mut self, styles: Vec<&'a R>) -> Self {
        self.styles = Some(styles);
        self
    }
    pub fn build(&self) -> HtmlProps {
        HtmlProps {
            lang: self.lang.as_ref().unwrap().to_owned(),
            head: self.head.as_ref().unwrap().to_owned(),
            body: self.body.as_ref().unwrap().to_owned(),
            scripts: self
                .scripts
                .as_ref()
                .unwrap()
                .iter()
                .map(|p| Path::new(p))
                .collect(),
            styles: self
                .styles
                .as_ref()
                .unwrap()
                .iter()
                .map(|p| Path::new(p))
                .collect(),
        }
    }
}
