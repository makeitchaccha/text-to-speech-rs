use anyhow::Context;
use google_cloud_texttospeech_v1::client::TextToSpeech;
use std::env;
use text_to_speech_rs::config::load_config;
use text_to_speech_rs::tts::registry::VoiceRegistry;
use tracing::info;

use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, GatewayIntents, GuildId};
use songbird::SerenityInit;
use text_to_speech_rs::handler::event_handler;
use text_to_speech_rs::session::manager::SessionManager;
use text_to_speech_rs::{command, handler};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

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

    let registry = registry_builder
        .build()
        .context("Failed to build VoiceRegistry")?;

    info!("VoiceRegistry built successfully.");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                command::moderation::register(),
                command::session::join(),
                command::session::leave(),
            ],
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
