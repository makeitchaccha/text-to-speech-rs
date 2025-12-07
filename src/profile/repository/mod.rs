use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use poise::serenity_prelude::{GuildId, UserId};

#[async_trait]
pub trait ProfileRepository: Send + Sync {
    async fn find_by_user(&self, user_id: UserId) -> Result<Option<String>>;
    async fn find_by_guild(&self, guild_id: GuildId) -> Result<Option<String>>;

    async fn save_user(&self, user_id: UserId, profile_id: &str) -> Result<()>;
    async fn save_guild(&self, guild_id: GuildId, profile_id: &str) -> Result<()>;

    async fn delete_user(&self, user_id: UserId) -> Result<()>;
    async fn delete_guild(&self, guild_id: GuildId) -> Result<()>;
}