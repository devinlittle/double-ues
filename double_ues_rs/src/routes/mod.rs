use axum::{Json, Router, http::StatusCode, routing::get};
use dashmap::DashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use uuid::Uuid;

use crate::routes::chat::{ChatMessage, UserFields};

mod chat;
mod commands;

#[derive(OpenApi)]
#[openapi(
      paths(
        crate::routes::health,
        // Chat paths
        crate::routes::chat::chat_ws,
    ),
    components(
        schemas(
            crate::routes::chat::ClientType,
            crate::routes::chat::RgbColor,
            crate::routes::chat::ChatMessage,
            crate::routes::chat::OkayMessage,
            crate::routes::chat::UserFields,
        )
    ),
    tags(
        (name = "chat_endpoints", description = "Authentication endpoints"),
        (name = "non_chat_related", description = "internal stuff like a health check endpoint"),
    )
)]
pub struct DaApiDoc;

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is alive!!"),
    ),
    tag = "non_chat_related"
)]
pub async fn health() -> Result<(), StatusCode> {
    Ok(())
}

type GlobalChannel = Arc<broadcast::Sender<ChatMessage>>;
type UserSettings = Arc<DashMap<Uuid, Arc<RwLock<UserFields>>>>;

#[derive(Clone)]
pub struct AppState {
    pub global_channel: GlobalChannel,
    pub users: UserSettings,
}

pub fn create_router() -> Router {
    let global_channel = Arc::new(broadcast::Sender::new(256));
    let users = Arc::new(DashMap::new());

    let app_state = AppState {
        global_channel,
        users,
    };

    let openapi = DaApiDoc::openapi();

    Router::new()
        .route("/health", get(health))
        .route("/ws/{type_of_client}", get(chat::chat_ws))
        .route(
            "/api-docs/openapi.json",
            get({
                let json_spec = openapi.clone();
                move || async { Json(json_spec) }
            }),
        )
        .merge(Scalar::with_url("/scalar", openapi))
        .with_state(app_state)
}
