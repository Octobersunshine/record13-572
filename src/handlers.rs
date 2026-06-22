use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

use crate::AppState;
use crate::models::{
    ApiResponse, ProgressResponse, SaveProgressRequest,
};

pub async fn save_progress(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SaveProgressRequest>,
) -> (StatusCode, Json<ApiResponse<ProgressResponse>>) {
    if payload.user_id.is_empty() || payload.audio_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("user_id and audio_id are required")),
        );
    }

    if payload.position < 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("position must be non-negative")),
        );
    }

    let result = state.store.save(
        &payload.user_id,
        &payload.audio_id,
        payload.position,
        payload.duration,
    );

    match result {
        Ok(progress) => (
            StatusCode::OK,
            Json(ApiResponse::ok(ProgressResponse::from(progress))),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Storage error: {}", e))),
        ),
    }
}

pub async fn get_progress(
    State(state): State<Arc<AppState>>,
    Path((user_id, audio_id)): Path<(String, String)>,
) -> (StatusCode, Json<ApiResponse<ProgressResponse>>) {
    match state.store.get(&user_id, &audio_id) {
        Some(progress) => (
            StatusCode::OK,
            Json(ApiResponse::ok(ProgressResponse::from(progress))),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("No progress found for this user and audio")),
        ),
    }
}

pub async fn list_progress(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> (StatusCode, Json<ApiResponse<Vec<ProgressResponse>>>) {
    let progresses = state.store.list_by_user(&user_id);
    let responses: Vec<ProgressResponse> = progresses
        .into_iter()
        .map(ProgressResponse::from)
        .collect();
    (StatusCode::OK, Json(ApiResponse::ok(responses)))
}

pub async fn delete_progress(
    State(state): State<Arc<AppState>>,
    Path((user_id, audio_id)): Path<(String, String)>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    match state.store.delete(&user_id, &audio_id) {
        Ok(true) => (StatusCode::OK, Json(ApiResponse::ok(()))),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("No progress found to delete")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Storage error: {}", e))),
        ),
    }
}
