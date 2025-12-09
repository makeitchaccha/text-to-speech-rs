pub mod session;
pub mod moderation;
pub mod profile;

pub type Error = anyhow::Error;
pub type Result<T> = std::result::Result<T, Error>;
pub type Context<'a> = poise::Context<'a, crate::handler::Data, Error>;