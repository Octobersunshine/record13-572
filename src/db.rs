use anyhow::{Context, Result};
use chrono::Utc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{AudioProgress, MergedProgressResponse, ProgressResponse};

const DATA_FILE: &str = "audio_progress.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProgressData {
    progresses: HashMap<String, AudioProgress>,
}

fn device_key(user_id: &str, audio_id: &str, device_id: &str) -> String {
    format!("{}:{}:{}", user_id, audio_id, device_id)
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
        device_id: &str,
        position: f64,
        duration: Option<f64>,
    ) -> Result<AudioProgress> {
        let mut data = self.data.write();
        let k = device_key(user_id, audio_id, device_id);
        let now = Utc::now();

        let progress = if let Some(existing) = data.progresses.get_mut(&k) {
            if position > existing.position {
                existing.position = position;
            }
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
                device_id: device_id.to_string(),
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

    pub fn force_save(
        &self,
        user_id: &str,
        audio_id: &str,
        device_id: &str,
        position: f64,
        duration: Option<f64>,
    ) -> Result<AudioProgress> {
        let mut data = self.data.write();
        let now = Utc::now();
        let dur = duration.unwrap_or(0.0);

        let prefix = format!("{}:{}:", user_id, audio_id);
        let keys_to_clear: Vec<String> = data
            .progresses
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();
        for k in keys_to_clear {
            data.progresses.remove(&k);
        }

        let k = device_key(user_id, audio_id, device_id);
        let new_progress = AudioProgress {
            id: Uuid::new_v4(),
            user_id: user_id.to_string(),
            audio_id: audio_id.to_string(),
            device_id: device_id.to_string(),
            position,
            duration: dur,
            created_at: now,
            updated_at: now,
        };
        data.progresses.insert(k, new_progress.clone());

        self.persist(&data)?;
        Ok(new_progress)
    }

    pub fn get_device(
        &self,
        user_id: &str,
        audio_id: &str,
        device_id: &str,
    ) -> Option<AudioProgress> {
        let data = self.data.read();
        let k = device_key(user_id, audio_id, device_id);
        data.progresses.get(&k).cloned()
    }

    pub fn get_merged(
        &self,
        user_id: &str,
        audio_id: &str,
    ) -> Option<MergedProgressResponse> {
        let devices: Vec<ProgressResponse> = self
            .list_devices(user_id, audio_id)
            .into_iter()
            .map(ProgressResponse::from)
            .collect();

        if devices.is_empty() {
            return None;
        }

        let best = devices
            .iter()
            .max_by(|a, b| {
                a.position
                    .partial_cmp(&b.position)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        Some(MergedProgressResponse {
            user_id: user_id.to_string(),
            audio_id: audio_id.to_string(),
            position: best.position,
            duration: best.duration,
            percentage: best.percentage,
            updated_at: best.updated_at,
            source_device: best.device_id.clone(),
            devices,
        })
    }

    pub fn list_devices(&self, user_id: &str, audio_id: &str) -> Vec<AudioProgress> {
        let data = self.data.read();
        let mut result: Vec<AudioProgress> = data
            .progresses
            .values()
            .filter(|p| p.user_id == user_id && p.audio_id == audio_id)
            .cloned()
            .collect();
        result.sort_by(|a, b| {
            b.position
                .partial_cmp(&a.position)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        result
    }

    pub fn list_by_user(&self, user_id: &str) -> Vec<MergedProgressResponse> {
        let data = self.data.read();

        let mut audio_ids: Vec<String> = data
            .progresses
            .values()
            .filter(|p| p.user_id == user_id)
            .map(|p| p.audio_id.clone())
            .collect();
        audio_ids.sort();
        audio_ids.dedup();

        audio_ids
            .into_iter()
            .filter_map(|aid| {
                let devices: Vec<ProgressResponse> = data
                    .progresses
                    .values()
                    .filter(|p| p.user_id == user_id && p.audio_id == aid)
                    .cloned()
                    .map(ProgressResponse::from)
                    .collect();

                if devices.is_empty() {
                    return None;
                }

                let best = devices
                    .iter()
                    .max_by(|a, b| {
                        a.position
                            .partial_cmp(&b.position)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap();

                Some(MergedProgressResponse {
                    user_id: user_id.to_string(),
                    audio_id: aid,
                    position: best.position,
                    duration: best.duration,
                    percentage: best.percentage,
                    updated_at: best.updated_at,
                    source_device: best.device_id.clone(),
                    devices,
                })
            })
            .collect()
    }

    pub fn delete_device(
        &self,
        user_id: &str,
        audio_id: &str,
        device_id: &str,
    ) -> Result<bool> {
        let mut data = self.data.write();
        let k = device_key(user_id, audio_id, device_id);
        let removed = data.progresses.remove(&k).is_some();
        if removed {
            self.persist(&data)?;
        }
        Ok(removed)
    }

    pub fn delete_all(&self, user_id: &str, audio_id: &str) -> Result<usize> {
        let mut data = self.data.write();
        let keys_to_remove: Vec<String> = data
            .progresses
            .keys()
            .filter(|k| {
                let parts: Vec<&str> = k.split(':').collect();
                parts.len() >= 2 && parts[0] == user_id && parts[1] == audio_id
            })
            .cloned()
            .collect();
        let count = keys_to_remove.len();
        for k in keys_to_remove {
            data.progresses.remove(&k);
        }
        if count > 0 {
            self.persist(&data)?;
        }
        Ok(count)
    }
}
