//! End-to-end tests for the SOCKS5 `UDP ASSOCIATE` client, driven against a real SOCKS5 server.
//!
//! The server below implements just enough of RFC 1928 to answer an associate request and relay
//! datagrams for it. It is spelled out byte by byte rather than built on [`socks5`], so that a
//! mistake in the wire format cannot cancel itself out against the client under test.

use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

use talpid_types::net::proxy::SocksAuth;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    time::timeout,
};
use tunnel_obfuscation::socks5::{self, Socks5UdpAssociation};

const PAYLOAD: &[u8] = b"a wireguard packet, as far as the proxy is concerned";

const USERNAME: &str = "mullvad";
const PASSWORD: &str = "hunter2";

/// SOCKS protocol version (5).
const SOCKS_VERSION: u8 = 0x05;
/// Username/password authentication subnegotiation version (RFC 1929).
const AUTH_VERSION: u8 = 0x01;
/// "No authentication required" method.
const METHOD_NO_AUTH: u8 = 0x00;
/// Username/password authentication method (RFC 1929).
const METHOD_USERNAME_PASSWORD: u8 = 0x02;
/// UDP ASSOCIATE command.
const CMD_UDP_ASSOCIATE: u8 = 0x03;
/// Reserved field value.
const RSV: u8 = 0x00;
/// "Succeeded" reply code, used both for requests and for authentication.
const REP_SUCCESS: u8 = 0x00;
/// A datagram that is not part of a fragment sequence.
const FRAG_STANDALONE: u8 = 0x00;
/// Address type: IPv4.
const ATYP_IPV4: u8 = 0x01;

/// How the test proxy behaves.
#[derive(Clone, Copy, Default)]
struct ProxyConfig {
    /// Demand username/password authentication rather than accepting the no-auth method.
    require_auth: bool,
    /// Answer with an unspecified BND.ADDR, meaning "the address you are already talking to me
    /// on". Real proxies commonly reply this way.
    unspecified_bind_addr: bool,
}

/// Spawn a SOCKS5 server on localhost that only handles `UDP ASSOCIATE`, returning its address.
///
/// The server task is detached and lives until the test process exits.
async fn spawn_proxy(config: ProxyConfig) -> io::Result<SocketAddr> {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await?;
    let proxy_addr = listener.local_addr()?;

    tokio::spawn(async move {
        while let Ok((control, _)) = listener.accept().await {
            tokio::spawn(async move {
                if let Err(error) = serve(control, config).await {
                    eprintln!("SOCKS5 client failed: {error}");
                }
            });
        }
    });

    Ok(proxy_addr)
}

/// Handshake with one client, then relay its datagrams until the process exits.
async fn serve(mut control: TcpStream, config: ProxyConfig) -> io::Result<()> {
    negotiate_auth(&mut control, config.require_auth).await?;
    let relay = accept_udp_associate(&mut control, config.unspecified_bind_addr).await?;

    // Dropping the control connection is what tears an association down, so hold it open for as
    // long as datagrams are relayed.
    let _control = control;
    relay_datagrams(relay).await
}

/// Read the method selection greeting and select a method, authenticating if one is demanded.
async fn negotiate_auth(control: &mut TcpStream, require_auth: bool) -> io::Result<()> {
    // Greeting: [VER][NMETHODS][METHODS..].
    let mut greeting = [0u8; 2];
    control.read_exact(&mut greeting).await?;
    let [version, method_count] = greeting;
    assert_eq!(version, SOCKS_VERSION, "unexpected version in the greeting");

    let mut methods = vec![0u8; usize::from(method_count)];
    control.read_exact(&mut methods).await?;

    if !require_auth {
        assert!(
            methods.contains(&METHOD_NO_AUTH),
            "the client did not offer the no-auth method"
        );
        return control.write_all(&[SOCKS_VERSION, METHOD_NO_AUTH]).await;
    }

    assert!(
        methods.contains(&METHOD_USERNAME_PASSWORD),
        "the client did not offer username/password authentication"
    );
    control
        .write_all(&[SOCKS_VERSION, METHOD_USERNAME_PASSWORD])
        .await?;

    // [VER][ULEN][UNAME][PLEN][PASSWD], per RFC 1929.
    let mut version = [0u8; 1];
    control.read_exact(&mut version).await?;
    assert_eq!(version[0], AUTH_VERSION, "unexpected auth version");

    let username = read_length_prefixed(control).await?;
    let password = read_length_prefixed(control).await?;
    assert_eq!(username, USERNAME.as_bytes(), "wrong username");
    assert_eq!(password, PASSWORD.as_bytes(), "wrong password");

    // [VER][STATUS].
    control.write_all(&[AUTH_VERSION, REP_SUCCESS]).await
}

