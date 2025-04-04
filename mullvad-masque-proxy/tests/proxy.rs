use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use bytes::BytesMut;
use tokio::fs;

use mullvad_masque_proxy::client;
use mullvad_masque_proxy::server;
use tokio::net::UdpSocket;

/// Set up a MASQUE proxy and test that it can be used to communicate with some UDP destination
#[tokio::test]
async fn test_server_and_client_forwarding() -> anyhow::Result<()> {
    const MAXIMUM_PACKET_SIZE: u16 = 1700;
    const HOST: &str = "test.test";

    let any_localhost_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    // Set up destination UDP server
    let target_udp_server = UdpSocket::bind(any_localhost_addr).await?;
    let target_udp_addr = target_udp_server
        .local_addr()
        .context("Retrieve dest UDP server addr")?;

    // Set up MASQUE server
    let server_tls_config = load_server_test_cert().await?;
    let server = server::Server::bind(
        any_localhost_addr,
        Default::default(),
        Arc::new(server_tls_config),
        MAXIMUM_PACKET_SIZE,
    )
    .context("Failed to start MASQUE server")?;

    let masque_server_addr = server.local_addr()?;

    tokio::spawn(server.run());

    // Set up MASQUE client
    let local_socket = UdpSocket::bind(any_localhost_addr)
        .await
        .context("Failed to bind address")?;
    let local_udp_addr = local_socket.local_addr().unwrap();

    let client = client::Client::connect_with_tls_config(
        local_socket,
        masque_server_addr,
        // Local QUIC address
        any_localhost_addr,
        target_udp_addr,
        HOST,
        client::default_tls_config(),
        MAXIMUM_PACKET_SIZE,
    )
    .await
    .context("Failed to start MASQUE client")?;

    tokio::spawn(client.run());

    // Connect to local UDP socket
    let proxy_client = UdpSocket::bind(any_localhost_addr).await?;
    proxy_client
        .connect(local_udp_addr)
        .await
        .context("Failed to connect to local UDP server")?;

    // Proxy client -> destination
    let mut rx_buf = BytesMut::with_capacity(128);
    proxy_client.send(b"abc").await?;
    let (_, proxy_addr) = target_udp_server
        .recv_buf_from(&mut rx_buf)
        .await
        .context("Expected to receive message")?;
    assert_eq!(&*rx_buf, b"abc", "Expected to receive message from client");

    // Destination -> proxy client
    let mut rx_buf = BytesMut::with_capacity(128);
    target_udp_server.send_to(b"def", proxy_addr).await?;
    proxy_client
        .recv_buf(&mut rx_buf)
        .await
        .context("Expected to receive message")?;
    assert_eq!(&*rx_buf, b"def", "Expected to receive message from server");

    Ok(())
}

async fn load_server_test_cert() -> anyhow::Result<rustls::ServerConfig> {
    let key = fs::read("tests/test.key").await.context("Read test key")?;
    let key = rustls_pemfile::private_key(&mut &*key)?.context("Invalid test key")?;

    let cert_chain = fs::read("tests/test.crt")
        .await
        .context("Read test certificate")?;
    let cert_chain = rustls_pemfile::certs(&mut &*cert_chain)
        .collect::<Result<_, _>>()
        .context("Invalid test certificate")?;

    let mut tls_config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_protocol_versions(&[&rustls::version::TLS13])?
    .with_no_client_auth()
    .with_single_cert(cert_chain, key)?;

    tls_config.max_early_data_size = u32::MAX;
    tls_config.alpn_protocols = vec![b"h3".into()];

    Ok(tls_config)
}
