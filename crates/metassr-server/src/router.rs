use axum::{routing::MethodRouter, Router};

pub struct RouterMut(Router);

impl RouterMut {
    pub fn route(&mut self, path: &str, method_router: MethodRouter<()>) {
        let app = self.0.clone();
        self.0 = app.route(path, method_router);
    }

    pub fn app(&self) -> Router {
        self.0.clone()
    }
}

impl From<Router> for RouterMut {
    fn from(router: Router) -> Self {
        Self(router)
    }
}
