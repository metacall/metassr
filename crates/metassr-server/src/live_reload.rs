use crate::rebuilder::RebuildType;
use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
    middleware::Next,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;

use tokio::{net::TcpStream, sync::broadcast};
use tracing::info;
// use tokio_tungstenite::accept_async;

#[derive(Debug, Serialize)]
struct LiveReloadMessage {
    #[serde(rename = "type")]
    type_: String,
    path: Option<String>,
}

pub struct LiveReloadServer {
    receiver: broadcast::Receiver<RebuildType>,
}

impl LiveReloadServer {
    pub fn new(receiver: broadcast::Receiver<RebuildType>) -> Self {
        Self { receiver }
    }

    pub async fn handle_connection(mut self, stream: TcpStream, addr: std::net::SocketAddr) {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during websocket handshake");

        println!("!!!!!!!!! New LiveReload connection from: {}", addr);

        let (mut ws_sender, mut _ws_receiver) = ws_stream.split();

        while let Ok(rebuild_type) = self.receiver.recv().await {
            let message: LiveReloadMessage = match rebuild_type {
                RebuildType::Page(ref path) => LiveReloadMessage {
                    type_: "page".to_string(),
                    path: Some(path.to_string_lossy().to_string()),
                },
                RebuildType::Layout => LiveReloadMessage {
                    type_: "layout".to_string(),
                    path: None,
                },
                RebuildType::Component => LiveReloadMessage {
                    type_: "component".to_string(),
                    path: None,
                },
                RebuildType::Style => LiveReloadMessage {
                    type_: "style".to_string(),
                    path: None,
                },
                RebuildType::Static => LiveReloadMessage {
                    type_: "static".to_string(),
                    path: None,
                },
            };
            let message_json = serde_json::to_string(&message).unwrap();
            if let Err(e) = ws_sender.send(Message::Text(message_json.into())).await {
                tracing::error!("Failed to send LiveReload message: {}", e);
                break;
            }
        }
    }
}

/// middlware to indject the live-reload.js script
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
