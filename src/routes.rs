use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::{handlers, AppState};

pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/progress", post(handlers::save_progress))
        .route("/api/progress/force", post(handlers::force_sync_progress))
        .route(
            "/api/progress/:user_id/:audio_id",
            get(handlers::get_progress).delete(handlers::delete_progress),
        )
        .route(
            "/api/progress/:user_id/:audio_id/:device_id",
            get(handlers::get_device_progress).delete(handlers::delete_device_progress),
        )
        .route("/api/progress/:user_id", get(handlers::list_progress))
        .with_state(state)
}
