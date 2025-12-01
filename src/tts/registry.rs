use std::collections::HashMap;
use std::sync::Arc;
use anyhow::anyhow;
use google_cloud_texttospeech_v1::client::TextToSpeech;
use crate::config::{AppConfig, CacheConfig, PresetConfig};
use crate::tts::cache::CachedVoice;
use crate::tts::google_cloud::GoogleCloudVoice;
use crate::tts::Voice;

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
    google_cloud: Option<TextToSpeech>,
}

impl VoiceRegistryBuilder {
    fn new(config: AppConfig) -> Self {
        Self {
            config,
            google_cloud: None,
        }
    }

    pub fn google_cloud(mut self, google_cloud: TextToSpeech) -> Self {
        self.google_cloud = Some(google_cloud);
        self
    }

    pub fn build(self) -> anyhow::Result<VoiceRegistry> {
        let mut voices = HashMap::new();


        for (id, preset) in &self.config.presets {
            let voice: Arc<dyn Voice> = match preset {
                PresetConfig::GoogleCloudVoice(c) => {
                    let client = self.google_cloud.as_ref()
                        .ok_or_else(|| anyhow!("Google Cloud text-to-speech is required for Google Cloud presets"))?
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
            CacheConfig::InMemory(cache) => Arc::new(CachedVoice::new(voice, cache.capacity)),
        }
    }
}
