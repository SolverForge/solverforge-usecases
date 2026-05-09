//! SolverForge Hospital - Axum Server
//!
//! Run with: cargo run --release --bin solverforge-hospital
//! Then open: http://localhost:7860

use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use solverforge_hospital::api;

#[tokio::main]
async fn main() {
    // Enable the stock SolverForge console logger so local runs show phase and
    // score progress without any app-specific logging glue.
    solverforge::console::init();

    let state = Arc::new(api::AppState::new());

    // The example keeps CORS permissive because it is primarily a local demo
    // app. Production deployments would usually lock this down.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // The router combines our backend routes, the shared SolverForge UI routes,
    // and the local static files that boot the browser app.
    let app = api::router(state)
        .merge(solverforge_ui::routes())
        .fallback_service(ServeDir::new("static"))
        .layer(cors);

    // `PORT` keeps the app easy to host on demo platforms, while `7860` stays
    // as the predictable local default advertised in the docs.
    let port = std::env::var("PORT")
        .ok()
        .and_then(|raw| raw.parse::<u16>().ok())
        .unwrap_or(7860);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("SolverForge Hospital listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
