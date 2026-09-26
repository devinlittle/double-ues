use axum::{
    extract::{
        ws::Message::{self, Text},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
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
pub struct UserFields {
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

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct OkayMessage {
    pub ok: ThisIsOk,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub enum ThisIsOk {
    OK,
}

static SYSTEM_USERNAME: &str = "SYSTEM";
static SYSTEM_FONT: &str = "Impact";

impl ChatMessage {
    pub fn system_chat(message: String) -> ChatMessage {
        Self {
            username: SYSTEM_USERNAME.to_string(),
            username_color: HexColor(255, 179, 0),
            message,
            message_font: SYSTEM_FONT.to_string(),
        }
    }
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

                let Some(user_fields_json) = msg.find(": ").map(|x| &msg[x + 2..]) else {
                    return;
                };

                let Ok(user_fields) = serde_json::from_str::<UserFields>(user_fields_json) else {return ;};
                let their_uuid = uuid::Uuid::new_v4();
                state.users.insert(their_uuid, Arc::new(RwLock::new(user_fields)));

                let user_settings = if let Some(read) = state.users.get(&their_uuid) {
                    let read = match read.read() {
                        Ok(read) => read,
                        Err(poisoned) => {
                            tracing::error!("LOCK POISONED FOR USER {}", their_uuid);
                            poisoned.into_inner()
                        }
                    };

                    read.clone()
                } else {
                    return;
                };


                let global_tx = state.global_channel;

                let announcement = format!("{:?} joined the chat", user_settings.username);

                info!(announcement);
                let _ =  global_tx.send(ChatMessage::system_chat(announcement));

                let Ok(okay_string) = serde_json::to_string(&OkayMessage {ok: ThisIsOk::OK}) else {return;};
                let _ = socket.send(okay_string.into()).await;

                while let Some(Ok(msg)) = socket.recv().await {
                    match msg {
                        Text(some_text) => {
                            // TODO: have a command system implimented, where users can change
                            // things with the stream/their onboard_fields

                            let plaintext = some_text.to_string();

                            let fields = user_settings.clone();

                            let chat = ChatMessage {
                                username: fields.username,
                                username_color: fields.username_color,
                                message: plaintext,
                                message_font: fields.message_font,
                            };

                             let _  = global_tx.send(chat);
                        }
                        _ => {
                            break;
                        }
                    }
                }


                let announcement = format!("{:?} left the chat", user_settings.username); 

                info!(announcement);
                if global_tx.send(ChatMessage::system_chat(announcement)).is_err() {
                    return;
                }
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
