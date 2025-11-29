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
    #[error("Cache error: {0}")]
    Cache(anyhow::Error),
    #[error("Unknown error: {0}")]
    Unknown(anyhow::Error),
}

#[derive(Debug,Clone,From,Into,Deref)]
pub struct  EncodedAudio(Vec<u8>);

#[derive(Debug,Clone,From,Into,Deref)]
pub struct PcmAudio(Vec<u8>);

/// # Source: the boundary of external/internal voice generator.
///
/// Since this trait allow various format,
/// we have to normalize the output with transcoder to
/// final codec(pcm) to play in discord voice channels.
#[async_trait]
pub trait Source: Send + Sync{
    fn identifier(&self) -> &str;

    async fn generate(&self, text: &str) -> Result<EncodedAudio, VoiceError>;
}

/// # Voice: normalized pcm ready to play with songbird
///
/// Contrary with Source, Voice is a normalized audio
/// trans-coded from source result.
#[async_trait]
pub trait Voice: Send + Sync{
    fn identifier(&self) -> &str;

    async fn generate(&self, text: &str) -> Result<PcmAudio, VoiceError>;
}
