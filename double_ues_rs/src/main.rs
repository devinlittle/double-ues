use tokio::{net::TcpListener, signal};
use tracing::info;
use tracing_subscriber::EnvFilter;

mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let router = routes::create_router();
    let listener = TcpListener::bind(":::5252").await.unwrap();
    let addr = &listener.local_addr().unwrap();

    info!("Listening on {:?}", addr);

    let _ = axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await;
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("marlon...GET HIM bc he `failed to install the SIGTERM handler 🥲`")
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("marlon...GET HIM bc he `failed to install the SIGTERM handler 🥲` but its alr because they are using Unix")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
