//! Barebones SOCKS5 client supporting UDP ASSOCIATE only (RFC 1928).
//!
//! This performs the SOCKS5 control handshake over TCP, requests a UDP association, and exposes
//! the resulting UDP relay endpoint. It also provides pure encode/decode of the SOCKS5 UDP request
//! header used to frame datagrams sent to and received from the relay.

use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

use talpid_types::net::proxy::SocksAuth;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned, byteorder::network_endian::U16,
};

/// SOCKS protocol version (5).
const SOCKS_VERSION: u8 = 0x05;
/// Username/password authentication subnegotiation version (RFC 1929).
const AUTH_VERSION: u8 = 0x01;
/// "No authentication required" method.
const METHOD_NO_AUTH: u8 = 0x00;
/// Username/password authentication method (RFC 1929).
const METHOD_USERNAME_PASSWORD: u8 = 0x02;
/// "No acceptable methods" method.
const METHOD_UNACCEPTABLE: u8 = 0xff;
/// UDP ASSOCIATE command.
const CMD_UDP_ASSOCIATE: u8 = 0x03;
/// Reserved field value.
const RSV: u8 = 0x00;
/// "Succeeded" reply code, used both for requests and for authentication.
const REP_SUCCESS: u8 = 0x00;
/// A datagram that is not part of a fragment sequence. We never fragment.
const FRAG_STANDALONE: u8 = 0x00;

/// Address type: IPv4.
const ATYP_IPV4: u8 = 0x01;
/// Address type: domain name (unsupported).
const ATYP_DOMAIN: u8 = 0x03;
/// Address type: IPv6.
const ATYP_IPV6: u8 = 0x04;

/// `[VER][CMD][RSV]`, the part of a request that precedes the address.
const REQUEST_PREFIX_LEN: usize = 3;

/// The largest `ATYP + ADDR + PORT` encoding, which is the IPv6 one.
const MAX_ADDR_LEN: usize = 1 + size_of::<Ipv6Endpoint>();

/// The largest per-datagram header, which is the one for an IPv6 destination.
///
/// Prefer [`header_len`] when the destination is known.
pub const MAX_HEADER_LEN: usize = size_of::<UdpHeader>() + size_of::<Ipv6Endpoint>();

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The SOCKS5 control connection failed during the handshake.
    #[error("SOCKS5 control connection I/O failed")]
    ControlIo(#[source] io::Error),

    /// The server replied with an unexpected SOCKS version.
    #[error("Unexpected SOCKS version: {0:#04x}")]
    UnexpectedVersion(u8),

    /// The server did not accept any of the methods we offered.
    #[error("SOCKS5 proxy rejected our authentication methods")]
    NoAcceptableAuthMethod,

    /// The server selected a method we did not offer, or cannot perform.
    #[error("SOCKS5 proxy selected an unsupported authentication method: {0:#04x}")]
    UnsupportedAuthMethod(u8),

    /// The server rejected our username and password.
    #[error("SOCKS5 proxy rejected our credentials")]
    AuthenticationFailed,

    /// The server rejected the UDP ASSOCIATE request (`REP != 0`).
    #[error("SOCKS5 request rejected (REP={0:#04x})")]
    RequestRejected(u8),

    /// The server replied with an address type we do not support.
    #[error("Unsupported address type in SOCKS5 reply: {0:#04x}")]
    UnsupportedReplyAddress(u8),
}

/// A SOCKS5 UDP association. Owns the TCP control connection that keeps the association alive;
/// exposes the relay endpoint to send/receive UDP datagrams to.
#[derive(Debug)]
pub struct Socks5UdpAssociation {
    // Kept alive; dropping it tears down the association.
    //
    // TODO: Reading from this would let us notice that the proxy has torn the association down,
    // which currently manifests as datagrams being silently discarded.
    _control: TcpStream,
    relay_endpoint: SocketAddr,
}

