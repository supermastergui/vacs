pub mod mock;
pub mod tokio;

use crate::signaling::Message;
use crate::signaling::error::SignalingError;
use async_trait::async_trait;

#[async_trait]
pub trait SignalingTransport: Send + Sync {
    async fn send(&mut self, msg: Message) -> Result<(), SignalingError>;
    async fn recv(&mut self) -> Result<Message, SignalingError>;
}
