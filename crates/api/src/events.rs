use std::convert::Infallible;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use axum_extra::sse::{Event, KeepAlive, Sse};
use futures::stream::{self, Stream};
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

pub async fn handle_sse(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.event_tx.subscribe();
    let stream = stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let data = match serde_json::to_string(&event) {
                        Ok(d) => d,
                        Err(_) => continue,
                    };
                    return Some((Ok(Event::default().data(data)), rx));
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub async fn handle_events(
    ws: Option<WebSocketUpgrade>,
    State(state): State<AppState>,
) -> ApiResult<axum::response::Response> {
    if let Some(ws) = ws {
        Ok(ws.on_upgrade(move |socket| handle_ws_socket(socket, state)))
    } else {
        let sse = handle_sse(State(state)).await;
        Ok(sse.into_response())
    }
}
