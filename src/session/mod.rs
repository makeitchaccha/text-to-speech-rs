use std::sync::Arc;
use anyhow::Context;
use poise::serenity_prelude::UserId;
use tokio::sync::mpsc;
use crate::tts::Voice;

pub mod actor;
pub mod manager;
mod sanitizer;

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

enum SessionCommand {
    Speak {
        text: String,
        voice: Arc<dyn Voice>,
        speaker: Option<Speaker>,
        priority: Priority,
    },
    Stop,
    NotifyPlaybackEnd, // for actor internal usage
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
}