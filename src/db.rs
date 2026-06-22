use anyhow::{Context, Result};
use chrono::Utc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::AudioProgress;

const DATA_FILE: &str = "audio_progress.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProgressData {
    progresses: HashMap<String, AudioProgress>,
}

fn key(user_id: &str, audio_id: &str) -> String {
    format!("{}:{}", user_id, audio_id)
}

#[derive(Clone)]
pub struct ProgressStore {
    data: Arc<RwLock<ProgressData>>,
    file_path: PathBuf,
}

impl ProgressStore {
    pub fn new() -> Result<Self> {
        let file_path = PathBuf::from(DATA_FILE);
        let data = if file_path.exists() {
            let content = std::fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read {}", DATA_FILE))?;
            if content.trim().is_empty() {
                ProgressData::default()
            } else {
                serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse {}", DATA_FILE))?
            }
        } else {
            ProgressData::default()
        };

        Ok(Self {
            data: Arc::new(RwLock::new(data)),
            file_path,
        })
    }

    fn persist(&self, data: &ProgressData) -> Result<()> {
        let content = serde_json::to_string_pretty(data)?;
        std::fs::write(&self.file_path, content)
            .with_context(|| format!("Failed to write {}", DATA_FILE))?;
        Ok(())
    }

    pub fn save(
        &self,
        user_id: &str,
        audio_id: &str,
        position: f64,
        duration: Option<f64>,
    ) -> Result<AudioProgress> {
        let mut data = self.data.write();
        let k = key(user_id, audio_id);
        let now = Utc::now();

        let progress = if let Some(existing) = data.progresses.get_mut(&k) {
            existing.position = position;
            if let Some(dur) = duration {
                if dur > 0.0 {
                    existing.duration = dur;
                }
            }
            existing.updated_at = now;
            existing.clone()
        } else {
            let new_progress = AudioProgress {
                id: Uuid::new_v4(),
                user_id: user_id.to_string(),
                audio_id: audio_id.to_string(),
                position,
                duration: duration.unwrap_or(0.0),
                created_at: now,
                updated_at: now,
            };
            data.progresses.insert(k, new_progress.clone());
            new_progress
        };

        self.persist(&data)?;
        Ok(progress)
    }

    pub fn get(&self, user_id: &str, audio_id: &str) -> Option<AudioProgress> {
        let data = self.data.read();
        let k = key(user_id, audio_id);
        data.progresses.get(&k).cloned()
    }

    pub fn list_by_user(&self, user_id: &str) -> Vec<AudioProgress> {
        let data = self.data.read();
        let mut result: Vec<AudioProgress> = data
            .progresses
            .values()
            .filter(|p| p.user_id == user_id)
            .cloned()
            .collect();
        result.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        result
    }

    pub fn delete(&self, user_id: &str, audio_id: &str) -> Result<bool> {
        let mut data = self.data.write();
        let k = key(user_id, audio_id);
        let removed = data.progresses.remove(&k).is_some();
        if removed {
            self.persist(&data)?;
        }
        Ok(removed)
    }
}
