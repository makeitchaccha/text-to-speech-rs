mod cache;

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VoiceError {
    #[error("API request failed: {0}")]
    Api(anyhow::Error),
    #[error("Cache error: {0}")]
    Cache(anyhow::Error),
    #[error("Unknown error: {0}")]
    Unknown(anyhow::Error),
}

#[async_trait]
pub trait Voice: Send + Sync{
    // identifier, used for logging and cache key.
    // this must be distinct from other voice sources.
    // e.g. "google(ja-JP-Wavenet-A, speed:1.2)"
    fn identifier(&self) -> &str;

    async fn generate(&self, text: &str) -> Result<Vec<u8>, VoiceError>;
}