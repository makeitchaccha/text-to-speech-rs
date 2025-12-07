use crate::profile::repository::ProfileRepository;
use anyhow::Result;
use std::sync::Arc;
use poise::serenity_prelude::{GuildId, UserId};

pub struct ProfileResolver{
    repository: Arc<dyn ProfileRepository>,
    fallback_profile_id: String,
}

impl ProfileResolver {
    pub fn new(repository: Arc<dyn ProfileRepository>, fallback_profile_id: String) -> Self {
        Self {
            repository,
            fallback_profile_id,
        }
    }

    pub async fn resolve(&self, user_id: UserId, guild_id: GuildId) -> Result<String> {
        if let Some(profile_id) = self.repository.find_by_user(user_id).await? {
            return Ok(profile_id);
        }

        self.resolve_guild(guild_id).await
    }

    pub async fn resolve_guild(&self, guild_id: GuildId) -> Result<String> {
        if let Some(profile_id) = self.repository.find_by_guild(guild_id).await? {
            return Ok(profile_id);
        }

        Ok(self.fallback_profile_id.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::profile::test_utils::MockProfileRepository;
    use super::*;

    struct TestContext {
        resolver: ProfileResolver,
        repository: Arc<dyn ProfileRepository>,
    }

    impl TestContext {
        fn new() -> Self {
            let repository = Arc::new(MockProfileRepository::new());
            let resolver = ProfileResolver::new(
                repository.clone(),
                "fallback-profile".to_string()
            );
            Self { resolver, repository }
        }

        async fn with_user_profile(self, user_id: u64, profile: &str) -> Self {
            self.repository.save_user(UserId::from(user_id), profile).await.unwrap();
            self
        }

        async fn with_guild_profile(self, guild_id: u64, profile: &str) -> Self {
            self.repository.save_guild(GuildId::from(guild_id), profile).await.unwrap();
            self
        }
    }

    #[tokio::test]
    async fn user_profile_takes_precedence_over_guild_profile() {
        let ctx = TestContext::new()
            .with_user_profile(1, "user-P").await
            .with_guild_profile(1, "guild-P").await;

        let result = ctx.resolver.resolve(UserId::from(1), GuildId::from(1)).await.unwrap();

        assert_eq!(result, "user-P");
    }

    #[tokio::test]
    async fn uses_guild_profile_if_user_profile_is_missing() {
        let ctx = TestContext::new()
            .with_guild_profile(1, "guild-P")
            .await;

        let result = ctx.resolver.resolve(UserId::from(1), GuildId::from(1)).await.unwrap();

        assert_eq!(result, "guild-P");
    }

    #[tokio::test]
    async fn returns_fallback_if_nothing_configured() {
        let ctx = TestContext::new();

        let result = ctx.resolver.resolve(UserId::from(1), GuildId::from(1)).await.unwrap();

        assert_eq!(result, "fallback-profile");
    }
}
