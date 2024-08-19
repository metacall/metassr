use std::convert::Infallible;

use axum::{
    extract::Request,
    handler::Handler,
    response::IntoResponse,
    routing::{MethodRouter, Route},
    Router,
};
use tower_layer::Layer;
use tower_service::Service;

#[derive(Debug, Clone)]
pub struct RouterMut<S: Clone + Send + Sync + 'static>(Router<S>);

impl<S: Clone + Send + Sync + 'static> RouterMut<S> {
    pub fn route(&mut self, path: &str, method_router: MethodRouter<S>) {
        let app = self.0.clone();
        self.0 = app.route(path, method_router);
    }

    pub fn app(&self) -> Router<S> {
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

    pub fn fallback<H, T>(&mut self, handler: H)
    where
        H: Handler<T, S>,
        T: 'static,
    {
        self.0 = self.0.clone().fallback(handler);
    }
}

impl<S: Clone + Send + Sync + 'static> From<Router<S>> for RouterMut<S> {
    fn from(router: Router<S>) -> Self {
        Self(router)
    }
}
