pub(crate) mod commands;

use tokio::sync::watch;
use vacs_protocol::ws::ClientInfo;
use vacs_signaling::client::SignalingClient;
use vacs_signaling::error::SignalingError;
use vacs_signaling::transport;
use crate::error::Error;

pub struct Connection {
    client: SignalingClient<transport::tokio::TokioTransport>,
    shutdown_tx: watch::Sender<()>,
}

impl Connection {
    pub async fn new(ws_url: &str) -> Result<Self, SignalingError> {
        let (shutdown_tx, shutdown_rx) = watch::channel(());
        let transport = transport::tokio::TokioTransport::new(ws_url).await?;
        let client = SignalingClient::new(transport, shutdown_rx);

        Ok(Self {
            client,
            shutdown_tx,
        })
    }

    pub async fn login(&mut self, token: &str)  -> Result<Vec<ClientInfo>, SignalingError> {
        self.client.login(token).await
    }

    pub async fn disconnect(&mut self) -> Result<(), Error> {
        self.shutdown_tx.send(()).map_err(|err| Error::Other(anyhow::anyhow!(err)))?;
        self.client.disconnect().await?;
        Ok(())
    }
}
