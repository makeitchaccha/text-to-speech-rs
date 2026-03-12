use async_trait::async_trait;
use serde::Deserialize;

use crate::tts::{Voice, VoiceDetail, VoiceError};

/// minimum client for Voicevox
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    base_url: reqwest::Url,
}

impl Client {
    pub fn new(http: reqwest::Client, base_url: reqwest::Url) -> Client {
        Client { http, base_url }
    }

    async fn audio_query(&self, text: &str, speaker: i32) -> anyhow::Result<String> {
        let url = self.base_url.join("audio_query")?;
        let res = self
            .http
            .post(url)
            .query(&[("text", text), ("speaker", &speaker.to_string())])
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await?;

        let body = res.text().await?;
        Ok(body)
    }

    async fn synthesis(&self, speaker: i32, body: String) -> anyhow::Result<Vec<u8>> {
        let url = self.base_url.join("synthesis")?;
        let res = self
            .http
            .post(url)
            .query(&[("speaker", &speaker.to_string())])
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::ACCEPT, "audio/wav")
            .body(body)
            .send()
            .await?;

        Ok(res.bytes().await?.to_vec())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct VoicevoxVoiceConfig {
    pub speaker_id: i32,
}

impl VoicevoxVoiceConfig {
    pub fn generate_default_detail(&self, key: &str) -> VoiceDetail {
        VoiceDetail {
            name: key.to_string(),
            provider: "Voicevox".to_string(),
            description: None,
        }
    }
}

pub struct VoicevoxVoice {
    identifier: String,
    client: Client,
    speaker_id: i32,
}

impl VoicevoxVoice {
    pub fn new(client: Client, speaker_id: i32) -> VoicevoxVoice {
        let identifier = Self::build_identifier(speaker_id);
        Self {
            identifier,
            client,
            speaker_id,
        }
    }

    fn build_identifier(speaker_id: i32) -> String {
        format!("voicevox-(id:{})", speaker_id)
    }
}

#[async_trait]
impl Voice for VoicevoxVoice {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    fn language(&self) -> &str {
        "ja-JP"
    }

    async fn generate(&self, text: &str) -> Result<Vec<u8>, VoiceError> {
        let res_audio_query = self
            .client
            .audio_query(text, self.speaker_id)
            .await
            .map_err(VoiceError::Api)?;
        let res_synthesis = self
            .client
            .synthesis(self.speaker_id, res_audio_query)
            .await
            .map_err(VoiceError::Api)?;

        Ok(res_synthesis)
    }
}
