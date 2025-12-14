use std::sync::Arc;
use crate::session::manager::SessionManager;
use crate::session::{SessionHandle, Speaker};
use crate::tts::registry::VoiceRegistry;
use anyhow::{anyhow, Context};
use fluent::{fluent_args, FluentArgs};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, VoiceState};
use crate::localization::Locales;
use crate::profile::repository::ProfileRepository;
use crate::profile::resolver::ProfileResolver;
use crate::sanitizer;

pub struct Data{
    pub session_manager: SessionManager,
    pub registry: VoiceRegistry,
    pub resolver: ProfileResolver,
    pub repository: Arc<dyn ProfileRepository>,
    pub tts_locales: Locales
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

        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            if new.user_id == ctx.cache.current_user().id {
                match voice_state_update_kind(old, new) {
                    ChannelTransition::Move { old_channel_id, new_channel_id } => {
                        data.session_manager.update_voice_channel(
                            old_channel_id,
                            new_channel_id,
                        )?;
                    },
                    ChannelTransition::Disconnect { old_channel_id: _ } => {
                        if let Some(guild_id) = old.as_ref().and_then(|state| state.guild_id) {
                            shutdown_session(&ctx, guild_id, &data.session_manager).await?;
                        }
                    }
                    _ => {}
                }

                return Ok(());
            }

            let guild_id = match new.guild_id {
                Some(id) => id,
                None => return Ok(()),
            };
            let user_id = new.user_id;

            for notification in generate_notifications(old, new) {
                if let Some(session) = data.session_manager.get_by_voice_channel(notification.channel_id) {
                    send_session_notification(
                        ctx, data, &session.handle,
                        guild_id, user_id,
                        notification.locale_id,
                    ).await?;
                }
            }
        }

        serenity::FullEvent::Message { new_message } => {
            if new_message.author.bot {
                // ignores bot message
                return Ok(());
            }

            if let Some(session) = data.session_manager.get_by_text_channel(new_message.channel_id) {
                let profile = data.resolver.resolve_with_fallback(new_message.author.id, new_message.guild_id.ok_or(anyhow::anyhow!("Guild not found"))?).await;

                let profile_str = match &profile {
                    Ok(profile) => profile.id.as_str(),
                    Err(_) => data.resolver.fallback(),
                };

                let voice = data.registry
                    .get(profile_str)
                    .ok_or_else(|| anyhow::anyhow!("No voice preset found"))?;

                let guild_id = new_message.guild_id.ok_or(anyhow::anyhow!("Message does not contain guild ID"))?;

                let text = new_message.content.clone();
                let text = sanitizer::replace_mentions(&text, ctx, guild_id, &new_message.mentions, &new_message.mention_roles, &new_message.mention_channels)?;
                let text = sanitizer::sanitize(&text, 300);

                if let Err(err) = session.handle.speak(text, voice, Speaker::new(new_message.author.id, new_message.author.display_name().to_string())).await.context("failed to send message") {
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

async fn shutdown_session(ctx: &serenity::Context, guild_id: serenity::GuildId, session_manager: &SessionManager) -> Result<(), anyhow::Error> {
    // free resources related session
    session_manager.remove(guild_id);
    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow::anyhow!("Songbird Voice client not initialized"))?
        .clone();
    manager.remove(guild_id).await?;

    Ok(())
}

enum ChannelTransition {
    Connect{ new_channel_id: ChannelId },
    Disconnect{ old_channel_id: ChannelId },
    Move{ new_channel_id: ChannelId, old_channel_id: ChannelId }, // old channel -> new channel
    Ignore
}

fn voice_state_update_kind(old: &Option<VoiceState>, new: &VoiceState) -> ChannelTransition {
    match (old.as_ref().and_then(|v| v.channel_id), new.channel_id) {
        (Some(old_channel_id), None) => ChannelTransition::Disconnect{ old_channel_id },
        (None, Some(new_channel_id)) => ChannelTransition::Connect{ new_channel_id },
        (Some(old_channel_id), Some(new_channel_id)) => {
            if old_channel_id != new_channel_id {
                ChannelTransition::Move{ old_channel_id, new_channel_id }
            } else {
                ChannelTransition::Ignore
            }
        }
        _ => ChannelTransition::Ignore
    }
}

struct Notification {
    channel_id: serenity::ChannelId,
    locale_id: &'static str,
}

fn generate_notifications(old: &Option<VoiceState>, new: &VoiceState) -> Vec<Notification> {
    let mut notifications = Vec::new();

    match voice_state_update_kind(old, new) {
        ChannelTransition::Connect { new_channel_id } => {
            notifications.push(Notification { channel_id: new_channel_id, locale_id: "user-join" });
        },
        ChannelTransition::Disconnect { old_channel_id } => {
            notifications.push(Notification { channel_id: old_channel_id, locale_id: "user-leave" });
        },
        ChannelTransition::Move { old_channel_id, new_channel_id } => {
            notifications.push(Notification { channel_id: old_channel_id, locale_id: "user-leave" });
            notifications.push(Notification { channel_id: new_channel_id, locale_id: "user-join" });
        },
        ChannelTransition::Ignore => {}
    }
    notifications
}

async fn send_session_notification(
    ctx: &serenity::Context,
    data: &Data,
    handle: &SessionHandle,
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    locale_id: &str
) -> Result<(), anyhow::Error> {
    let profile = data.resolver.resolve_with_fallback(user_id, guild_id).await;

    let profile_str = match &profile {
        Ok(profile) => profile.id.as_str(),
        Err(_) => data.resolver.fallback()
    };

    let voice = data.registry.get(profile_str).ok_or(anyhow::anyhow!("No voice preset found"))?;
    let name = guild_id.to_guild_cached(&ctx.cache)
        .and_then(|guild| guild.members.get(&user_id).map(|member| member.display_name().to_owned()))
        .unwrap_or_else(|| data.tts_locales.resolve(voice.language(), "someone", None).unwrap_or("someone".to_owned()));

    handle.announce(data.tts_locales.resolve(voice.language(), locale_id, Some(&fluent_args!["user" => name]))?, voice).await?;

    Ok(())
}