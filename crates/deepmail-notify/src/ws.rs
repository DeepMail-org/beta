use crate::auth_client::AuthClient;
use crate::db::sessions;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::{SinkExt, StreamExt};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

type WsSender = (Uuid, mpsc::UnboundedSender<String>);

#[derive(Clone)]
pub struct WsHub {
    connections: Arc<RwLock<HashMap<Uuid, Vec<WsSender>>>>,
}

impl WsHub {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(
        &self,
        tenant_id: Uuid,
        session_id: Uuid,
        tx: mpsc::UnboundedSender<String>,
    ) {
        let mut conns = self.connections.write().await;
        conns
            .entry(tenant_id)
            .or_insert_with(Vec::new)
            .push((session_id, tx));
    }

    pub async fn remove_connection(&self, tenant_id: Uuid, session_id: Uuid) {
        let mut conns = self.connections.write().await;
        if let Some(list) = conns.get_mut(&tenant_id) {
            list.retain(|(sid, _)| *sid != session_id);
            if list.is_empty() {
                conns.remove(&tenant_id);
            }
        }
    }

    pub async fn broadcast_to_tenant(&self, tenant_id: Uuid, msg: &str) -> usize {
        let conns = self.connections.read().await;
        let senders = match conns.get(&tenant_id) {
            Some(s) => s,
            None => return 0,
        };

        let mut sent = 0usize;
        let mut dead: Vec<Uuid> = Vec::new();

        for (session_id, tx) in senders {
            if tx.send(msg.to_string()).is_ok() {
                sent += 1;
            } else {
                dead.push(*session_id);
            }
        }

        drop(conns);

        if !dead.is_empty() {
            let mut conns = self.connections.write().await;
            if let Some(list) = conns.get_mut(&tenant_id) {
                list.retain(|(sid, _)| !dead.contains(sid));
                if list.is_empty() {
                    conns.remove(&tenant_id);
                }
            }
        }

        sent
    }
}

#[derive(Clone)]
pub struct WsState {
    pub hub: WsHub,
    pub auth_client: Option<Arc<AuthClient>>,
    pub own_pool: Arc<PgPool>,
}

#[derive(serde::Deserialize)]
pub struct WsParams {
    token: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    State(state): State<WsState>,
) -> Response {
    let auth = match &state.auth_client {
        Some(a) => a,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "auth service unavailable",
            )
                .into_response();
        }
    };

    let claims = match auth.validate_token(&params.token).await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "WS auth failed");
            return (StatusCode::UNAUTHORIZED, "authentication failed").into_response();
        }
    };

    ws.on_upgrade(move |socket| handle_ws_connection(socket, claims, state))
}

async fn handle_ws_connection(
    socket: WebSocket,
    claims: crate::auth_client::TokenClaims,
    state: WsState,
) {
    let session_id = match sessions::insert_session(
        &state.own_pool,
        claims.tenant_id,
        claims.user_id,
        &format!("ws-{}", Uuid::new_v4()),
    )
    .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(error = %e, "failed to insert WS session");
            return;
        }
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    state
        .hub
        .add_connection(claims.tenant_id, session_id, tx)
        .await;

    let (mut sink, mut stream) = socket.split();

    let connected_msg = serde_json::json!({
        "type": "connected",
        "tenant_id": claims.tenant_id.to_string(),
        "user_id": claims.user_id.to_string(),
    })
    .to_string();

    if sink.send(Message::Text(connected_msg)).await.is_err() {
        state
            .hub
            .remove_connection(claims.tenant_id, session_id)
            .await;
        let _ = sessions::mark_disconnected(&state.own_pool, session_id).await;
        return;
    }

    let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(30));
    heartbeat.tick().await;

    loop {
        tokio::select! {
            msg = stream.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if text.contains("\"type\":\"ping\"") {
                            let pong = r#"{"type":"pong"}"#;
                            if sink.send(Message::Text(pong.to_string())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if sink.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
            hub_msg = rx.recv() => {
                match hub_msg {
                    Some(text) => {
                        if sink.send(Message::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            _ = heartbeat.tick() => {
                let hb = r#"{"type":"heartbeat"}"#;
                if sink.send(Message::Text(hb.to_string())).await.is_err() {
                    break;
                }
            }
        }
    }

    state
        .hub
        .remove_connection(claims.tenant_id, session_id)
        .await;
    let _ = sessions::mark_disconnected(&state.own_pool, session_id).await;

    tracing::debug!(
        tenant_id = %claims.tenant_id,
        user_id = %claims.user_id,
        session_id = %session_id,
        "WS disconnected"
    );
}
