/* solverforge-fsr — Axum server and static browser shell.

The binary serves the SolverForge UI assets, this app's static files, and the
retained-job API from one process so the Hugging Face Docker Space only needs a
single `PORT` binding. */

use solverforge_fsr::api;

use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    solverforge::console::init();

    let state = Arc::new(api::AppState::new());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = api::router(state)
        .merge(solverforge_ui::routes())
        .fallback_service(ServeDir::new("static"))
        .layer(cors);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(7860);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("▸ solverforge-fsr listening on http://{}", addr);
    println!("▸ Open http://localhost:{} in your browser\n", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("▸ solverforge-fsr shutting down");
}
