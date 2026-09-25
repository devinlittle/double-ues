use axum::{
    extract::{
        ws::Message::{self, Text},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;

use crate::routes::AppState;

#[derive(PartialEq, Eq, Deserialize, ToSchema)]
pub enum ClientType {
    User,
    ChatFrontend,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct HexColor(pub u8, pub u8, pub u8);

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct OnboardFields {
    pub username: String,
    pub username_color: HexColor,
    pub message_font: String,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct ChatMessage {
    pub username: String,
    pub username_color: HexColor,
    pub message: String,
    pub message_font: String,
}

#[allow(clippy::needless_return)]
#[utoipa::path(
    post,
    params(
        ("type_of_client" = ClientType, description = "Determines whether the client provides to the chat or recieves chat messages")
    ),
    path = "/ws/{type_of_client}",
    responses(
        (status = 101, description = "Moving over from http to the websocket"),
    ),
    tag = "chat_endpoints",
)]
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

                let Some(onboard_fields_json) = msg.find(": ").map(|x| &msg[x + 2..]) else {
                    return;
                };

                let Ok(onbaord_fields) = serde_json::from_str::<OnboardFields>(onboard_fields_json) else {return ;};

                info!("{:?} is conected to the chat", onbaord_fields.username);

                let global_tx = state.global_channel;

                while let Some(Ok(msg)) = socket.recv().await {
                    match msg {
                        Text(some_text) => {
                            let plaintext = some_text.to_string();

                            let fields = onbaord_fields.clone();

                            let chat = ChatMessage {
                                username: fields.username,
                                username_color: fields.username_color,
                                message: plaintext,
                                message_font: fields.message_font,
                            };

                            if global_tx.send(chat).is_err() {
                                return;
                            }
                        }
                        _ => {
                            return;
                        }
                    }
                }

                info!("{:?} is disconnect to the chat", onbaord_fields.username);
            }
            ClientType::ChatFrontend => {
                let mut chat_rx = state.global_channel.subscribe();

                while let Ok(msg) = chat_rx.recv().await {
                    let Some(chat) = serde_json::to_string(&msg).ok() else { return;};
                    if socket.send(Message::Text(chat.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    })
}
