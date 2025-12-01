use anyhow::Context;
use google_cloud_texttospeech_v1::client::TextToSpeech;
use tracing::info;
use text_to_speech_rs::config::load_config;
use text_to_speech_rs::tts::registry::VoiceRegistry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting text-to-speech bot");

    let config = load_config("config.toml")
        .context("Failed to load config.toml")?;

    info!("Loaded config");

    let mut registry_builder = VoiceRegistry::builder(config.clone());

    if let Some(_google_cloud_config) = &config.backend.google_cloud {
        info!("Using Google Cloud credentials");
        let client = TextToSpeech::builder().build().await?;

        registry_builder = registry_builder.google_cloud(client);
    }

    let _registry = registry_builder
        .build()
        .context("Failed to build VoiceRegistry")?;

    info!("VoiceRegistry built successfully.");



    Ok(())
}
