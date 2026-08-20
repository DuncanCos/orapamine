pub mod logic;
pub mod protocol;
pub mod state;
pub mod ws;

use axum::routing::get;
use axum::Router;
use state::AppState;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

/// Construit le routeur axum : le point d'entrée WebSocket `/ws`, et les
/// fichiers statiques du client compilé en repli (SPA : tout chemin
/// inconnu retombe sur `index.html`).
pub fn build_router(state: Arc<AppState>, static_dir: &str) -> Router {
    let index = format!("{static_dir}/index.html");
    let serve_dir = ServeDir::new(static_dir).not_found_service(ServeFile::new(index));

    Router::new()
        .route("/ws", get(ws::ws_handler))
        .fallback_service(serve_dir)
        .with_state(state)
}
