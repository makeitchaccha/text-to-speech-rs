use async_trait::async_trait;
use poise::serenity_prelude::{GuildId, UserId};
use sqlx::SqlitePool;
use crate::profile::repository::ProfileRepository;

pub struct SQLiteProfileRepository {
    pool: SqlitePool
}

impl SQLiteProfileRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProfileRepository for SQLiteProfileRepository {
    async fn find_by_user(&self, user_id: UserId) -> anyhow::Result<Option<String>> {
        let id = user_id.to_string();
        let record = sqlx::query!(
                "SELECT profile_id FROM user_profiles WHERE user_id = ? -- sqlite",
                id
            )
            .fetch_optional(&self.pool).await?;

        Ok(record.map(|record| record.profile_id))
    }

    async fn find_by_guild(&self, guild_id: GuildId) -> anyhow::Result<Option<String>> {
        let id = guild_id.to_string();
        let record = sqlx::query!(
                "SELECT profile_id FROM guild_profiles WHERE guild_id = ? -- sqlite",
                id
            )
            .fetch_optional(&self.pool).await?;

        Ok(record.map(|record| record.profile_id))
    }

    async fn find_highest_priority(&self, user_id: UserId, guild_id: GuildId) -> anyhow::Result<Option<String>> {
        let user_id = user_id.to_string();
        let guild_id = guild_id.to_string();
        let record = sqlx::query!(
                r#"SELECT CAST(COALESCE(
                    (SELECT profile_id FROM user_profiles WHERE user_id = ?),
                    (SELECT profile_id FROM guild_profiles WHERE guild_id = ?)
                ) AS TEXT) as profile_id -- sqlite"#,
                user_id,
                guild_id
            ).fetch_optional(&self.pool).await?;

        Ok(record.and_then(|r| r.profile_id))
    }

    async fn save_user(&self, user_id: UserId, profile_id: &str) -> anyhow::Result<()> {
        let id = user_id.to_string();
        let _ = sqlx::query!(
                "INSERT INTO user_profiles(user_id, profile_id) VALUES(?, ?) ON CONFLICT (user_id) DO UPDATE SET profile_id = EXCLUDED.profile_id -- sqlite",
                id,
                profile_id
            ).execute(&self.pool).await?;

        Ok(())
    }

    async fn save_guild(&self, guild_id: GuildId, profile_id: &str) -> anyhow::Result<()> {
        let id = guild_id.to_string();
        let _ = sqlx::query!(
                "INSERT INTO guild_profiles(guild_id, profile_id) VALUES(?, ?) ON CONFLICT (guild_id) DO UPDATE SET profile_id = EXCLUDED.profile_id -- sqlite",
                id,
                profile_id
            ).execute(&self.pool).await?;

        Ok(())
    }

    async fn delete_user(&self, user_id: UserId) -> anyhow::Result<()> {
        let id = user_id.to_string();
        let _ = sqlx::query!(
                "DELETE FROM user_profiles WHERE user_id = ? -- sqlite",
                id
            ).execute(&self.pool).await?;
        Ok(())
    }

    async fn delete_guild(&self, guild_id: GuildId) -> anyhow::Result<()> {
        let id = guild_id.to_string();
        let _ = sqlx::query!(
                "DELETE FROM guild_profiles WHERE guild_id = ? -- sqlite",
                id
            ).execute(&self.pool).await?;

        Ok(())
    }
}