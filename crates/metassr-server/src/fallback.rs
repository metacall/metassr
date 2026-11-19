use axum::response::Html;

pub struct Fallback(String);

impl Fallback {
    pub fn to_html(&self) -> Html<String> {
        Html(self.0.clone())
    }
}

impl Default for Fallback {
    fn default() -> Self {
        Self("<h1>Service not found.</h1>".to_string())
    }
}
