mod session;
mod moderation;
mod profile;

pub fn commands() -> Vec<poise::Command<crate::handler::Data, Error>> {
    vec![
        session::join(),
        session::leave(),
        moderation::register(),
        profile::voice(),
    ]
}

pub type Error = anyhow::Error;
pub type Result<T> = std::result::Result<T, Error>;
pub type Context<'a> = poise::Context<'a, crate::handler::Data, Error>;