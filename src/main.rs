use anyhow::Context;
use google_cloud_texttospeech_v1::client::TextToSpeech;
use std::env;
use text_to_speech_rs::config::load_config;
use text_to_speech_rs::tts::registry::VoiceRegistry;
use tracing::info;

use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, GatewayIntents, GuildId};
use songbird::SerenityInit;
use tracing_subscriber::EnvFilter;
use text_to_speech_rs::handler;
use text_to_speech_rs::handler::event_handler;
use text_to_speech_rs::session::manager::SessionManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Starting text-to-speech bot");

    let config = load_config("config.toml")
        .context("Failed to load config.toml")?;

    // just for test usage and should be rewritten soon.
    let guild_id = GuildId::new(env::var("TTSBOT_TMP_GUILD_ID").expect("Guild ID must be set").parse().expect("Should be an integer"));
    let connect_to = ChannelId::new(env::var("TTSBOT_TMP_VOICE_CHANNEL_ID").expect("Voice Channel ID must be set").parse().expect("Should be an integer"));
    let reading = ChannelId::new(env::var("TTSBOT_TMP_READING_CHANNEL_ID").expect("Reading Channel ID must be set").parse().expect("Should be an integer"));

    info!("Loaded config");

    let mut registry_builder = VoiceRegistry::builder(config.clone());

    if let Some(_google_cloud_config) = &config.backend.google_cloud {
        info!("Using Google Cloud credentials");
        let client = TextToSpeech::builder().build().await?;

        registry_builder = registry_builder.google_cloud(client);
    }

    let registry = registry_builder
        .build()
        .context("Failed to build VoiceRegistry")?;

    info!("VoiceRegistry built successfully.");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(handler::Data {
                    session_manager: SessionManager::new(),
                    registry,
                    tmp_guild_id: guild_id,
                    tmp_voice_channel_id: connect_to,
                    tmp_reading_channel_id: reading,
                })
            })
        })
        .build();

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = serenity::ClientBuilder::new(config.bot.token, intents)
        .register_songbird()
        .framework(framework).await?;

    client.start().await?;

    Ok(())
}
