use std::sync::Arc;
use anyhow::Context;
use async_trait::async_trait;
use tokio::sync::mpsc;
use crate::tts::Voice;

pub mod actor;
pub mod manager;

pub enum Priority {
    Low,
    High,
}

pub enum SessionCommand {
    Speak {
        text: String,
        voice: Arc<dyn Voice>,
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
    pub fn new(tx: mpsc::Sender<SessionCommand>) -> Self {
        Self { tx }
    }

    pub async fn speak(&self, text: String, voice: Arc<dyn Voice>) -> anyhow::Result<()> {
        self.tx.send(SessionCommand::Speak { text, voice, priority: Priority::Low }).await?;
        Ok(())
    }

    pub async fn stop(&self) -> anyhow::Result<()> {
        self.tx.send(SessionCommand::Stop).await?;
        Ok(())
    }
}