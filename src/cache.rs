use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::transcript::Conversation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TranscriptCacheKey {
    pub transcript_hash: String,
}

pub struct TranscriptCache {
    cache_dir: PathBuf,
}

impl TranscriptCache {
    pub async fn new() -> Result<Self> {
        let cache_dir = get_cache_directory()?;

        tokio::fs::create_dir_all(&cache_dir)
            .await
            .context("Failed to create cache directory")?;

        Ok(Self { cache_dir })
    }

    pub async fn get(&self, key: &TranscriptCacheKey) -> Option<Conversation> {
        match cacache::read(&self.cache_dir, &key.transcript_hash).await {
            Ok(data) => serde_json::from_slice(&data).ok(),
            Err(_) => None,
        }
    }

    pub async fn insert(&self, key: TranscriptCacheKey, value: Conversation) {
        if let Ok(serialized) = serde_json::to_vec(&value) {
            let _ = cacache::write(&self.cache_dir, &key.transcript_hash, serialized).await;
        }
    }
}

fn get_cache_directory() -> Result<PathBuf> {
    let cache_base = dirs::cache_dir().context("Failed to determine cache directory")?;
    Ok(cache_base.join(env!("CARGO_CRATE_NAME")))
}

pub fn compute_transcript_hash(transcript: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(transcript);
    format!("{:x}", hasher.finalize())
}
