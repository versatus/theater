use async_trait::async_trait;
use std::{error::Error, fmt::Debug};
use thiserror;
use tokio::sync::broadcast::Receiver;
use tracing;

#[deprecated(note = "error forwarding will be handled by the TheaterResult type")]
#[derive(thiserror::Error, Debug)]
pub enum TheaterError {
    #[error("{0}")]
    Other(String),
}

#[deprecated(note = "replaced by TheaterResult")]
pub type Result<T> = std::result::Result<T, TheaterError>;

#[derive(Debug)]
pub enum Recover<T> {
    Value(T),
    State(ActorState),
}

pub struct TheaterResult<T, E: Error + Debug>(std::result::Result<T, E>);
impl<T, E: Error + Debug> TheaterResult<T, E> {
    /// Logs error types with extra context, converting the error to a recoverable state,
    /// otherwise forwarding the contained value.
    ///
    /// Based on the `.inspect_err` method in `std::result::Result`, with the
    /// focus on recovering from handler execution failure.
    pub fn with_context(self, context: &str) -> Recover<T> {
        if let Err(ref err) = self.0 {
            if !context.is_empty() {
                tracing::error!("{context}: {err:?}");
            } else {
                tracing::error!("{err:?}");
            }
            Recover::State(ActorState::Running)
        } else {
            Recover::Value(self.0.unwrap())
        }
    }

    /// Shorthand for `.with_context()` when no extra context is needed.
    ///
    /// Logs error types, converting the error to a recoverable state,
    /// otherwise forwarding the contained value.
    pub fn on_err(self) -> Recover<T> {
        self.with_context("")
    }
}
// Useful in the case that the program needs to exit or use the Result type directly.
impl<T, E: Error + Debug> Into<std::result::Result<T, E>> for TheaterResult<T, E> {
    fn into(self) -> std::result::Result<T, E> {
        self.0
    }
}
impl<T, E: Error + Debug> AsRef<std::result::Result<T, E>> for TheaterResult<T, E> {
    fn as_ref(&self) -> &std::result::Result<T, E> {
        &self.0
    }
}

#[async_trait]
pub trait Handler<M>
where
    M: std::fmt::Debug + Clone + Send,
{
    fn id(&self) -> ActorId;
    fn label(&self) -> ActorLabel;
    fn status(&self) -> ActorState;
    fn set_status(&mut self, state: ActorState);

    #[deprecated(note = "use on_handle method")]
    /// Called every time a message is received by an actor
    async fn handle(&mut self, msg: M) -> Result<ActorState>;

    /// Called every time a message is received by an [`Actor`].
    async fn on_handle<E: Error + Debug>(&mut self, msg: M) -> TheaterResult<ActorState, E>;

    /// Called before starting the message processing loop
    fn on_start(&self) {}

    /// Called before starting the message processing loop
    fn on_tick(&self) {}

    /// Called before stopping the message processing loop
    fn on_stop(&self) {}

    #[deprecated(note = "error forwarding will be handled by the TheaterResult type")]
    fn on_error(&self, err: &TheaterError) {}
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum ActorState {
    Starting,
    Started,
    Running,
    #[default]
    Stopped,
    Terminating,
}

pub type MessageBytes = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub from: ActorId,
    pub data: MessageBytes,
}

pub type ActorId = String;
pub type ActorLabel = String;

#[async_trait]
pub trait Actor<M>
where
    M: std::fmt::Debug + Clone + Send,
{
    /// Uniquely identifies an actor within the system
    fn id(&self) -> ActorId;

    /// Optional human-readable label
    fn label(&self) -> ActorLabel;
    fn status(&self) -> ActorState;

    #[deprecated(note = "replaced by on_start")]
    async fn start(&mut self, message_rx: &mut Receiver<M>) -> Result<()>;

    async fn on_start<E: Error + Debug>(&mut self, message_rx: &mut Receiver<M>);
}
