use async_trait::async_trait;
use poise::serenity_prelude::{GuildId, UserId};
use sqlx::PgPool;
use crate::profile::repository::ProfileRepository;

pub struct PostgresRepository {
    pool: PgPool
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        PostgresRepository { pool }
    }
}

#[async_trait]
impl ProfileRepository for PostgresRepository {
    async fn find_by_user(&self, user_id: UserId) -> anyhow::Result<Option<String>> {
        let id = user_id.to_string();
        let record = sqlx::query!(
                "SELECT profile_id FROM user_profiles WHERE user_id = $1 -- postgres",
                id
            ).fetch_optional(&self.pool).await?;

        Ok(record.map(|record| record.profile_id))
    }

    async fn find_by_guild(&self, guild_id: GuildId) -> anyhow::Result<Option<String>> {
        let id = guild_id.to_string();
        let record = sqlx::query!(
                "SELECT profile_id FROM guild_profiles WHERE guild_id = $1 -- postgres",
                id
            ).fetch_optional(&self.pool).await?;

        Ok(record.map(|record| record.profile_id))
    }

    async fn find_highest_priority(&self, user_id: UserId, guild_id: GuildId) -> anyhow::Result<Option<String>> {
        let user_id = user_id.to_string();
        let guild_id = guild_id.to_string();
        let record = sqlx::query!(
                r#"
                SELECT COALESCE(
                    (SELECT profile_id FROM user_profiles WHERE user_id = $1),
                    (SELECT profile_id FROM guild_profiles WHERE guild_id = $2)
                ) as profile_id -- postgres"#,
                user_id,
                guild_id
            ).fetch_optional(&self.pool).await?;

        Ok(record.and_then(|r| r.profile_id))
    }

    async fn save_user(&self, user_id: UserId, profile_id: &str) -> anyhow::Result<()> {
        let id = user_id.to_string();
        let _ = sqlx::query!(
                "INSERT INTO user_profiles(user_id, profile_id) VALUES($1, $2) ON CONFLICT (user_id) DO UPDATE SET profile_id = EXCLUDED.profile_id -- postgres",
                id,
                profile_id
            ).fetch_optional(&self.pool).await?;

        Ok(())
    }

    async fn save_guild(&self, guild_id: GuildId, profile_id: &str) -> anyhow::Result<()> {
        let id = guild_id.to_string();
        let _ = sqlx::query!(
                "INSERT INTO guild_profiles(guild_id, profile_id) VALUES($1, $2) ON CONFLICT (guild_id) DO UPDATE SET profile_id = EXCLUDED.profile_id -- postgres",
                id,
                profile_id
            ).fetch_optional(&self.pool).await?;

        Ok(())
    }

    async fn delete_user(&self, user_id: UserId) -> anyhow::Result<()> {
        let id = user_id.to_string();
        let _ = sqlx::query!(
                "DELETE FROM user_profiles WHERE user_id = $1 -- postgres",
                id
            ).execute(&self.pool).await?;
        Ok(())
    }

    async fn delete_guild(&self, guild_id: GuildId) -> anyhow::Result<()> {
        let id = guild_id.to_string();
        let _ = sqlx::query!(
                "DELETE FROM guild_profiles WHERE guild_id = $1 -- postgres",
                id
            ).execute(&self.pool).await?;

        Ok(())
    }
}