use crate::signaling::Message;
use crate::signaling::transport::{SignalingError, SignalingTransport};
use async_trait::async_trait;
use tokio::sync::mpsc;

pub struct MockHandle {
    pub outgoing_rx: mpsc::Receiver<Message>,
    pub incoming_tx: mpsc::Sender<Message>,
}

pub struct MockTransport {
    outgoing: mpsc::Sender<Message>,
    incoming: mpsc::Receiver<Message>,
}

impl MockTransport {
    #[tracing::instrument(level = "info")]
    pub fn new() -> (Self, MockHandle) {
        let (outgoing_tx, outgoing_rx) = mpsc::channel(32);
        let (incoming_tx, incoming_rx) = mpsc::channel(32);

        let transport = Self {
            outgoing: outgoing_tx,
            incoming: incoming_rx,
        };

        let handle = MockHandle {
            outgoing_rx,
            incoming_tx,
        };

        (transport, handle)
    }
}

#[async_trait]
impl SignalingTransport for MockTransport {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn send(&mut self, msg: Message) -> Result<(), SignalingError> {
        tracing::debug!("Sending message");
        self.outgoing.send(msg).await.map_err(|err| {
            tracing::warn!(?err, "Failed to send message");
            SignalingError::Transport(anyhow::anyhow!(err))
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn recv(&mut self) -> Result<Message, SignalingError> {
        match self.incoming.recv().await {
            Some(msg) => {
                tracing::debug!(?msg, "Received message");
                Ok(msg)
            }
            None => {
                tracing::warn!("Channel closed");
                Err(SignalingError::Disconnected)
            }
        }
    }
}
