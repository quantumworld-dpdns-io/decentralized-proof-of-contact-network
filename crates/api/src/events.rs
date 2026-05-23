use std::convert::Infallible;

use axum::body::Body;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use futures::stream::{self, StreamExt};
use poi_core::types::{ContactProof, NodeId, OrbitalWindow, ProofId, VerificationStatus};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::ApiResult;
use crate::models::{AppState, PeerInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    ProofCreated(ContactProof),
    ProofVerified {
        proof_id: ProofId,
        status: VerificationStatus,
    },
    PeerConnected(PeerInfo),
    PeerDisconnected(NodeId),
    WindowOpened(OrbitalWindow),
}

pub async fn handle_ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_socket(socket, state))
}

async fn handle_ws_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.event_tx.subscribe();
    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        if let Ok(data) = serde_json::to_string(&event) {
                            if socket.send(Message::Text(data)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket event stream lagged by {} messages", n);
                    }
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(_)) => {}
                    _ => break,
                }
            }
        }
    }
}

pub async fn handle_sse(state: AppState) -> Response {
    let rx = state.event_tx.subscribe();
    let stream = stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let data = match serde_json::to_string(&event) {
                        Ok(d) => d,
                        Err(_) => continue,
                    };
                    let msg = format!("data: {}\n\n", data);
                    return Some((Ok::<_, Infallible>(axum::body::Bytes::from(msg)), rx));
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    });

    Response::builder()
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap()
}

pub async fn handle_events(
    ws: Option<WebSocketUpgrade>,
    State(state): State<AppState>,
) -> ApiResult<Response> {
    match ws {
        Some(ws) => Ok(ws.on_upgrade(move |socket| handle_ws_socket(socket, state))),
        None => {
            let sse = handle_sse(state).await;
            Ok(sse)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use poi_core::types::{NodeId, OrbitalWindow, WindowType};
    use chrono::{Duration, Utc};

    #[test]
    fn test_app_event_serialization() {
        let event = AppEvent::WindowOpened(
            OrbitalWindow::new(Utc::now(), Utc::now() + Duration::hours(1), WindowType::Standard)
        );
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("WindowOpened"));
    }

    #[test]
    fn test_app_event_peer_connected() {
        let peer = PeerInfo {
            id: NodeId::new(),
            address: "127.0.0.1:9000".into(),
            connected_at: Utc::now(),
            latency_ms: 10,
        };
        let event = AppEvent::PeerConnected(peer);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("PeerConnected"));
    }
}
