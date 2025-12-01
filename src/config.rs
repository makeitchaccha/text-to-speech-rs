use std::collections::HashMap;
use serde::Deserialize;
use crate::tts::google_cloud::GoogleCloudVoiceConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub bot: BotConfig,

    pub presets: HashMap<String, PresetConfig>
}

#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    pub token: String
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "engine")]
pub enum PresetConfig {
    #[serde(rename="google_cloud")]
    GoogleCloudVoice(GoogleCloudVoiceConfig),
}