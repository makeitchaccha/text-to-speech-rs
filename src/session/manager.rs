use dashmap::DashMap;
use poise::serenity_prelude::{ChannelId, GuildId};
use crate::session::SessionHandle;

#[derive(Debug)]
pub struct SessionManager {
    sessions: DashMap<GuildId, SessionHandle>,

    text_channels: DashMap<ChannelId, GuildId>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
            text_channels: DashMap::new(),
        }
    }

    pub fn register(&self, guild_id: GuildId, text_channel_id: ChannelId, handle: SessionHandle) {
        self.sessions.insert(guild_id, handle);
        self.text_channels.insert(text_channel_id, guild_id);
        tracing::info!("Registered session for guild: {}, text_channel: {}", guild_id, text_channel_id);
    }

    pub fn get(&self, guild_id: GuildId) -> Option<SessionHandle> {
        self.sessions.get(&guild_id).map(|r| r.value().clone())
    }

    pub fn get_by_text_channel(&self, text_channel_id: ChannelId) -> Option<SessionHandle> {
        let guild_id = self.text_channels.get(&text_channel_id)?;
        self.get(*guild_id)
    }

    pub fn remove(&self, guild_id: GuildId) {
        if self.sessions.remove(&guild_id).is_some() {
            self.text_channels.retain(|_, gid| *gid != guild_id);
            tracing::info!("Removed session for guild: {}", guild_id);
        }
    }
}