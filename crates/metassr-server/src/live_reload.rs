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
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<String>>,
}

impl RebuildType {
    fn as_message(&self) -> LiveReloadMessage {
        match self {
            RebuildType::Page(path) => LiveReloadMessage {
                type_: "page".to_string(),
                path: Some(path.to_string_lossy().to_string()),
                errors: None,
            },
            RebuildType::BuildError { errors } => LiveReloadMessage {
                type_: "build_error".to_string(),
                path: None,
                errors: Some(errors.clone()),
            },
            _ => LiveReloadMessage {
                type_: self.to_string(),
                path: None,
                errors: None,
            },
        }
    }
}

pub struct LiveReloadServer {
    receiver: Receiver<RebuildType>,
    last_errors: std::sync::Arc<std::sync::Mutex<Option<Vec<String>>>>,
}

impl LiveReloadServer {
    pub fn new(
        receiver: Receiver<RebuildType>,
        last_errors: std::sync::Arc<std::sync::Mutex<Option<Vec<String>>>>,
    ) -> Self {
        Self {
            receiver,
            last_errors,
        }
    }

    pub async fn handle_connection(mut self, stream: TcpStream) {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during websocket handshake");

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // immediately send the last known error if there is one on handshake,
        // but wait for the client's ready signal first so that the
        // onmessage handler is guaranteed to be in place before we push anything
        let last_err_msgs = self.last_errors.lock().unwrap().clone();
        if let Some(err_msgs) = last_err_msgs {
            // drain incoming frames until the client sends {type:"ready"}
            'ready: while let Some(msg_result) = ws_receiver.next().await {
                if let Ok(Message::Text(text)) = msg_result {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        if parsed.get("type").and_then(|t| t.as_str()) == Some("ready") {
                            break 'ready;
                        }
                    }
                }
            }
            let message = RebuildType::BuildError { errors: err_msgs }.as_message();
            if let Ok(message_json) = serde_json::to_string(&message) {
                let _ = ws_sender.send(Message::Text(message_json.into())).await;
            }
        }

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
