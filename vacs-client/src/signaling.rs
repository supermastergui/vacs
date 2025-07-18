use tokio::sync::watch;
use vacs_signaling::client::SignalingClient;
use vacs_signaling::transport;

pub struct Session {
    client: SignalingClient<transport::tokio::TokioTransport>,
    shutdown_tx: watch::Sender<()>,
}

impl Session {
    pub async fn new() -> anyhow::Result<Self> {
        let (shutdown_tx, shutdown_rx) = watch::channel(());
        let transport = transport::tokio::TokioTransport::new("wss://localhost:8080/ws").await?;
        let client = SignalingClient::new(transport, shutdown_rx);

        Ok(Self {
            client,
            shutdown_tx,
        })
    }

    pub async fn login(self, token: String) -> anyhow::Result<()> {
        todo!()
    }
}
