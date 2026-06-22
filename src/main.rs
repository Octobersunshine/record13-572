mod db;
mod handlers;
mod models;
mod routes;

use anyhow::Result;
use db::ProgressStore;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub store: ProgressStore,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("audio_progress_tracker=info,tower_http=info")),
        )
        .init();

    let store = ProgressStore::new()?;
    let state = Arc::new(AppState { store });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = routes::create_routes(state.clone());
    let app = api_routes
        .layer(cors)
        .nest_service("/", ServeDir::new("static"));

    let addr = "0.0.0.0:3000";
    info!("Server starting on http://{}", addr);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