impl Socks5UdpAssociation {
    /// Perform the SOCKS5 handshake and UDP ASSOCIATE over `control`, returning the association
    /// together with the relay endpoint.
    ///
    /// `control` must already be connected to the proxy. The caller owns socket setup, so that a
    /// firewall mark or a tunnel bypass can be applied before the handshake.
    ///
    /// # Blocking
    ///
    /// Waits for the proxy to answer the method negotiation, the optional authentication, and the
    /// associate request, so this blocks for as long as the proxy takes to respond to all three.
    /// Apply a timeout if that is a concern.
    pub async fn associate(mut control: TcpStream, auth: Option<&SocksAuth>) -> Result<Self> {
        negotiate_auth(&mut control, auth).await?;

        // Request: [VER][CMD=UDP ASSOCIATE][RSV][ATYP][DST.ADDR][DST.PORT].
        // We send an unspecified DST matching the proxy's address family, telling the relay to
        // accept from any client source. Our own source port is not known at this point, since the
        // caller binds its UDP socket separately.
        let peer_addr = control.peer_addr().map_err(Error::ControlIo)?;
        let dst = match peer_addr {
            SocketAddr::V4(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
            SocketAddr::V6(_) => SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0),
        };

        let mut request = [0u8; REQUEST_PREFIX_LEN + MAX_ADDR_LEN];
        request[..REQUEST_PREFIX_LEN].copy_from_slice(&[SOCKS_VERSION, CMD_UDP_ASSOCIATE, RSV]);
        let addr_len = write_socks_addr(&mut request[REQUEST_PREFIX_LEN..], dst);
        control
            .write_all(&request[..REQUEST_PREFIX_LEN + addr_len])
            .await
            .map_err(Error::ControlIo)?;

        // Reply: [VER][REP][RSV][ATYP][BND.ADDR][BND.PORT].
        let relay_endpoint = read_reply(&mut control).await?;

        // Proxies commonly reply with an unspecified address, meaning "the address you are already
        // talking to me on". Relaying datagrams there would fail.
        let relay_endpoint = if relay_endpoint.ip().is_unspecified() {
            SocketAddr::new(peer_addr.ip(), relay_endpoint.port())
        } else {
            relay_endpoint
        };

        Ok(Socks5UdpAssociation {
            _control: control,
            relay_endpoint,
        })
    }

    /// The UDP relay endpoint (BND.ADDR:BND.PORT) to send encapsulated datagrams to.
    pub fn relay_endpoint(&self) -> SocketAddr {
        self.relay_endpoint
    }
}

/// The length of the header that [`write_udp_header`] produces for `dst`.
pub const fn header_len(dst: &SocketAddr) -> usize {
    let addr_len = match dst {
        SocketAddr::V4(_) => size_of::<Ipv4Endpoint>(),
        SocketAddr::V6(_) => size_of::<Ipv6Endpoint>(),
    };
    size_of::<UdpHeader>() + addr_len
}

/// Write the SOCKS5 UDP request header for a datagram destined for `dst` to the start of `out`.
///
/// This is the in-place counterpart of [`encode_udp_datagram`], for callers that prepend the
/// header to a buffer already holding the payload, rather than building a new one.
///
/// # Panics
///
/// Panics if `out` is shorter than [`header_len`] for `dst`.
pub fn write_udp_header(dst: SocketAddr, out: &mut [u8]) {
    let len = header_len(&dst);
    assert!(
        out.len() >= len,
        "buffer of {} bytes is too small for a {len} byte SOCKS5 header",
        out.len(),
    );

    let (header_out, addr_out) = out.split_at_mut(size_of::<UdpHeader>());
    let atyp = match dst {
        SocketAddr::V4(dst) => write_endpoint(addr_out, Ipv4Endpoint::from(dst)),
        SocketAddr::V6(dst) => write_endpoint(addr_out, Ipv6Endpoint::from(dst)),
    };
    UdpHeader {
        rsv: [RSV; 2],
        frag: FRAG_STANDALONE,
        atyp,
    }
    .write_to_prefix(header_out)
    .expect("the buffer is exactly the size of the header");
}

/// Encapsulate `payload` destined for `dst` into a SOCKS5 UDP request datagram, writing the framed
/// bytes into `out` (cleared first). FRAG is always 0.
pub fn encode_udp_datagram(dst: SocketAddr, payload: &[u8], out: &mut Vec<u8>) {
    out.clear();
    out.resize(header_len(&dst), 0);
    write_udp_header(dst, out);
    out.extend_from_slice(payload);
}

