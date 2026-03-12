use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tts::{Voice, VoiceDetail, VoiceError};

#[derive(Serialize, Deserialize)]
struct LazyAudioQuery {
    accent_phrases: Box<serde_json::value::RawValue>,
    #[serde(rename = "speedScale")]
    speed_scale: f64,
    #[serde(rename = "pitchScale")]
    pitch_scale: f64,
    #[serde(rename = "intonationScale")]
    intonation_scale: f64,
    #[serde(rename = "volumeScale")]
    volume_scale: f64,
    #[serde(rename = "prePhonemeLength")]
    pre_phoneme_length: f64,
    #[serde(rename = "postPhonemeLength")]
    post_phoneme_length: f64,
    #[serde(rename = "pauseLength")]
    pause_length: Option<f64>,
    #[serde(rename = "pauseLengthScale")]
    pause_length_scale: Option<f64>,
    #[serde(rename = "outputSamplingRate")]
    output_sampling_rate: u32,
    #[serde(rename = "outputStereo")]
    output_stereo: bool,
    kana: Option<String>,
}

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

    async fn audio_query(&self, text: &str, speaker: i32) -> anyhow::Result<LazyAudioQuery> {
        let url = self.base_url.join("audio_query")?;
        let res = self
            .http
            .post(url)
            .query(&[("text", text), ("speaker", &speaker.to_string())])
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await?;

        let body: LazyAudioQuery = res.json().await?;
        Ok(body)
    }

    async fn synthesis(
        &self,
        speaker: i32,
        audio_query: LazyAudioQuery,
    ) -> anyhow::Result<Vec<u8>> {
        let url = self.base_url.join("synthesis")?;
        let res = self
            .http
            .post(url)
            .query(&[("speaker", &speaker.to_string())])
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::ACCEPT, "audio/wav")
            .json(&audio_query)
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
        let audio_query = self
            .client
            .audio_query(text, self.speaker_id)
            .await
            .map_err(VoiceError::Api)?;

        let res_synthesis = self
            .client
            .synthesis(self.speaker_id, audio_query)
            .await
            .map_err(VoiceError::Api)?;

        Ok(res_synthesis)
    }
}
