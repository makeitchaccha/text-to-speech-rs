use async_trait::async_trait;
use moka::future::Cache;
use sha2::Digest;
use sha2::digest::Update;
use crate::tts::{Voice, VoiceError};

pub struct CachedVoice {
    identifier: String,
    inner: Box<dyn Voice>,
    cache: Cache<String, Vec<u8>>,
}

impl CachedVoice {
    pub fn new(inner: Box<dyn Voice>, cache: Cache<String, Vec<u8>>) -> Self {
        Self {
            identifier: format!("cached-{}", inner.identifier()),
            inner,
            cache,
        }
    }
}

#[async_trait]
impl Voice for CachedVoice {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    async fn generate(&self, text: &str) -> Result<Vec<u8>, VoiceError> {
        let key = hex::encode(sha2::Sha256::new().chain(self.identifier.as_bytes()).chain(text.as_bytes()).finalize());

        if let Some(data) = self.cache.get(&key).await {
            return Ok(data)
        }

        let data = self.inner.generate(text).await?;

        self.cache.insert(key, data.clone()).await;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use super::*;

    struct MockVoice {
        call_count: Arc<AtomicUsize>,
    }

    impl MockVoice {
        fn new() -> Self {
            Self { call_count: Arc::new(AtomicUsize::new(0)) }
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

    #[tokio::test]
    async fn test_cache_hit() {
        let mock = MockVoice::new();
        let call_count = mock.call_count.clone();

        let cached_voice = CachedVoice::new(Box::new(mock), Cache::new(100));

        let text = "hello";

        // in case of same text
        let result = cached_voice.generate(text).await.unwrap();
        assert_eq!(result.to_vec(), b"hello");
        assert_eq!(call_count.load(Ordering::SeqCst), 1, "First call should hit the inner voice");

        let result = cached_voice.generate(text).await.unwrap();
        assert_eq!(result.to_vec(), b"hello");
        assert_eq!(call_count.load(Ordering::SeqCst), 1, "Second call should hit the cache");

        // different text
        let _ = cached_voice.generate("world").await;
        assert_eq!(call_count.load(Ordering::SeqCst), 2, "New text should hit the inner voice");
    }
}