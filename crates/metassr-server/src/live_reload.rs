use crate::rebuilder::RebuildType;
use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
    middleware::Next,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;

use tokio::{net::TcpStream, sync::broadcast::Receiver};
use tracing::info;

#[derive(Debug, Serialize)]
struct LiveReloadMessage {
    #[serde(rename = "type")]
    type_: String,
    path: Option<String>,
}

impl RebuildType {
    fn as_message(&self) -> LiveReloadMessage {
        let (type_, path) = match self {
            RebuildType::Page(path) => {
                ("page".to_string(), Some(path.to_string_lossy().to_string()))
            }
            _ => (self.to_string(), None),
        };

        LiveReloadMessage { type_, path }
    }
}

pub struct LiveReloadServer {
    receiver: Receiver<RebuildType>,
}

impl LiveReloadServer {
    pub fn new(receiver: Receiver<RebuildType>) -> Self {
        Self { receiver }
    }

    pub async fn handle_connection(mut self, stream: TcpStream) {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during websocket handshake");

        let (mut ws_sender, _) = ws_stream.split();

        while let Ok(rebuild_type) = self.receiver.recv().await {
            let message = rebuild_type.as_message();
            let message_json = serde_json::to_string(&message).unwrap();

            if let Err(e) = ws_sender.send(Message::Text(message_json.into())).await {
                tracing::error!("Failed to send LiveReload message: {}", e);
                break;
            }
        }
    }
}

/// middleware to inject the live-reload.js script
pub async fn inject_live_reload_script(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let response = next.run(req).await;

    // Check if the response is HTML
    let is_html: bool = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| {
            v.to_str()
                .unwrap_or("")
                .to_lowercase()
                .contains("text/html")
        })
        .unwrap_or(false);

    if is_html {
        let (parts, body) = response.into_parts();

        let body_bytes = axum::body::to_bytes(body, usize::MAX).await.map_err(|e| {
            info!("Failed to read response body: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let body_str = String::from_utf8_lossy(&body_bytes).to_string();

        // Inject script before </body> or append if </body> is missing
        let modified_body = body_str.replace(
            "</body>",
            r#"<script src="/livereload/script.js"></script></body>"#,
        );

        return Ok(Response::builder()
            .status(parts.status)
            .header(header::CONTENT_TYPE, "text/html")
            .header(header::CACHE_CONTROL, "no-cache") // Prevent caching in dev
            .body(Body::from(modified_body))
            .unwrap());
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{header, Request, StatusCode},
        routing::get,
        Router,
    };
    use tower_service::Service;

    #[tokio::test]
    async fn injects_script_into_html_responses() {
        let app = Router::new()
            .route(
                "/",
                get(|| async {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                        .body(Body::from("<html><body>Hello</body></html>"))
                        .unwrap()
                }),
            )
            .layer(axum::middleware::from_fn(inject_live_reload_script));

        let mut app = app;
        let response = Service::call(
            &mut app,
            Request::builder().uri("/").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-cache"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8_lossy(&body);
        assert!(html.contains(r#"<script src="/livereload/script.js"></script>"#));
    }

    #[tokio::test]
    async fn leaves_non_html_responses_unchanged() {
        let app = Router::new()
            .route(
                "/json",
                get(|| async {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(r#"{"ok":true}"#))
                        .unwrap()
                }),
            )
            .layer(axum::middleware::from_fn(inject_live_reload_script));

        let mut app = app;
        let response = Service::call(
            &mut app,
            Request::builder().uri("/json").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().get(header::CACHE_CONTROL).is_none());

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(String::from_utf8_lossy(&body), r#"{"ok":true}"#);
    }
}
