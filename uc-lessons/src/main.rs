//! Axum entrypoint for the lesson-timetabling app.
//!
//! Run with `make run-release`, then open the printed local URL. The same
//! binary is used by the Docker Space image, where `PORT` is provided by the
//! platform.

use solverforge_lessons::api;

use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Use the stock SolverForge console logger so solve progress appears in
    // local runs and Space container logs.
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

    // Hugging Face Spaces inject `PORT`; 7860 remains the local default used in
    // docs, tests, and the Makefile.
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(7860);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("▸ solverforge-lessons listening on http://{}", addr);
    println!("▸ Open http://localhost:{} in your browser\n", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
