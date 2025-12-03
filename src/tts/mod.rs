mod cache;
pub mod google_cloud;
pub mod registry;

use async_trait::async_trait;
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

/// # Voice: normalized pcm ready to play with songbird
///
/// Contrary with Source, Voice is a normalized audio
/// trans-coded from source result.
#[async_trait]
pub trait Voice: Send + Sync{
    fn identifier(&self) -> &str;

    async fn generate(&self, text: &str) -> Result<Vec<u8>, VoiceError>;
}

#[cfg(test)]
pub mod test_utils {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use async_trait::async_trait;
    use crate::tts::{Voice, VoiceError};

    #[derive(Clone)]
    pub struct MockVoice {
        call_count: Arc<AtomicUsize>,
    }

    impl MockVoice {
        pub fn new() -> Self {
            Self { call_count: Arc::new(AtomicUsize::new(0)) }
        }

        pub fn call_count(&self) -> usize {
            self.call_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl Voice for MockVoice {
        fn identifier(&self) -> &str {
            "mock"
        }

        async fn generate(&self, text: &str) -> Result<Vec<u8>, VoiceError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(text.as_bytes().to_vec())
        }
    }
}