/// Parse a SOCKS5 UDP datagram received from the relay.
///
/// Returns the source address and a slice of the inner payload.
///
/// Returns `None` (datagram dropped) when:
///   - the datagram is truncated/malformed, or
///   - `FRAG != 0` (fragment).
pub fn decode_udp_datagram(datagram: &[u8]) -> Option<(SocketAddr, &[u8])> {
    let (header, rest) = UdpHeader::read_from_prefix(datagram).ok()?;

    if header.frag != FRAG_STANDALONE {
        let frag = header.frag;
        log::trace!("Dropping fragmented SOCKS5 UDP datagram (FRAG={frag})");
        // TODO: implement fragment reassembly instead of dropping fragments.
        return None;
    }

    read_socks_addr(header.atyp, rest)
}

/// Negotiate an authentication method, and authenticate if the proxy asks us to.
async fn negotiate_auth(control: &mut TcpStream, auth: Option<&SocksAuth>) -> Result<()> {
    // Greeting: [VER][NMETHODS][METHODS..].
    let greeting = match auth {
        Some(_) => &[SOCKS_VERSION, 2, METHOD_NO_AUTH, METHOD_USERNAME_PASSWORD][..],
        None => &[SOCKS_VERSION, 1, METHOD_NO_AUTH],
    };
    control
        .write_all(greeting)
        .await
        .map_err(Error::ControlIo)?;

    // Method selection reply: [VER][METHOD].
    let mut reply = [0u8; 2];
    control
        .read_exact(&mut reply)
        .await
        .map_err(Error::ControlIo)?;
    let [version, method] = reply;
    if version != SOCKS_VERSION {
        return Err(Error::UnexpectedVersion(version));
    }

    match (method, auth) {
        (METHOD_NO_AUTH, _) => Ok(()),
        (METHOD_UNACCEPTABLE, _) => Err(Error::NoAcceptableAuthMethod),
        (METHOD_USERNAME_PASSWORD, Some(auth)) => authenticate(control, auth).await,
        // Anything else, including a method we have no credentials for.
        (method, _) => Err(Error::UnsupportedAuthMethod(method)),
    }
}

/// Authenticate with a username and password, as specified by
/// [RFC 1929](https://datatracker.ietf.org/doc/html/rfc1929).
async fn authenticate(control: &mut TcpStream, auth: &SocksAuth) -> Result<()> {
    let username = auth.username().as_bytes();
    let password = auth.password().as_bytes();

    // [VER][ULEN][UNAME][PLEN][PASSWD]. `SocksAuth` guarantees that both fit in a length byte.
    let mut request = vec![AUTH_VERSION, username.len() as u8];
    request.extend_from_slice(username);
    request.push(password.len() as u8);
    request.extend_from_slice(password);
    control
        .write_all(&request)
        .await
        .map_err(Error::ControlIo)?;

    // [VER][STATUS].
    let mut reply = [0u8; 2];
    control
        .read_exact(&mut reply)
        .await
        .map_err(Error::ControlIo)?;
    let [_version, status] = reply;
    if status != REP_SUCCESS {
        return Err(Error::AuthenticationFailed);
    }
    Ok(())
}

/// Read and validate a SOCKS5 request reply, returning the bound (relay) endpoint.
async fn read_reply(control: &mut TcpStream) -> Result<SocketAddr> {
    // [VER][REP][RSV][ATYP].
    let mut head = [0u8; 4];
    control
        .read_exact(&mut head)
        .await
        .map_err(Error::ControlIo)?;
    let [version, reply, _rsv, atyp] = head;
    if version != SOCKS_VERSION {
        return Err(Error::UnexpectedVersion(version));
    }
    if reply != REP_SUCCESS {
        return Err(Error::RequestRejected(reply));
    }

    match atyp {
        ATYP_IPV4 => read_endpoint::<Ipv4Endpoint>(control).await,
        ATYP_IPV6 => read_endpoint::<Ipv6Endpoint>(control).await,
        // Domain names (0x03) are not supported; we only deal in IP endpoints.
        atyp => Err(Error::UnsupportedReplyAddress(atyp)),
    }
}

