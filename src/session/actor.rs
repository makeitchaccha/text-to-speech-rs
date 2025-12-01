use std::sync::{Arc};
use async_trait::async_trait;
use songbird::{Call, Event, EventContext, EventHandler};
use tokio::sync::{mpsc, Mutex};
use tracing;
use crate::session::{Priority, SessionCommand, SessionHandle};
use crate::tts::Voice;

struct QueueItem {
    text: String,
    voice: Arc<dyn Voice>,
    priority: Priority
}

enum WorkerEvent {
    Ready
}

enum WorkerCommand{
    Play(QueueItem)
}

struct PlaybackEndNotifier { tx: mpsc::Sender<WorkerEvent> }
#[async_trait]
impl EventHandler for PlaybackEndNotifier {
    async fn act(&self, _: &EventContext<'_>) -> Option<Event> {
        let _ = self.tx.send(WorkerEvent::Ready).await;
        None
    }
}

pub struct SessionActor {
    rx: mpsc::Receiver<SessionCommand>,
    call: Arc<Mutex<Call>>
}

impl SessionActor {
    pub fn new(call: Arc<Mutex<Call>>) -> (Self, SessionHandle) {
        let (cmd_tx, cmd_rx) = mpsc::channel(100);

        let actor = Self {
            rx: cmd_rx,
            call
        };

        (actor, SessionHandle::new(cmd_tx))
    }

    pub async fn run(mut self) {
        tracing::info!("Session actor started");

        while let Some(cmd) = self.rx.recv().await {
            match cmd {
                SessionCommand::Speak { text, voice, priority: _ } => {
                    self.handle_speak(text, voice).await;
                }
                SessionCommand::Stop => {
                    let mut handler = self.call.lock().await;
                    handler.stop();
                }
                SessionCommand::NotifyPlaybackEnd => {
                    // ignore
                }
            }
        }
    }

    async fn handle_speak(&mut self, text: String, voice: Arc<dyn Voice>) {
        let mut handler = self.call.lock().await;
        if handler.queue().len() >= 5 {
            tracing::warn!("Queue full, dropping: {}", text);
            return;
        }

        let audio_data = match voice.generate(&text).await {
            Ok(audio_data) => audio_data,
            Err(e) => {
                tracing::error!("Failed to generate audio: {}", e);
                return;
            }
        };

        handler.enqueue_input(audio_data.into()).await;
    }
}
