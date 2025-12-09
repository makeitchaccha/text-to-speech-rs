use std::sync::Arc;
use crate::session::manager::SessionManager;
use crate::session::Speaker;
use crate::tts::registry::VoiceRegistry;
use anyhow::Context;
use poise::serenity_prelude as serenity;
use crate::profile::repository::ProfileRepository;
use crate::profile::resolver::ProfileResolver;
use crate::sanitizer;

pub struct Data{
    pub session_manager: SessionManager,
    pub registry: VoiceRegistry,
    pub resolver: ProfileResolver,
    pub repository: Arc<dyn ProfileRepository>,
}

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, anyhow::Error>,
    data: &Data,
) -> Result<(), anyhow::Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            tracing::info!("Ready: {}", data_about_bot.user.name);
        }

        serenity::FullEvent::VoiceStateUpdate { old: _, new } => {
            if new.user_id != ctx.cache.current_user().id || new.channel_id.is_some() {
                return Ok(());
            }

            // free resources related session
            let guild_id = new.guild_id.ok_or(anyhow::anyhow!("Guild not found"))?;
            data.session_manager.remove(guild_id);
            let manager = songbird::get(ctx)
                .await
                .ok_or_else(|| anyhow::anyhow!("Songbird Voice client not initialized"))?
                .clone();
            manager.remove(guild_id).await?;
        }

        serenity::FullEvent::Message { new_message } => {
            if new_message.author.bot {
                // ignores bot message
                return Ok(());
            }

            if let Some(handle) = data.session_manager.get_by_text_channel(new_message.channel_id) {
                let profile = data.resolver.resolve_with_fallback(new_message.author.id, new_message.guild_id.ok_or(anyhow::anyhow!("Guild not found"))?).await;

                let profile_str = match &profile {
                    Ok(profile) => profile.as_str(),
                    Err(_) => data.resolver.fallback(),
                };

                let voice = data.registry
                    .get(profile_str)
                    .ok_or_else(|| anyhow::anyhow!("No voice preset found"))?;

                let guild_id = new_message.guild_id.ok_or(anyhow::anyhow!("Message does not contain guild ID"))?;

                let text = new_message.content.clone();
                let text = sanitizer::replace_mentions(&text, ctx, guild_id, &new_message.mentions, &new_message.mention_roles, &new_message.mention_channels)?;
                let text = sanitizer::sanitize(&text, 300);

                if let Err(err) = handle.speak(text, voice, Speaker::new(new_message.author.id, new_message.author.display_name().to_string())).await.context("failed to send message") {
                    tracing::error!("Error sending message: {:?}", err);
                    // lazy delete
                    data.session_manager.remove(guild_id);
                }
            }
        }
        _ => {}
    }
    Ok(())
}