use std::sync::Arc;
use anyhow::Context as _;
use poise::serenity_prelude::{ChannelId, Mentionable};
use crate::command::{Context, Result};
use crate::session::actor::SessionActor;
use crate::session::driver::SongbirdDriver;

fn user_voice_channel_id(ctx: &Context<'_>) -> Result<ChannelId> {
    let channel_id = ctx.guild()
        .ok_or_else(|| anyhow::anyhow!("Guild not found"))?
        .voice_states
        .get(&ctx.author().id)
        .and_then(|vs| vs.channel_id)
        .ok_or_else(|| anyhow::anyhow!("You have to join a voice channel to start text-to-speech"))?;
    Ok(channel_id)
}

#[poise::command(slash_command, guild_only)]
pub async fn join(ctx: Context<'_>) -> Result<()> {
    let guild_id = ctx.guild_id().ok_or_else(|| anyhow::anyhow!("Guild only"))?;

    let channel_id = user_voice_channel_id(&ctx)?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or_else(|| anyhow::anyhow!("Songbird Voice client not initialized"))?
        .clone();

    let handler = manager.join(guild_id, channel_id).await
        .context("Failed to join voice channel")?;

    // prepare session actor to start text-to-speech
    let driver = SongbirdDriver { call: handler };
    let (actor, handle) = SessionActor::new(Arc::new(driver));

    tokio::spawn(actor.run());

    ctx.data().session_manager.register(guild_id, ctx.channel_id(), handle.clone());

    let voice = ctx.data().registry.get("wavenet-a").unwrap();
    handle.announce("読み上げを開始します".to_string(), voice).await?;

    ctx.say(format!("Now, I'm reading {} in {}", ctx.channel_id().mention(), channel_id.mention())).await?;

    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn leave(ctx: Context<'_>) -> Result<()> {
    let guild_id = ctx.guild_id().ok_or_else(|| anyhow::anyhow!("Guild only"))?;

    let handle = ctx.data().session_manager.get(guild_id).ok_or(anyhow::anyhow!("Guild not found"))?;

    handle.leave().await?;

    ctx.say("Thank you for using text-to-speech-rs beta").await?;

    Ok(())
}