/// Read a length byte followed by that many bytes.
async fn read_length_prefixed(control: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut len = [0u8; 1];
    control.read_exact(&mut len).await?;

    let mut bytes = vec![0u8; usize::from(len[0])];
    control.read_exact(&mut bytes).await?;
    Ok(bytes)
}

/// Read the `UDP ASSOCIATE` request, bind a relay socket for it, and reply with the socket's
/// address.
async fn accept_udp_associate(
    control: &mut TcpStream,
    unspecified_bind_addr: bool,
) -> io::Result<UdpSocket> {
    // Request: [VER][CMD][RSV][ATYP][DST.ADDR][DST.PORT].
    let mut request = [0u8; 4];
    control.read_exact(&mut request).await?;
    let [version, command, reserved, address_type] = request;
    assert_eq!(version, SOCKS_VERSION, "unexpected version in the request");
    assert_eq!(command, CMD_UDP_ASSOCIATE, "expected a UDP ASSOCIATE");
    assert_eq!(reserved, RSV, "RSV must be zero");
    assert_eq!(address_type, ATYP_IPV4, "expected an IPv4 DST.ADDR");

    // DST tells a proxy which source to accept relayed datagrams from. This one relays for
    // whoever shows up first instead, so it is only read to keep the stream in sync.
    let mut destination = [0u8; 6];
    control.read_exact(&mut destination).await?;

    let relay = UdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await?;
    let bind_addr = match unspecified_bind_addr {
        true => Ipv4Addr::UNSPECIFIED,
        false => Ipv4Addr::LOCALHOST,
    };

    // Reply: [VER][REP][RSV][ATYP][BND.ADDR][BND.PORT].
    let mut reply = vec![SOCKS_VERSION, REP_SUCCESS, RSV, ATYP_IPV4];
    reply.extend_from_slice(&bind_addr.octets());
    reply.extend_from_slice(&relay.local_addr()?.port().to_be_bytes());
    control.write_all(&reply).await?;

    Ok(relay)
}

/// Relay datagrams between the client and whatever destinations it addresses.
async fn relay_datagrams(relay: UdpSocket) -> io::Result<()> {
    let outbound = UdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await?;
    let mut client = None;

    let mut from_client = vec![0u8; usize::from(u16::MAX)];
    let mut from_destination = vec![0u8; usize::from(u16::MAX)];

    loop {
        tokio::select! {
            result = relay.recv_from(&mut from_client) => {
                let (len, sender) = result?;
                client = Some(sender);

                let (destination, payload) = decapsulate(&from_client[..len]);
                outbound.send_to(payload, destination).await?;
            }
            result = outbound.recv_from(&mut from_destination) => {
                let (len, origin) = result?;
                let Some(client) = client else { continue };

                let datagram = encapsulate(origin, &from_destination[..len]);
                relay.send_to(&datagram, client).await?;
            }
        }
    }
}

/// Split a datagram from the client into the destination it names and the payload behind it.
fn decapsulate(datagram: &[u8]) -> (SocketAddr, &[u8]) {
    // [RSV][RSV][FRAG][ATYP][DST.ADDR][DST.PORT][DATA], for an IPv4 DST.ADDR.
    let [rsv_high, rsv_low, fragment, address_type, a, b, c, d, port_high, port_low, payload @ ..] =
        datagram
    else {
        panic!("truncated datagram from the client");
    };
    assert_eq!([*rsv_high, *rsv_low], [RSV, RSV], "RSV must be zero");
    assert_eq!(*fragment, FRAG_STANDALONE, "the client must not fragment");
    assert_eq!(*address_type, ATYP_IPV4, "expected an IPv4 destination");

    let destination = SocketAddr::from((
        Ipv4Addr::new(*a, *b, *c, *d),
        u16::from_be_bytes([*port_high, *port_low]),
    ));

    (destination, payload)
}

/// Frame `payload` as having arrived from `origin`, the way a proxy hands it back to the client.
fn encapsulate(origin: SocketAddr, payload: &[u8]) -> Vec<u8> {
    let SocketAddr::V4(origin) = origin else {
        panic!("the test destinations are all IPv4");
    };

    let mut datagram = vec![RSV, RSV, FRAG_STANDALONE, ATYP_IPV4];
    datagram.extend_from_slice(&origin.ip().octets());
    datagram.extend_from_slice(&origin.port().to_be_bytes());
    datagram.extend_from_slice(payload);
    datagram
}

/// Spawn a UDP server that echoes every datagram back to its sender, returning its address.
async fn spawn_echo_server() -> io::Result<SocketAddr> {
    let socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await?;
    let echo_addr = socket.local_addr()?;

    tokio::spawn(async move {
        let mut buffer = vec![0u8; usize::from(u16::MAX)];
        while let Ok((len, from)) = socket.recv_from(&mut buffer).await {
            let _ = socket.send_to(&buffer[..len], from).await;
        }
    });

    Ok(echo_addr)
}

