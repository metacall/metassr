use std::convert::Infallible;

use axum::{
    extract::Request,
    response::IntoResponse,
    routing::{MethodRouter, Route},
    Router,
};
use tower_layer::Layer;
use tower_service::Service;

pub struct RouterMut(Router);

impl RouterMut {
    pub fn route(&mut self, path: &str, method_router: MethodRouter<()>) {
        let app = self.0.clone();
        self.0 = app.route(path, method_router);
    }

    pub fn app(&self) -> Router {
        self.0.clone()
    }

    pub fn layer<L>(&mut self, layer: L)
    where
        L: Layer<Route> + Clone + Send + 'static,
        L::Service: Service<Request> + Clone + Send + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        self.0 = self.0.clone().layer(layer)
    }
}

impl From<Router> for RouterMut {
    fn from(router: Router) -> Self {
        Self(router)
    }
}
