//! End-to-end tests for the SOCKS5 `UDP ASSOCIATE` client, driven against a real SOCKS5 server.

use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use fast_socks5::{
    Socks5Command,
    server::{Socks5ServerProtocol, run_udp_proxy},
};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tunnel_obfuscation::socks5::{self, Socks5UdpAssociation};

const PAYLOAD: &[u8] = b"a wireguard packet, as far as the proxy is concerned";

/// Spawn a SOCKS5 server on localhost that only handles `UDP ASSOCIATE`, returning its address.
///
/// The server task is detached and lives until the test process exits.
async fn spawn_proxy() -> io::Result<SocketAddr> {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await?;
    let proxy_addr = listener.local_addr()?;

    tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                if let Err(error) = serve(socket).await {
                    eprintln!("SOCKS5 client failed: {error}");
                }
            });
        }
    });

    Ok(proxy_addr)
}

async fn serve(socket: TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let localhost = IpAddr::from(Ipv4Addr::LOCALHOST);

    let (protocol, command, target) = Socks5ServerProtocol::accept_no_auth(socket)
        .await?
        .read_command()
        .await?;
    assert_eq!(command, Socks5Command::UDPAssociate);

    run_udp_proxy(
        protocol,
        &target,
        Some(localhost),
        localhost,
        Some(localhost),
    )
    .await?;
    Ok(())
}

/// Spawn a UDP server that echoes every datagram back to its sender, returning its address.
async fn spawn_echo_server() -> io::Result<SocketAddr> {
    let socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await?;
    let echo_addr = socket.local_addr()?;

    tokio::spawn(async move {
        let mut buffer = [0u8; u16::MAX as usize];
        while let Ok((len, from)) = socket.recv_from(&mut buffer).await {
            let _ = socket.send_to(&buffer[..len], from).await;
        }
    });

    Ok(echo_addr)
}

/// Relay a datagram to `destination` through `relay_addr`, and return the reply along with the
/// address the reply claims to have come from.
async fn relay_datagram(
    socket: &UdpSocket,
    relay_addr: SocketAddr,
    destination: SocketAddr,
    payload: &[u8],
) -> io::Result<(SocketAddr, Vec<u8>)> {
    let mut datagram = Vec::new();
    socks5::encode_udp_datagram(destination, payload, &mut datagram);
    socket.send_to(&datagram, relay_addr).await?;

    let mut buffer = [0u8; u16::MAX as usize];
    let (len, _from) = socket.recv_from(&mut buffer).await?;
    let (origin, echoed) = socks5::decode_udp_datagram(&buffer[..len])
        .ok_or_else(|| io::Error::other("the proxy relayed a malformed datagram"))?;

    Ok((origin, echoed.to_vec()))
}

#[tokio::test]
async fn datagram_round_trips_through_the_proxy() {
    let proxy_addr = spawn_proxy().await.unwrap();
    let echo_addr = spawn_echo_server().await.unwrap();

    let control = TcpStream::connect(proxy_addr).await.unwrap();
    let association = Socks5UdpAssociation::associate(control, None)
        .await
        .unwrap();

    // The relay address must be usable as-is, even if the proxy replied with an unspecified one.
    let relay_addr = association.relay_endpoint();
    assert!(!relay_addr.ip().is_unspecified());

    let socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .await
        .unwrap();
    let (origin, echoed) = relay_datagram(&socket, relay_addr, echo_addr, PAYLOAD)
        .await
        .unwrap();

    assert_eq!(origin, echo_addr);
    assert_eq!(echoed, PAYLOAD);
}

/// The association must survive more than a single datagram, since a tunnel relies on it for its
/// entire lifetime.
#[tokio::test]
async fn association_relays_repeatedly() {
    const DATAGRAMS: usize = 10;

    let proxy_addr = spawn_proxy().await.unwrap();
    let echo_addr = spawn_echo_server().await.unwrap();

    let control = TcpStream::connect(proxy_addr).await.unwrap();
    let association = Socks5UdpAssociation::associate(control, None)
        .await
        .unwrap();

    let socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .await
        .unwrap();

    for i in 0..DATAGRAMS {
        let payload = format!("datagram {i}");
        let (origin, echoed) = relay_datagram(
            &socket,
            association.relay_endpoint(),
            echo_addr,
            payload.as_bytes(),
        )
        .await
        .unwrap();

        assert_eq!(origin, echo_addr);
        assert_eq!(echoed, payload.as_bytes());
    }
}