/// How long to wait for the proxy to relay a reply.
///
/// Everything here runs on localhost, so exceeding this means the datagram was never relayed
/// back. A malformed one makes the proxy reject it, which would otherwise hang the test forever
/// rather than fail it.
const RELAY_TIMEOUT: Duration = Duration::from_secs(10);

/// Relay a datagram to `destination` through `relay_addr`, and return the reply along with the
/// address the reply claims to have come from.
///
/// # Blocking
///
/// Waits for the reply to come back, for at most [`RELAY_TIMEOUT`].
async fn relay_datagram(
    socket: &UdpSocket,
    relay_addr: SocketAddr,
    destination: SocketAddr,
    payload: &[u8],
) -> io::Result<(SocketAddr, Vec<u8>)> {
    let mut datagram = Vec::new();
    socks5::encode_udp_datagram(destination, payload, &mut datagram);
    socket.send_to(&datagram, relay_addr).await?;

    let mut buffer = vec![0u8; usize::from(u16::MAX)];
    let (len, _from) = timeout(RELAY_TIMEOUT, socket.recv_from(&mut buffer))
        .await
        .map_err(|_| io::Error::other("the proxy never relayed the datagram back"))??;

    let (origin, echoed) = socks5::decode_udp_datagram(&buffer[..len])
        .ok_or_else(|| io::Error::other("the proxy relayed a malformed datagram"))?;

    Ok((origin, echoed.to_vec()))
}

/// Bind the UDP socket that a client sends encapsulated datagrams from.
async fn bind_client_socket() -> UdpSocket {
    UdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .await
        .unwrap()
}

#[tokio::test]
async fn datagram_round_trips_through_the_proxy() {
    let proxy_addr = spawn_proxy(ProxyConfig::default()).await.unwrap();
    let echo_addr = spawn_echo_server().await.unwrap();

    let control = TcpStream::connect(proxy_addr).await.unwrap();
    let association = Socks5UdpAssociation::associate(control, None)
        .await
        .unwrap();

    let socket = bind_client_socket().await;
    let (origin, echoed) = relay_datagram(&socket, association.relay_endpoint(), echo_addr, PAYLOAD)
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

    let proxy_addr = spawn_proxy(ProxyConfig::default()).await.unwrap();
    let echo_addr = spawn_echo_server().await.unwrap();

    let control = TcpStream::connect(proxy_addr).await.unwrap();
    let association = Socks5UdpAssociation::associate(control, None)
        .await
        .unwrap();

    let socket = bind_client_socket().await;

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

/// A proxy that answers with an unspecified BND.ADDR means "the address you are already talking to
/// me on", so the relay endpoint must come out usable rather than pointing at `0.0.0.0`.
#[tokio::test]
async fn unspecified_bind_address_resolves_to_the_proxy() {
    let config = ProxyConfig {
        unspecified_bind_addr: true,
        ..ProxyConfig::default()
    };
    let proxy_addr = spawn_proxy(config).await.unwrap();
    let echo_addr = spawn_echo_server().await.unwrap();

    let control = TcpStream::connect(proxy_addr).await.unwrap();
    let association = Socks5UdpAssociation::associate(control, None)
        .await
        .unwrap();

    let relay_addr = association.relay_endpoint();
    assert_eq!(relay_addr.ip(), proxy_addr.ip());

    let socket = bind_client_socket().await;
    let (origin, echoed) = relay_datagram(&socket, relay_addr, echo_addr, PAYLOAD)
        .await
        .unwrap();

    assert_eq!(origin, echo_addr);
    assert_eq!(echoed, PAYLOAD);
}

/// A proxy that demands credentials must be answered with the RFC 1929 subnegotiation before the
/// associate request is sent.
#[tokio::test]
async fn association_authenticates_with_username_and_password() {
    let config = ProxyConfig {
        require_auth: true,
        ..ProxyConfig::default()
    };
    let proxy_addr = spawn_proxy(config).await.unwrap();
    let echo_addr = spawn_echo_server().await.unwrap();

    let auth = SocksAuth::new(USERNAME.to_owned(), PASSWORD.to_owned()).unwrap();
    let control = TcpStream::connect(proxy_addr).await.unwrap();
    let association = Socks5UdpAssociation::associate(control, Some(&auth))
        .await
        .unwrap();

    let socket = bind_client_socket().await;
    let (origin, echoed) = relay_datagram(&socket, association.relay_endpoint(), echo_addr, PAYLOAD)
        .await
        .unwrap();

    assert_eq!(origin, echo_addr);
    assert_eq!(echoed, PAYLOAD);
}
