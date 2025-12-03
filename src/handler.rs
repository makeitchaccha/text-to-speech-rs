use crate::session::actor::SessionActor;
use crate::session::driver::SongbirdDriver;
use crate::session::manager::SessionManager;
use crate::session::Speaker;
use crate::tts::registry::VoiceRegistry;
use anyhow::Context;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, GuildId};
use std::sync::Arc;

pub struct Data{
    pub session_manager: SessionManager,
    pub registry: VoiceRegistry,

    // temporary!
    pub tmp_reading_channel_id: ChannelId,
    pub tmp_voice_channel_id: ChannelId,
    pub tmp_guild_id: GuildId,
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

            let manager = songbird::get(ctx)
                .await
                .expect("Songbird Voice client placed in at initialisation.")
                .clone();

            let handler = manager.join(data.tmp_guild_id, data.tmp_voice_channel_id).await.context("failed to connect to the voice channel")?;

            let (actor, handle) = SessionActor::new(Arc::new(SongbirdDriver{ call: handler }));

            tokio::spawn(actor.run());

            handle.announce(String::from("Canaryがボイスチャンネルに参加しました"), data.registry.get("wavenet-a").unwrap()).await?;

            data.session_manager.register(data.tmp_guild_id, data.tmp_reading_channel_id, handle);
        }

        serenity::FullEvent::VoiceStateUpdate { old: _, new } => {
            if new.user_id != ctx.cache.current_user().id || new.channel_id.is_some() {
                return Ok(());
            }

            data.session_manager.remove(new.guild_id.ok_or(anyhow::anyhow!("Guild not found"))?);
        }

        serenity::FullEvent::Message { new_message } => {
            if let Some(handle) = data.session_manager.get_by_text_channel(new_message.channel_id) {
                let voice = data.registry
                    .get("wavenet-a")
                    .ok_or_else(|| anyhow::anyhow!("No voice preset found"))?;

                let text = new_message.content.clone();

                if let Err(err) = handle.speak(text, voice, Speaker::new(new_message.author.id, new_message.author.display_name().to_string())).await.context("failed to send message") {
                    tracing::error!("Error sending message: {:?}", err);
                    // lazy delete
                    data.session_manager.remove(new_message.guild_id.ok_or(anyhow::anyhow!("Message does not contain guild ID"))?);
                }
            }
        }
        _ => {}
    }
    Ok(())
}