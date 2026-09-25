use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::Message::{self, Text},
    },
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::{debug, info};

use crate::routes::AppState;

#[derive(PartialEq, Eq, Deserialize)]
pub enum ClientType {
    User,
    ChatFrontend,
}

#[allow(clippy::needless_return)]
pub async fn chat_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(type_of_client): Path<ClientType>,
) -> impl IntoResponse {
    ws.on_upgrade(move |mut socket| async move {
        match type_of_client {
            ClientType::User => {
                let Some(Ok(Message::Text(msg))) = socket.recv().await else {
                    return;
                };

                if !msg.contains("im: ") {
                    return;
                }

                let Some(username) = msg.find(": ").map(|x| &msg[x + 2..]) else {
                    return;
                };

                info!("{:?} is conected to the chat", username);

                let global_tx = state.global_channel;

                while let Some(Ok(msg)) = socket.recv().await {
                    match msg {
                        Text(some_text) => {
                            let plaintext = some_text.to_string();

                            let text = format!("{}: {}", username, plaintext);

                            if global_tx.send(text).is_err() {
                                return;
                            }
                        }
                        _ => {
                            return;
                        }
                    }
                }

                info!("{:?} is disconnect to the chat", username);
            }
            ClientType::ChatFrontend => {
                let mut chat_rx = state.global_channel.subscribe();

                while let Ok(msg) = chat_rx.recv().await {
                    if socket.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    })
}
