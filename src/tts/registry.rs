use crate::config::{AppConfig, CacheConfig, ProfileConfig};
use crate::tts::cache::CachedVoice;
use crate::tts::google_cloud::GoogleCloudVoice;
use crate::tts::Voice;
use anyhow::Context;
use google_cloud_texttospeech_v1::client::TextToSpeech;
use moka::future::Cache;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct VoiceRegistry {
    voices: Arc<HashMap<String, Arc<dyn Voice>>>,
}

impl VoiceRegistry {
    pub fn builder(config: AppConfig) -> VoiceRegistryBuilder {
        VoiceRegistryBuilder::new(config)
    }

    pub fn new(voices: HashMap<String, Arc<dyn Voice>>) -> Self {
        Self {
            voices: Arc::new(voices),
        }
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn Voice>> {
        self.voices.get(id).cloned()
    }
}

pub struct VoiceRegistryBuilder {
    config: AppConfig,
    moka_cache: Option<Cache<String, Vec<u8>>>,
    google_cloud: Option<TextToSpeech>,
}

impl VoiceRegistryBuilder {
    fn new(config: AppConfig) -> Self {
        let moka_cache = match &config.cache {
            CacheConfig::InMemory(c) => {
                Some(Cache::new(c.capacity))
            },
            _ => None,
        };

        Self {
            config,
            moka_cache,
            google_cloud: None,
        }
    }

    pub fn google_cloud(mut self, google_cloud: TextToSpeech) -> Self {
        self.google_cloud = Some(google_cloud);
        self
    }

    pub fn build(self) -> anyhow::Result<VoiceRegistry> {
        let mut voices = HashMap::new();

        for (id, preset) in &self.config.profiles {
            let voice: Arc<dyn Voice> = match preset {
                ProfileConfig::GoogleCloudVoice(c) => {
                    let client = self.google_cloud.as_ref()
                        .with_context(|| format!(
                            "Preset '{}' requires Google Cloud backend, but it is not configured. Please verify that [backend.google_cloud] exists and 'enabled = true' in config.toml.",
                            id
                        ))?
                        .clone();

                    self.wrap_with_cache(Box::new(GoogleCloudVoice::new(client, c.clone())))
                },
            };

            voices.insert(id.to_string(), voice);
        }

        Ok(VoiceRegistry::new(voices))
    }

    fn wrap_with_cache(&self, voice: Box<dyn Voice>) -> Arc<dyn Voice> {
        match &self.config.cache {
            CacheConfig::Disabled => Arc::from(voice),
            CacheConfig::InMemory(_) => Arc::new(CachedVoice::new(voice, self.moka_cache.as_ref().expect("moka cache must be set").clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BotConfig, CacheConfig, DatabaseConfig, DatabaseKind, InMemoryCacheConfig};
    use crate::tts::google_cloud::GoogleCloudVoiceConfig;

    fn create_test_config(cache: CacheConfig) -> AppConfig {
        let mut presets = HashMap::new();
        presets.insert(
            "test_preset".to_string(),
            ProfileConfig::GoogleCloudVoice(GoogleCloudVoiceConfig {
                language_code: "ja-JP".to_string(),
                name: Some("ja-JP-Wavenet-A".to_string()),
                ..Default::default()
            }),
        );

        AppConfig {
            bot: Default::default(),
            database: DatabaseConfig{
                kind: DatabaseKind::SQLite,
                url: "".to_string(),
            },
            backend: Default::default(),
            cache,
            profiles: presets,
        }
    }

    async fn create_dummy_client() -> TextToSpeech {
        TextToSpeech::builder().with_endpoint("http://localhost:0").build().await.unwrap()
    }

    #[tokio::test]
    async fn test_build_with_cache_enabled() {
        let config = create_test_config(CacheConfig::InMemory(InMemoryCacheConfig { capacity: 100 }));
        let client = create_dummy_client().await;

        let registry = VoiceRegistry::builder(config)
            .google_cloud(client)
            .build()
            .expect("Should build successfully");

        let voice = registry.get("test_preset").expect("Preset should exist");

        assert!(voice.identifier().starts_with("cached"), "ID should start with cached: {}", voice.identifier());
        assert!(voice.identifier().contains("google"), "ID should contain internal voice id");
    }

    #[tokio::test]
    async fn test_build_with_cache_disabled() {
        let config = create_test_config(CacheConfig::Disabled);
        let client = create_dummy_client().await;

        let registry = VoiceRegistry::builder(config)
            .google_cloud(client)
            .build()
            .expect("Should build successfully");

        let voice = registry.get("test_preset").expect("Preset should exist");

        assert!(!voice.identifier().starts_with("cached"), "ID should NOT start with cached: {}", voice.identifier());
        assert!(voice.identifier().starts_with("google"), "ID should start directly with google");
    }

    #[tokio::test]
    async fn test_build_fail_missing_client() {
        let config = create_test_config(CacheConfig::Disabled);

        // build without client
        let result = VoiceRegistry::builder(config)
            .build();
        assert!(result.is_err());
    }
}