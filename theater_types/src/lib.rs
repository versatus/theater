use async_trait::async_trait;
use tokio::sync::broadcast::Receiver;

#[derive(Debug, Clone, thiserror::Error)]
pub enum TheaterError {
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, TheaterError>;

#[async_trait]
pub trait Handler<M>
where
    M: std::fmt::Debug + Clone + Send,
{
    fn id(&self) -> ActorId;
    fn label(&self) -> ActorLabel;
    fn status(&self) -> ActorState;
    fn set_status(&mut self, state: ActorState);

    /// Called every time a message is received by an actor
    async fn handle(&mut self, msg: M) -> Result<ActorState>;

    /// Called before starting the message processing loop
    fn on_start(&self) {}

    /// Called before starting the message processing loop
    fn on_tick(&self) {}

    /// Called before stopping the message processing loop
    fn on_stop(&self) {}

    /// Called if errors are emitted from the handler function
    fn on_error(&self, _err: TheaterError) {}
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
    async fn start(&mut self, message_rx: &mut Receiver<M>) -> Result<tokio_util::sync::CancellationToken>;
}