/// Read an address and port of a known type off the control connection.
async fn read_endpoint<E: Endpoint>(control: &mut TcpStream) -> Result<SocketAddr> {
    let mut endpoint = E::new_zeroed();
    control
        .read_exact(endpoint.as_mut_bytes())
        .await
        .map_err(Error::ControlIo)?;
    Ok(endpoint.socket_addr())
}

/// Write an `ATYP`-prefixed address to the start of `out`, returning the number of bytes written.
///
/// # Panics
///
/// Panics if `out` is too small to hold the address.
fn write_socks_addr(out: &mut [u8], addr: SocketAddr) -> usize {
    let (atyp, addr_out) = out.split_at_mut(1);
    atyp[0] = match addr {
        SocketAddr::V4(addr) => write_endpoint(addr_out, Ipv4Endpoint::from(addr)),
        SocketAddr::V6(addr) => write_endpoint(addr_out, Ipv6Endpoint::from(addr)),
    };
    1 + addr_len(&addr)
}

/// Parse an `ATYP`-prefixed address of type `atyp` from `data`, returning the address and the
/// remaining bytes. Domain names (0x03) are unsupported and yield `None`.
fn read_socks_addr(atyp: u8, data: &[u8]) -> Option<(SocketAddr, &[u8])> {
    match atyp {
        ATYP_IPV4 => read_endpoint_prefix::<Ipv4Endpoint>(data),
        ATYP_IPV6 => read_endpoint_prefix::<Ipv6Endpoint>(data),
        // Domain names are unsupported; we only ever deal in IP endpoints.
        ATYP_DOMAIN => None,
        // Unknown address types.
        _ => None,
    }
}

fn read_endpoint_prefix<E: Endpoint>(data: &[u8]) -> Option<(SocketAddr, &[u8])> {
    let (endpoint, rest) = E::read_from_prefix(data).ok()?;
    Some((endpoint.socket_addr(), rest))
}

/// Write an endpoint to the start of `out`, returning its `ATYP`.
fn write_endpoint<E: Endpoint>(out: &mut [u8], endpoint: E) -> u8 {
    endpoint
        .write_to_prefix(out)
        .expect("the caller ensures the buffer is large enough");
    E::ATYP
}

fn addr_len(addr: &SocketAddr) -> usize {
    match addr {
        SocketAddr::V4(_) => size_of::<Ipv4Endpoint>(),
        SocketAddr::V6(_) => size_of::<Ipv6Endpoint>(),
    }
}

/// Leading fixed-layout prefix of a SOCKS5 UDP datagram: `[RSV(2)][FRAG(1)][ATYP(1)]`.
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(C)]
struct UdpHeader {
    rsv: [u8; 2],
    frag: u8,
    atyp: u8,
}

/// An address followed by a big-endian port, as carried in SOCKS5 messages.
trait Endpoint: FromBytes + IntoBytes + Immutable + KnownLayout + Unaligned + Sized {
    /// The `ATYP` value identifying this kind of address.
    const ATYP: u8;

    fn socket_addr(&self) -> SocketAddr;
}

/// An IPv4 address followed by a big-endian port, as carried in SOCKS5 messages.
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(C)]
struct Ipv4Endpoint {
    addr: [u8; 4],
    port: U16,
}

impl Endpoint for Ipv4Endpoint {
    const ATYP: u8 = ATYP_IPV4;

    fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::from(self.addr)), self.port.get())
    }
}

impl From<SocketAddrV4> for Ipv4Endpoint {
    fn from(addr: SocketAddrV4) -> Self {
        Ipv4Endpoint {
            addr: addr.ip().octets(),
            port: addr.port().into(),
        }
    }
}

/// An IPv6 address followed by a big-endian port, as carried in SOCKS5 messages.
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(C)]
struct Ipv6Endpoint {
    addr: [u8; 16],
    port: U16,
}

impl Endpoint for Ipv6Endpoint {
    const ATYP: u8 = ATYP_IPV6;

    fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V6(Ipv6Addr::from(self.addr)), self.port.get())
    }
}

