use std::collections::HashMap;
use anyhow::anyhow;
use config::Config;
use serde::Deserialize;
use crate::tts::google_cloud::GoogleCloudVoiceConfig;

pub fn load_config(path: &str) -> anyhow::Result<AppConfig> {
    let config = Config::builder()
        .add_source(config::File::with_name(path))
        .add_source(config::Environment::with_prefix("TTSBOT").separator("__"))
        .build()?;

    config.try_deserialize()
        .map_err(|e| anyhow!(e))
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub bot: BotConfig,

    #[serde(default)]
    pub backend: BackendConfig,

    pub cache: CacheConfig,

    pub presets: HashMap<String, PresetConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    pub token: String
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BackendConfig {
    pub google_cloud: Option<GoogleCloudBackendConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleCloudBackendConfig {
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_timeout() -> u64 { 5 }

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum CacheConfig {
    #[serde(rename = "disabled")]
    Disabled,
    #[serde(rename = "in_memory")]
    InMemory(InMemoryCacheConfig),
}

#[derive(Debug, Clone, Deserialize)]
pub struct InMemoryCacheConfig {
    pub capacity: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "engine")]
pub enum PresetConfig {
    #[serde(rename="google_cloud")]
    GoogleCloudVoice(GoogleCloudVoiceConfig),
}