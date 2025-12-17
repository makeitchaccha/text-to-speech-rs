use anyhow::Context;
use google_cloud_texttospeech_v1::client::TextToSpeech;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::GatewayIntents;
use songbird::SerenityInit;
use sqlx::{Pool, Postgres, Sqlite};
use std::sync::Arc;
use text_to_speech_rs::config::{load_config, DatabaseConfig, DatabaseKind};
use text_to_speech_rs::handler::event_handler;
use text_to_speech_rs::localization::{load_discord_locales, load_tts_locales};
use text_to_speech_rs::profile::repository::ProfileRepository;
use text_to_speech_rs::profile::resolver::ProfileResolver;
use text_to_speech_rs::session::manager::SessionManager;
use text_to_speech_rs::tts::registry::VoiceRegistry;
use text_to_speech_rs::{command, handler};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let tts_locales = load_tts_locales("en")?;
    let discord_locales = load_discord_locales("en-US")?;

    info!("Starting text-to-speech bot");

    let config = load_config("config.toml")
        .context("Failed to load config.toml")?;

    config.verify()?;

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

    let repository = prepare_repository(&config.database).await?;

    let resolver = ProfileResolver::new(repository.clone(), config.bot.global_profile.clone());

    let mut commands = command::commands();

    discord_locales.apply(&mut commands)?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
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
                    resolver,
                    repository,
                    tts_locales,
                    discord_locales
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

async fn prepare_repository(config: &DatabaseConfig) -> anyhow::Result<Arc<dyn ProfileRepository>> {
    match config.kind {
        DatabaseKind::SQLite => {
            #[cfg(feature = "sqlite")]
            {
                use text_to_speech_rs::profile::repository::sqlite::SQLiteProfileRepository;
                info!("Opening SQLite database...");
                let pool: Pool<Sqlite> = sqlx::SqlitePool::connect(&config.url).await?;
                Ok(Arc::new(SQLiteProfileRepository::new(pool)))
            }
            #[cfg(not(feature = "sqlite"))]
            anyhow::bail!("SQLite selected, but 'sqlite' feature is not enabled.")
        },
        DatabaseKind::Postgres => {
            #[cfg(feature = "postgres")]
            {
                use text_to_speech_rs::profile::repository::postgres::PostgresRepository;
                info!("Connecting to PostgreSQL...");
                let pool: Pool<Postgres> = sqlx::PgPool::connect(&config.url).await?;
                Ok(Arc::new(PostgresRepository::new(pool)))
            }
            #[cfg(not(feature = "postgres"))]
            anyhow::bail!("PostgreSQL selected, but 'postgres' feature is not enabled.")
        }
    }
}
