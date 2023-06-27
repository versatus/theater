use std::marker::PhantomData;

use async_trait::async_trait;
use theater_types::{Actor, ActorId, ActorLabel, ActorState, Handler, Result};
use tokio::sync::broadcast::Receiver;

#[derive(Debug, Clone)]
pub struct ActorImpl<H, M>
where
    H: Handler<M> + Send,
    M: std::fmt::Debug + Clone + Send,
{
    handler: H,
    stop_early: bool,
    // NOTE: https://doc.rust-lang.org/nomicon/phantom-data.html#phantomdata
    _m: PhantomData<M>,
}

impl<H, M> ActorImpl<H, M>
where
    H: Handler<M> + Send,
    M: std::fmt::Debug + Clone + Send,
{
    pub fn new(handler: H) -> Self {
        ActorImpl {
            handler,
            stop_early: true,
            _m: PhantomData,
        }
    }

    fn set_early_stop(&mut self, val: bool) {
        self.stop_early = val
    }
}

#[async_trait]
impl<H, M> Actor<M> for ActorImpl<H, M>
where
    H: Handler<M> + Send,
    M: std::fmt::Debug + Clone + Send,
{
    fn id(&self) -> ActorId {
        self.handler.id()
    }

    fn status(&self) -> ActorState {
        self.handler.status()
    }

    async fn start(&mut self, message_rx: &mut Receiver<M>) -> Result<()> {
        self.handler.set_status(ActorState::Starting);

        self.handler.on_start();

        self.handler.set_status(ActorState::Started);

        while let Ok(message) = message_rx.recv().await {
            self.handler.on_tick();

            match self.handler.handle(message).await {
                Err(err) => {
                    self.handler.on_error(err);
                    if self.stop_early {
                        break;
                    }
                },
                Ok(result) => {
                    if result == ActorState::Stopped {
                        self.handler.set_status(ActorState::Terminating);
                        self.handler.on_stop();

                        return Ok(());
                    }
                },
            }
        }

        self.handler.set_status(ActorState::Stopped);

        Ok(())
    }
}
