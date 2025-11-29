use async_trait::async_trait;
use moka::future::Cache;
use crate::tts::{Voice, VoiceError};

pub struct CachedVoice<T: Voice> {
    identifier: String,
    inner: T,
    cache: Cache<String, Vec<u8>>,
}

impl<T: Voice> CachedVoice<T> {
    pub fn new(inner: T, capacity: u64) -> Self {
        Self {
            identifier: format!("cached-{}", inner.identifier()),
            inner,
            cache: Cache::new(capacity)
        }
    }
}

#[async_trait]
impl<T: Voice> Voice for CachedVoice<T> {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    async fn generate(&self, text: &str) -> Result<Vec<u8>, VoiceError> {
        if let Some(data) = self.cache.get(text).await {
            return Ok(data)
        }

        let data = self.inner.generate(text).await?;

        self.cache.insert(text.to_owned(), data.clone()).await;

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

        let cached_voice = CachedVoice::new(mock, 100);

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