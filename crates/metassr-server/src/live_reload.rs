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
    use std::path::PathBuf;
    use tower_service::Service;

    // ---- middleware tests ----

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

    #[tokio::test]
    async fn does_not_inject_when_body_tag_missing() {
        let app = Router::new()
            .route(
                "/fragment",
                get(|| async {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from("<div>no body tag here</div>"))
                        .unwrap()
                }),
            )
            .layer(axum::middleware::from_fn(inject_live_reload_script));

        let mut app = app;
        let response = Service::call(
            &mut app,
            Request::builder()
                .uri("/fragment")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8_lossy(&body);
        assert!(!html.contains("livereload"));
    }

    #[tokio::test]
    async fn preserves_original_status_code() {
        let app = Router::new()
            .route(
                "/not-found",
                get(|| async {
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from("<html><body>404</body></html>"))
                        .unwrap()
                }),
            )
            .layer(axum::middleware::from_fn(inject_live_reload_script));

        let mut app = app;
        let response = Service::call(
            &mut app,
            Request::builder()
                .uri("/not-found")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // ---- message serialization (must match what live-reload.js parses) ----

    #[test]
    fn page_message_serializes_with_type_and_path() {
        let msg = RebuildType::Page(PathBuf::from("src/pages/index.tsx")).as_message();
        let json: serde_json::Value = serde_json::to_value(&msg).unwrap();

        assert_eq!(json["type"], "page");
        assert_eq!(json["path"], "src/pages/index.tsx");
    }

    #[test]
    fn non_page_message_serializes_with_null_path() {
        let msg = RebuildType::Layout.as_message();
        let json: serde_json::Value = serde_json::to_value(&msg).unwrap();

        assert_eq!(json["type"], "layout");
        assert!(json["path"].is_null());
    }

    // ---- WebSocket server ----

    #[tokio::test]
    async fn server_sends_rebuild_message_over_websocket() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (sender, _) = tokio::sync::broadcast::channel::<RebuildType>(16);

        let receiver = sender.subscribe();
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let server = LiveReloadServer::new(receiver);
            server.handle_connection(stream).await;
        });

        let url = format!("ws://{}", addr);
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(&url).await.unwrap();

        // Page rebuild — should have type and path
        sender
            .send(RebuildType::Page(PathBuf::from("src/pages/about.tsx")))
            .unwrap();

        let msg = ws_stream.next().await.unwrap().unwrap();
        let json: serde_json::Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
        assert_eq!(json["type"], "page");
        assert_eq!(json["path"], "src/pages/about.tsx");

        // Style rebuild — should have type only
        sender.send(RebuildType::Style).unwrap();

        let msg = ws_stream.next().await.unwrap().unwrap();
        let json: serde_json::Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
        assert_eq!(json["type"], "style");
        assert!(json["path"].is_null());
    }
}
