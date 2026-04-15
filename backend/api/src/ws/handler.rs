use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use focusflow_db::models::{ProfilingQuestion, Task, WidgetState};

use crate::middleware::auth::validate_token;
use crate::{AppState, ErrorResponse};

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum WsMessage {
    TaskUpdate(Task),
    WidgetUpdate(WidgetState),
    QuestionReady(ProfilingQuestion),
    Ping,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let claims = validate_token(&query.token, &state.jwt_secret).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "invalid or expired token".into(),
                details: None,
            }),
        )
    })?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid user id in token".into(),
                details: None,
            }),
        )
    })?;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, user_id)))
}

async fn handle_socket(mut socket: WebSocket, _state: AppState, user_id: Uuid) {
    tracing::info!(user_id = %user_id, "websocket connected");

    let mut ping_interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                let msg = serde_json::to_string(&WsMessage::Ping).unwrap_or_default();
                if socket.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }

            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        tracing::debug!(user_id = %user_id, text = %text, "ws message received");
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::warn!(user_id = %user_id, error = %e, "ws error");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    tracing::info!(user_id = %user_id, "websocket disconnected");
}
