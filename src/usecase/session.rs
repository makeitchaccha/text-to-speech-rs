use std::sync::Arc;
use anyhow::Context;
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, CreateEmbed, Guild, GuildId};
use crate::handler::Data;
use crate::localization::Locales;
use crate::profile::resolver::ProfileResolver;
use crate::session::actor::SessionActor;
use crate::session::driver::SongbirdDriver;
use crate::session::manager::SessionManager;
use crate::tts::registry::VoicePackageRegistry;

pub async fn start(ctx: &serenity::Context, data: &Data, guild_id: GuildId, text_channel_id: ChannelId, voice_channel_id: ChannelId) -> anyhow::Result<()> {
    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow::anyhow!("Songbird Voice client not initialized"))?
        .clone();

    let handler = manager.join(guild_id, voice_channel_id).await
        .context("Failed to join voice channel")?;

    // prepare session actor to start text-to-speech
    let driver = SongbirdDriver { call: handler };
    let (actor, handle) = SessionActor::new(Arc::new(driver));

    tokio::spawn(actor.run());

    data.session_manager.register(guild_id, text_channel_id, voice_channel_id, handle.clone());

    let profile = data.resolver.resolve_guild_with_fallback(guild_id).await;

    let profile_str = match &profile {
        Ok(profile) => profile.id.as_str(),
        Err(_) => data.resolver.fallback()
    };

    let voice = data.registry.get_voice(profile_str).unwrap();

    handle.announce(data.tts_locales.resolve(voice.language(), "launch", None, None)?, voice).await?;

    Ok(())
}