impl From<SocketAddrV6> for Ipv6Endpoint {
    fn from(addr: SocketAddrV6) -> Self {
        Ipv6Endpoint {
            addr: addr.ip().octets(),
            port: addr.port().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::net::TcpListener;

    use super::*;

    const IPV4_DESTINATION: &str = "192.0.2.10:4711";
    const IPV6_DESTINATION: &str = "[2001:db8::1]:51820";

    const RELAY: &str = "127.0.0.1:9999";

    /// Serve one handshake from a hand-rolled server, asserting the exact bytes we send, and
    /// return the association the client ended up with.
    ///
    /// `expected_methods` is the method list the greeting must offer, and `credentials` the
    /// username and password the RFC 1929 exchange must carry, if any.
    async fn handshake(
        auth: Option<SocksAuth>,
        expected_methods: &'static [u8],
        credentials: Option<(&'static str, &'static str)>,
    ) -> Result<Socks5UdpAssociation> {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let proxy_addr = listener.local_addr().unwrap();
        let relay: SocketAddr = RELAY.parse().unwrap();

        let server = tokio::spawn(async move {
            let (mut conn, _) = listener.accept().await.unwrap();

            // Greeting: [VER][NMETHODS][METHODS..].
            let mut greeting = vec![0u8; 2 + expected_methods.len()];
            conn.read_exact(&mut greeting).await.unwrap();
            assert_eq!(greeting[0], SOCKS_VERSION);
            assert_eq!(usize::from(greeting[1]), expected_methods.len());
            assert_eq!(&greeting[2..], expected_methods);

            let method = match credentials {
                Some(_) => METHOD_USERNAME_PASSWORD,
                None => METHOD_NO_AUTH,
            };
            conn.write_all(&[SOCKS_VERSION, method]).await.unwrap();

            // [VER][ULEN][UNAME][PLEN][PASSWD].
            if let Some((username, password)) = credentials {
                let mut expected = vec![AUTH_VERSION, username.len() as u8];
                expected.extend_from_slice(username.as_bytes());
                expected.push(password.len() as u8);
                expected.extend_from_slice(password.as_bytes());

                let mut request = vec![0u8; expected.len()];
                conn.read_exact(&mut request).await.unwrap();
                assert_eq!(request, expected);

                conn.write_all(&[AUTH_VERSION, REP_SUCCESS]).await.unwrap();
            }

            // Request: [VER][CMD=UDP ASSOCIATE][RSV][ATYP=IPv4][addr(4)][port(2)].
            let mut request = [0u8; REQUEST_PREFIX_LEN + 1 + 4 + 2];
            conn.read_exact(&mut request).await.unwrap();
            assert_eq!(request[0], SOCKS_VERSION);
            assert_eq!(request[1], CMD_UDP_ASSOCIATE);
            assert_eq!(request[3], ATYP_IPV4);

            // Reply: [VER][REP=0x00][RSV][ATYP=IPv4][BND.ADDR][BND.PORT].
            let mut reply = vec![0u8; REQUEST_PREFIX_LEN + MAX_ADDR_LEN];
            reply[..REQUEST_PREFIX_LEN].copy_from_slice(&[SOCKS_VERSION, REP_SUCCESS, RSV]);
            let addr_len = write_socks_addr(&mut reply[REQUEST_PREFIX_LEN..], relay);
            reply.truncate(REQUEST_PREFIX_LEN + addr_len);
            conn.write_all(&reply).await.unwrap();

            // Keep the control connection open until the client is done.
            let mut scratch = [0u8; 1];
            let _ = conn.read(&mut scratch).await;
        });

        let control = TcpStream::connect(proxy_addr).await.unwrap();
        let association = Socks5UdpAssociation::associate(control, auth.as_ref()).await;
        server.abort();

        association
    }

    #[tokio::test]
    async fn handshake_returns_relay_endpoint() {
        let association = handshake(None, &[METHOD_NO_AUTH], None).await.unwrap();
        assert_eq!(association.relay_endpoint(), RELAY.parse().unwrap());
    }

    /// Credentials must actually reach the proxy, rather than being silently dropped.
    #[tokio::test]
    async fn handshake_authenticates_with_credentials() {
        let auth = SocksAuth::new("FooBar".to_owned(), "hunter2".to_owned()).unwrap();

        let association = handshake(
            Some(auth),
            &[METHOD_NO_AUTH, METHOD_USERNAME_PASSWORD],
            Some(("FooBar", "hunter2")),
        )
        .await
        .unwrap();

        assert_eq!(association.relay_endpoint(), RELAY.parse().unwrap());
    }

    #[test]
    fn roundtrip_ipv4() {
        let dst: SocketAddr = IPV4_DESTINATION.parse().unwrap();
        let payload = b"hello wireguard";

        let mut buf = Vec::new();
        encode_udp_datagram(dst, payload, &mut buf);

        let (src, decoded) = decode_udp_datagram(&buf).unwrap();
        assert_eq!(src, dst);
        assert_eq!(decoded, payload);
    }

    #[test]
    fn roundtrip_ipv6() {
        let dst: SocketAddr = IPV6_DESTINATION.parse().unwrap();
        let payload = b"hello over ipv6";

        let mut buf = Vec::new();
        encode_udp_datagram(dst, payload, &mut buf);

        let (src, decoded) = decode_udp_datagram(&buf).unwrap();
        assert_eq!(src, dst);
        assert_eq!(decoded, payload);
    }

    #[test]
    fn encode_clears_output() {
        let dst: SocketAddr = "192.0.2.10:80".parse().unwrap();
        let mut buf = vec![0xff, 0xff, 0xff];
        encode_udp_datagram(dst, b"x", &mut buf);
        // The pre-existing junk must be gone (header begins with RSV=0x0000).
        assert_eq!(&buf[0..3], &[0x00, 0x00, 0x00]);
    }

    #[test]
    fn known_wire_bytes_ipv4() {
        let dst: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let mut buf = Vec::new();
        encode_udp_datagram(dst, b"abc", &mut buf);

        let expected = [
            0x00, 0x00, // RSV
            0x00, // FRAG
            0x01, // ATYP = IPv4
            127, 0, 0, 1, // DST.ADDR
            0x1f, 0x90, // DST.PORT = 8080, big-endian
            b'a', b'b', b'c', // DATA
        ];
        assert_eq!(buf, expected);
    }

    #[test]
    fn known_wire_bytes_ipv6() {
        let dst: SocketAddr = "[::1]:8080".parse().unwrap();
        let mut buf = Vec::new();
        encode_udp_datagram(dst, b"abc", &mut buf);

        let expected = [
            0x00, 0x00, // RSV
            0x00, // FRAG
            0x04, // ATYP = IPv6
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // DST.ADDR
            0x1f, 0x90, // DST.PORT = 8080, big-endian
            b'a', b'b', b'c', // DATA
        ];
        assert_eq!(buf, expected);
    }

    /// The header sizes that the tunnel MTU calculation assumes.
    #[test]
    fn header_lengths() {
        assert_eq!(header_len(&IPV4_DESTINATION.parse().unwrap()), 10);
        assert_eq!(header_len(&IPV6_DESTINATION.parse().unwrap()), 22);
        assert_eq!(MAX_HEADER_LEN, 22);
    }

    #[test]
    fn fragment_is_dropped() {
        let dst: SocketAddr = IPV4_DESTINATION.parse().unwrap();
        let mut buf = Vec::new();
        encode_udp_datagram(dst, b"data", &mut buf);
        // Set FRAG (byte index 2) to a non-zero value.
        buf[2] = 0x01;
        assert!(decode_udp_datagram(&buf).is_none());
    }

    /// No prefix of a valid datagram may be mistaken for a complete one.
    #[test]
    fn truncated_datagrams_are_dropped() {
        let dst: SocketAddr = IPV6_DESTINATION.parse().unwrap();
        let mut buf = Vec::new();
        encode_udp_datagram(dst, b"data", &mut buf);

        for len in 0..header_len(&dst) {
            assert!(
                decode_udp_datagram(&buf[..len]).is_none(),
                "expected a {len} byte datagram to be dropped as truncated"
            );
        }
    }

    #[test]
    fn unsupported_domain_atyp_is_dropped() {
        let datagram = [
            0x00,
            0x00,        // RSV
            0x00,        // FRAG
            ATYP_DOMAIN, // ATYP = domain name (unsupported)
            0x03,
            b'a',
            b'b',
            b'c', // length-prefixed domain
            0x00,
            0x50, // port
        ];
        assert!(decode_udp_datagram(&datagram).is_none());
    }
}
