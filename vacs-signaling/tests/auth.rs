use pretty_assertions::assert_eq;
use std::time::Duration;
use test_log::test;
use tokio::sync::watch;
use vacs_protocol::ClientInfo;
use vacs_signaling::client;
use vacs_signaling::transport;

#[test(tokio::test)]
async fn login() {
    let test_app = vacs_server::test_utils::TestApp::new().await;

    let transport = transport::tokio::TokioTransport::new(test_app.addr())
        .await
        .expect("Failed to create transport");
    let (shutdown_tx, shutdown_rx) = watch::channel(());
    let mut client = client::SignalingClient::builder(transport, shutdown_rx)
        .with_login_timeout(Duration::from_millis(100))
        .build();

    let res = client.login("client1", "token1").await;
    assert!(res.is_ok());
    assert_eq!(
        res.unwrap(),
        vec![ClientInfo {
            id: "client1".to_string(),
            display_name: "client1".to_string()
        }]
    );

    shutdown_tx.send(()).unwrap();
}
