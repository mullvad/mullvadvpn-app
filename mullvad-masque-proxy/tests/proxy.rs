use std::iter;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Context;
use bytes::BytesMut;
use mullvad_masque_proxy::MIN_IPV4_MTU;
use rand::RngCore;
use tokio::fs;

use mullvad_masque_proxy::client;
use mullvad_masque_proxy::server;
use tokio::net::UdpSocket;
use tokio::time::timeout;

/// Set up a MASQUE proxy and test that it can be used to communicate with some UDP destination
#[tokio::test]
async fn test_server_and_client_forwarding() -> anyhow::Result<()> {
    timeout(Duration::from_secs(1), async {
        const MTU: u16 = 1700;
        let (client, server) = setup_masque(MTU).await?;

        // Proxy client -> destination
        let mut rx_buf = BytesMut::with_capacity(128);
        client.send(b"abc").await?;
        let (_, proxy_addr) = server
            .recv_buf_from(&mut rx_buf)
            .await
            .context("Expected to receive message")?;
        assert_eq!(&*rx_buf, b"abc", "Expected to receive message from client");

        // Destination -> proxy client
        let mut rx_buf = BytesMut::with_capacity(128);
        server.send_to(b"def", proxy_addr).await?;
        client
            .recv_buf(&mut rx_buf)
            .await
            .context("Expected to receive message")?;
        assert_eq!(&*rx_buf, b"def", "Expected to receive message from server");

        Ok(())
    })
    .await?
}

/// End to end test with fragmentation.
/// Note: This doesn't actually check whether fragmentation occurs, only that packets actually
/// reach their destinations when fragmentation *should* be present.
#[tokio::test]
async fn test_server_and_client_fragmentation() -> anyhow::Result<()> {
    let mut valid_send_packet_sizes = vec![0, 1, 10, 100, 1280, 5000];

    // Maximum packet size sans UDP and QUIC headers, sans 1 byte context ID.
    //
    // NOTE: On macOS, the maximum UDP packet size is equal to the value set by
    // `sysctl net.inet.udp.maxdgram`
    #[cfg(not(target_os = "macos"))]
    valid_send_packet_sizes.push(u16::MAX - 8 - 41 - 1);

    let valid_mtus = [MIN_IPV4_MTU, 1280, 1500, 1700, 5000, 20000, u16::MAX];

    let params = valid_mtus
        .into_iter()
        .flat_map(|mtu| iter::repeat(mtu).zip(&valid_send_packet_sizes));

    async fn run_test(mtu: u16, send_packet_size: usize) -> anyhow::Result<()> {
        let (client, server) = setup_masque(mtu).await?;

        // Proxy client -> destination
        // Send a random packet, large enough to be fragmented
        let mut fragment_me = vec![0u8; send_packet_size];
        rand::thread_rng().fill_bytes(&mut fragment_me);

        client.send(&fragment_me).await?;

        let mut rx_buf = BytesMut::with_capacity(send_packet_size + 100);
        let (_, proxy_addr) = server
            .recv_buf_from(&mut rx_buf)
            .await
            .context("Expected to receive message")?;
        let read = rx_buf.split();
        assert_eq!(
            &*read, &fragment_me,
            "Expected to receive reassembled message from client"
        );

        // Destination -> proxy client
        // Send a random packet, large enough to be fragmented
        let mut fragment_me = vec![0u8; send_packet_size];
        rand::thread_rng().fill_bytes(&mut fragment_me);

        server.send_to(&fragment_me, proxy_addr).await?;

        let mut rx_buf = BytesMut::with_capacity(send_packet_size + 100);
        let blen = client
            .recv_buf(&mut rx_buf)
            .await
            .context("Expected to receive message")?;

        let read = rx_buf.split();
        eprintln!(
            "from server: {}, {}, {}",
            fragment_me.len(),
            read.len(),
            blen
        );
        assert_eq!(
            &*read, &fragment_me,
            "Expected to receive reassembled message from server"
        );

        Ok(())
    }

    for (mtu, &send_packet_size) in params {
        timeout(
            Duration::from_secs(1),
            run_test(mtu, send_packet_size.into()),
        )
        .await?
        .context(anyhow!("mtu={mtu}, send_packet_size={send_packet_size}"))?;
    }

    Ok(())
}

/// Set up a client and server connected by a MASQUE proxy.
/// This returns a UDP socket that is connected to the local MASQUE client,
/// and a UDP socket that represents the other endpoint.
/// Note that the server socket (second returned value) is not connected,
/// so `recv_from` must be used.
async fn setup_masque(mtu: u16) -> anyhow::Result<(UdpSocket, UdpSocket)> {
    const HOST: &str = "test.test";

    let any_localhost_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    // Set up destination UDP server
    let destination_udp_server = UdpSocket::bind(any_localhost_addr).await?;
    let target_udp_addr = destination_udp_server
        .local_addr()
        .context("Retrieve dest UDP server addr")?;

    // Set up MASQUE server
    let server_tls_config = load_server_test_cert().await?;
    let server = server::Server::bind(
        any_localhost_addr,
        Default::default(),
        Arc::new(server_tls_config),
        mtu,
    )
    .context("Failed to start MASQUE server")?;

    let masque_server_addr = server.local_addr()?;

    tokio::spawn(async move {
        if let Err(err) = server.run().await {
            eprintln!("server.run() failed: {err}");
        }
    });

    // Set up MASQUE client
    let local_socket = UdpSocket::bind(any_localhost_addr)
        .await
        .context("Failed to bind address")?;
    let masque_client_addr = local_socket.local_addr().unwrap();

    let client = client::Client::connect_with_tls_config(
        local_socket,
        masque_server_addr,
        // Local QUIC address
        any_localhost_addr,
        target_udp_addr,
        HOST,
        client::default_tls_config(),
        mtu,
    )
    .await
    .context("Failed to start MASQUE client")?;

    tokio::spawn(async move {
        if let Err(err) = client.run().await {
            eprintln!("client.run() failed: {err}");
        }
    });

    // Connect to local UDP socket
    let proxy_client = UdpSocket::bind(any_localhost_addr).await?;
    proxy_client
        .connect(masque_client_addr)
        .await
        .context("Failed to connect to local UDP server")?;

    Ok((proxy_client, destination_udp_server))
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
