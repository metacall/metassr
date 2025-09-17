// crates/metassr-server/src/live_reload.rs
use crate::rebuilder::RebuildType;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;

use tokio::{net::TcpStream, sync::broadcast};
// use tokio_tungstenite::accept_async;

#[derive(Debug, Serialize)]
struct LiveReloadMessage {
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

        tracing::info!("New LiveReload connection from: {}", addr);

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

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
