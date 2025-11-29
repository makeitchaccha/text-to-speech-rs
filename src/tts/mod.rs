mod cache;
mod google;

use async_trait::async_trait;
use derive_more::Into;
use derive_more::with_trait::{Deref, DerefMut, From};
use thiserror::Error;

const DISCORD_SAMPLE_RATE: i32 = 48_000;

#[derive(Debug, Error)]
pub enum VoiceError {
    #[error("API request failed: {0}")]
    Api(anyhow::Error),
    #[error("Decoding error: {0}")]
    Decoding(anyhow::Error),
    #[error("Cache error: {0}")]
    Cache(anyhow::Error),
    #[error("Unknown error: {0}")]
    Unknown(anyhow::Error),
}

/// # Voice: normalized pcm ready to play with songbird
///
/// Contrary with Source, Voice is a normalized audio
/// trans-coded from source result.
#[async_trait]
pub trait Voice: Send + Sync{
    fn identifier(&self) -> &str;

    async fn generate(&self, text: &str) -> Result<Vec<u8>, VoiceError>;
}
