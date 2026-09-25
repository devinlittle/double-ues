use tokio::net::TcpListener;
use tracing::{Level, info};
mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let router = routes::create_router();
    let listener = TcpListener::bind(":::5252").await.unwrap();
    let addr = &listener.local_addr().unwrap();

    info!("Listening on {:?}", addr);

    axum::serve(listener, router).await.unwrap()
}
