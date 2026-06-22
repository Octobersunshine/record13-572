use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

use crate::AppState;
use crate::models::{
    ApiResponse, MergedProgressResponse, ProgressResponse, SaveProgressRequest,
};

pub async fn save_progress(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SaveProgressRequest>,
) -> (StatusCode, Json<ApiResponse<MergedProgressResponse>>) {
    if payload.user_id.is_empty() || payload.audio_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("user_id and audio_id are required")),
        );
    }

    if payload.device_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("device_id is required for multi-device sync")),
        );
    }

    if payload.position < 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("position must be non-negative")),
        );
    }

    let save_result = state.store.save(
        &payload.user_id,
        &payload.audio_id,
        &payload.device_id,
        payload.position,
        payload.duration,
    );

    match save_result {
        Ok(_) => {
            let merged = state.store.get_merged(&payload.user_id, &payload.audio_id);
            match merged {
                Some(m) => (StatusCode::OK, Json(ApiResponse::ok(m))),
                None => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error("Failed to retrieve merged progress after save")),
                ),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Storage error: {}", e))),
        ),
    }
}

pub async fn force_sync_progress(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SaveProgressRequest>,
) -> (StatusCode, Json<ApiResponse<MergedProgressResponse>>) {
    if payload.user_id.is_empty() || payload.audio_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("user_id and audio_id are required")),
        );
    }

    if payload.device_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("device_id is required for force sync")),
        );
    }

    if payload.position < 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("position must be non-negative")),
        );
    }

    let save_result = state.store.force_save(
        &payload.user_id,
        &payload.audio_id,
        &payload.device_id,
        payload.position,
        payload.duration,
    );

    match save_result {
        Ok(_) => {
            let merged = state.store.get_merged(&payload.user_id, &payload.audio_id);
            match merged {
                Some(m) => (StatusCode::OK, Json(ApiResponse::ok(m))),
                None => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error("Failed to retrieve merged progress after force sync")),
                ),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Storage error: {}", e))),
        ),
    }
}

pub async fn get_progress(
    State(state): State<Arc<AppState>>,
    Path((user_id, audio_id)): Path<(String, String)>,
) -> (StatusCode, Json<ApiResponse<MergedProgressResponse>>) {
    match state.store.get_merged(&user_id, &audio_id) {
        Some(merged) => (StatusCode::OK, Json(ApiResponse::ok(merged))),
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(
                "No progress found for this user and audio on any device",
            )),
        ),
    }
}

pub async fn get_device_progress(
    State(state): State<Arc<AppState>>,
    Path((user_id, audio_id, device_id)): Path<(String, String, String)>,
) -> (StatusCode, Json<ApiResponse<ProgressResponse>>) {
    match state.store.get_device(&user_id, &audio_id, &device_id) {
        Some(progress) => (StatusCode::OK, Json(ApiResponse::ok(ProgressResponse::from(progress)))),
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(
                "No progress found for this user, audio and device",
            )),
        ),
    }
}

pub async fn list_progress(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> (StatusCode, Json<ApiResponse<Vec<MergedProgressResponse>>>) {
    let merged_list = state.store.list_by_user(&user_id);
    (StatusCode::OK, Json(ApiResponse::ok(merged_list)))
}

pub async fn delete_progress(
    State(state): State<Arc<AppState>>,
    Path((user_id, audio_id)): Path<(String, String)>,
) -> (StatusCode, Json<ApiResponse<usize>>) {
    match state.store.delete_all(&user_id, &audio_id) {
        Ok(0) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("No progress found to delete")),
        ),
        Ok(count) => (StatusCode::OK, Json(ApiResponse::ok(count))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Storage error: {}", e))),
        ),
    }
}

pub async fn delete_device_progress(
    State(state): State<Arc<AppState>>,
    Path((user_id, audio_id, device_id)): Path<(String, String, String)>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    match state.store.delete_device(&user_id, &audio_id, &device_id) {
        Ok(true) => (StatusCode::OK, Json(ApiResponse::ok(()))),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("No progress found for this device to delete")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Storage error: {}", e))),
        ),
    }
}
