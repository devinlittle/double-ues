use std::sync::Arc;

use axum::{Router, http::StatusCode, routing::get};
use tokio::sync::broadcast;

mod chat;

#[derive(Clone)]
pub struct AppState {
    pub global_channel: Arc<broadcast::Sender<String>>,
}

pub fn create_router() -> Router {
    let global_channel = Arc::new(broadcast::Sender::new(256));

    let app_state = AppState { global_channel };

    Router::new()
        .route("/health", get(health))
        .route("/ws/{type_of_client}", get(chat::chat_ws))
        .with_state(app_state)
}

pub async fn health() -> Result<(), StatusCode> {
    Ok(())
}
