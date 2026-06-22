use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioProgress {
    pub id: Uuid,
    pub user_id: String,
    pub audio_id: String,
    pub position: f64,
    pub duration: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SaveProgressRequest {
    pub user_id: String,
    pub audio_id: String,
    pub position: f64,
    pub duration: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ProgressResponse {
    pub user_id: String,
    pub audio_id: String,
    pub position: f64,
    pub duration: f64,
    pub percentage: f64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: String::new(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: message.to_string(),
        }
    }
}

impl From<AudioProgress> for ProgressResponse {
    fn from(p: AudioProgress) -> Self {
        let percentage = if p.duration > 0.0 {
            (p.position / p.duration) * 100.0
        } else {
            0.0
        };
        Self {
            user_id: p.user_id,
            audio_id: p.audio_id,
            position: p.position,
            duration: p.duration,
            percentage,
            updated_at: p.updated_at,
        }
    }
}
