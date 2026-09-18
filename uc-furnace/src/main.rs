use solverforge_furnace::api;

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
    println!("SolverForge Furnace listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
