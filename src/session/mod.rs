use std::sync::Arc;
use anyhow::Context;
use poise::serenity_prelude::UserId;
use tokio::sync::mpsc;
use crate::tts::Voice;

pub mod actor;
pub mod manager;
mod sanitizer;
pub mod driver;

#[derive(Clone, Copy)]
enum Priority {
    User,
    System,
}

#[derive(Debug, Clone)]
pub struct Speaker{
    user_id: UserId,
    name: String,
}

impl Speaker{
    pub fn new(user_id: UserId, name: String) -> Self{
        Self{user_id, name}
    }
}

#[derive(Clone)]
enum SessionCommand {
    Speak {
        text: String,
        voice: Arc<dyn Voice>,
        speaker: Option<Speaker>,
        priority: Priority,
    },
    Stop,
    Leave, // user intentionally disconnected by command
    Disconnect, // internal usage: Songbird drive
}

#[derive(Debug, Clone)]
pub struct SessionHandle {
    tx: mpsc::Sender<SessionCommand>,
}

impl SessionHandle {
    fn new(tx: mpsc::Sender<SessionCommand>) -> Self {
        Self { tx }
    }

    pub async fn speak(&self, text: String, voice: Arc<dyn Voice>, speaker: Speaker) -> anyhow::Result<()> {
        self.tx.send(SessionCommand::Speak { text, voice, speaker: Some(speaker), priority: Priority::User }).await?;
        Ok(())
    }

    pub async fn announce(&self, text: String, voice: Arc<dyn Voice>) -> anyhow::Result<()> {
        self.tx.send(SessionCommand::Speak { text, voice, speaker: None, priority: Priority::System}).await?;
        Ok(())
    }

    pub async fn stop(&self) -> anyhow::Result<()> {
        self.tx.send(SessionCommand::Stop).await?;
        Ok(())
    }

    pub async fn leave(&self) -> anyhow::Result<()> {
        self.tx.send(SessionCommand::Leave).await?;
        Ok(())
    }
}