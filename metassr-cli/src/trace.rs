use axum::{
    http::{HeaderValue, Request},
    response::Response,
    Router,
};
use tokio::time::Duration;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};

// TODO: Refacor this part
use tracing::{debug, error, Span};
pub fn http_trace(router: Router, enable_http_logging: bool) -> Router {
    let trace_layer = TraceLayer::new_for_http()
        .on_failure(
            |err: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
                error!(
                    target = "http",
                    latency = format!("{latency:?}"),
                    "Server failure: {err}",
                )
            },
        )
        .on_request(move |req: &Request<_>, _span: &Span| {
            if enable_http_logging.clone() {
                debug!(
                    target = "http",
                    user_agent=?  req.headers().get("user-agent").unwrap_or(&HeaderValue::from_str("Unknown").unwrap()),
                    "request: {} {}",
                    req.method(),
                    req.uri().path(),
                )
            }
        })
        .on_response(move |res: &Response<_>, latency: Duration, _span: &Span| {
            if enable_http_logging.clone() {
                debug!(
                    target = "http",
                    "[{}]: generated in {:?}",
                    res.status(),
                    latency,
                )
            }
        });
    router.layer(trace_layer)
}
