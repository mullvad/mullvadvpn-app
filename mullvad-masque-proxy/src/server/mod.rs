use std::{
    collections::HashSet,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use anyhow::{Context, anyhow, ensure};
use bytes::{Bytes, BytesMut};
use h3::{
    proto::varint::VarInt,
    quic::{BidiStream, StreamId},
    server::{self, Connection, RequestStream},
};
use h3_datagram::{datagram::Datagram, datagram_traits::HandleDatagramsExt};
use http::{StatusCode, Uri, header};
use quinn::{Endpoint, Incoming, crypto::rustls::QuicServerConfig};
use tokio::{net::UdpSocket, select, sync::mpsc, task};
use typed_builder::TypedBuilder;

use crate::{
    MASQUE_WELL_KNOWN_PATH, MAX_INFLIGHT_PACKETS, MIN_IPV4_MTU, MIN_IPV6_MTU, QUIC_HEADER_SIZE,
    compute_udp_payload_size,
    fragment::{self, DefragReceived, Fragments},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Bad TLS config")]
    BadTlsConfig(#[source] quinn::crypto::rustls::NoInitialCipherSuite),
    #[error("Failed to bind server socket")]
    BindSocket(#[source] io::Error),
    #[error("Failed to send negotiation response")]
    SendNegotiationResponse(#[source] h3::Error),
    #[error("Invalid MTU: must be at least {min_mtu}")]
    InvalidMtu { min_mtu: u16 },
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Server {
    endpoint: Endpoint,
    params: Arc<ServerParams>,
}

#[derive(TypedBuilder)]
pub struct ServerParams {
    /// Allowed target IPs for the proxy connection
    pub allowed_hosts: AllowedIps,

    /// Server hostname expected from clients
    #[builder(default)]
    pub hostname: Option<String>,

    /// Maximum transfer unit
    // TODO: remove this
    #[builder(default = 1500)]
    pub mtu: u16,

    /// Authorization header expected from clients
    #[builder(default)]
    pub auth_header: Option<String>,
}

#[derive(Default, Clone)]
pub struct AllowedIps {
    hosts: Arc<HashSet<IpAddr>>,
}

impl<T: IntoIterator<Item = IpAddr>> From<T> for AllowedIps {
    fn from(value: T) -> Self {
        AllowedIps {
            hosts: Arc::new(value.into_iter().collect()),
        }
    }
}

impl AllowedIps {
    fn ip_allowed(&self, ip: IpAddr) -> bool {
        self.hosts.is_empty() || self.hosts.contains(&ip)
    }
}

impl Server {
    pub fn bind(
        bind_addr: SocketAddr,
        tls_config: Arc<rustls::ServerConfig>,
        params: ServerParams,
    ) -> Result<Self> {
        Self::validate_mtu(params.mtu, bind_addr)?;

        let server_config = quinn::ServerConfig::with_crypto(Arc::new(
            QuicServerConfig::try_from(tls_config).map_err(Error::BadTlsConfig)?,
        ));

        let endpoint = Endpoint::server(server_config, bind_addr).map_err(Error::BindSocket)?;

        Ok(Self {
            endpoint,
            params: Arc::new(params),
        })
    }

    const fn validate_mtu(mtu: u16, bind_addr: SocketAddr) -> Result<()> {
        let min_mtu = if bind_addr.is_ipv4() {
            MIN_IPV4_MTU
        } else {
            MIN_IPV6_MTU
        };
        if mtu >= min_mtu {
            Ok(())
        } else {
            Err(Error::InvalidMtu { min_mtu })
        }
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.endpoint.local_addr()
    }

    pub async fn run(self) -> Result<()> {
        while let Some(new_connection) = self.endpoint.accept().await {
            tokio::spawn(Self::handle_incoming_connection(
                new_connection,
                Arc::clone(&self.params),
            ));
        }
        Ok(())
    }

    async fn handle_incoming_connection(connection: Incoming, server_params: Arc<ServerParams>) {
        let conn = match connection.await {
            Ok(conn) => conn,
            Err(err) => {
                log::error!("accepting connection failed: {:?}", err);
                return;
            }
        };

        log::debug!("new connection established");

        let quinn_conn = conn.clone();

        let Ok(connection) = server::builder()
            .enable_datagram(true)
            .build(h3_quinn::Connection::new(conn))
            .await
        else {
            log::error!("Failed to construct a new H3 server connection");
            return;
        };

        Self::accept_proxy_request(quinn_conn, connection, server_params).await;
    }

    /// Accept an HTTP request and try to handle it as a proxy request.
    async fn accept_proxy_request(
        quic_conn: quinn::Connection,
        mut http_conn: Connection<h3_quinn::Connection, Bytes>,
        server_params: Arc<ServerParams>,
    ) {
        let (http_request, mut stream) = match http_conn.accept().await {
            Ok(Some((req, stream))) => (req, stream),

            // indicating no more streams to be received
            Ok(None) => return,

            Err(err) => {
                log::error!("error on accept {}", err);
                return;
            }
        };

        let proxy_uri = match ProxyUri::try_from(http_request.uri()) {
            Ok(proxy_uri) => proxy_uri,
            Err(e) => {
                log::debug!("Bad proxy URI: {e}");
                return;
            }
        };

        if let Some(required_auth) = &server_params.auth_header {
            match http_request.headers().get(header::AUTHORIZATION) {
                Some(actual_auth) if actual_auth == required_auth => (),
                _ => return handle_invalid_auth(stream).await,
            }
        }

        if let Some(hostname) = &server_params.hostname
            && &proxy_uri.hostname != hostname
        {
            let valid_uri = ProxyUri {
                hostname: hostname.clone(),
                ..proxy_uri
            };

            respond_with_redirect(stream, valid_uri).await;

            // NOTE: Recursing like this makes us vulnerable to DoS if the client keeps
            // sending the wrong hostname. This is fine since this is just an example server.
            Box::pin(Self::accept_proxy_request(
                quic_conn,
                http_conn,
                server_params,
            ))
            .await;

            return;
        }

        if !server_params
            .allowed_hosts
            .ip_allowed(proxy_uri.target_addr.ip())
        {
            return handle_disallowed_ip(stream).await;
        }

        let bind_addr = SocketAddr::new(unspecified_addr(proxy_uri.target_addr.ip()), 0);
        let Ok(udp_socket) = UdpSocket::bind(bind_addr).await else {
            return handle_failed_socket(stream).await;
        };
        if let Err(err) = udp_socket.connect(proxy_uri.target_addr).await {
            log::error!("Failed to set destination for UDP socket: {err}");
            return handle_failed_socket(stream).await;
        };

        if handle_established_connection(&mut stream).await.is_err() {
            return;
        }

        let stream_id = stream.id();
        let udp_socket = Arc::new(udp_socket);
        let (client_tx, client_rx) = mpsc::channel(MAX_INFLIGHT_PACKETS);
        let (send_tx, send_rx) = mpsc::channel(MAX_INFLIGHT_PACKETS);

        let mut connection_task =
            task::spawn(connection_task(stream_id, http_conn, send_rx, client_tx));
        let mut proxy_rx_task = task::spawn(proxy_rx_task(
            stream_id,
            quic_conn,
            proxy_uri.target_addr,
            server_params.mtu,
            Arc::clone(&udp_socket),
            send_tx,
        ));
        let mut proxy_tx_task = task::spawn(proxy_tx_task(udp_socket, client_rx));

        select! {
            _ = &mut connection_task => {}
            _ = &mut proxy_rx_task   => {}
            _ = &mut proxy_tx_task   => {}
        }

        connection_task.abort();
        proxy_rx_task.abort();
        proxy_tx_task.abort();

        // TODO: stream.finish()?
    }
}

/// Forward packets from `send_rx` to `connection`, and from `connection` to `client_tx`.
async fn connection_task(
    stream_id: StreamId,
    mut connection: Connection<h3_quinn::Connection, Bytes>,
    mut send_rx: mpsc::Receiver<Bytes>,
    client_tx: mpsc::Sender<Datagram>,
) -> anyhow::Result<()> {
    loop {
        tokio::select! {
            outgoing_packet = send_rx.recv() => {
                let Some(outgoing_packet) = outgoing_packet else {
                    break; // sender is gone
                };

                // TODO: is this blocking?
                connection.send_datagram(stream_id, outgoing_packet)
                    .context("Error sending QUIC datagram to client")?;
            }
            incoming_packet = connection.read_datagram() => match incoming_packet {
                Ok(Some(received_packet)) => {
                    ensure!(
                        received_packet.stream_id() == stream_id,
                        "Received unexpected stream ID from client",
                    );

                    if client_tx.send(received_packet).await.is_err() {
                        break; // receiver is gone
                    }
                }
                Ok(None) => break, // EOF
                Err(err) => {
                    return Err(err).context("Error reading QUIC datagram from client");
                }
            },
        }
    }

    Ok(())
}

/// Reassemble and forward packet fragments from `client_rx` to `udp_socket`.
async fn proxy_tx_task(udp_socket: impl AsRef<UdpSocket>, mut client_rx: mpsc::Receiver<Datagram>) {
    let udp_socket = udp_socket.as_ref();
    let mut fragments = Fragments::default();
    loop {
        let Some(quic_datagram) = client_rx.recv().await else {
            break;
        };

        let quic_payload = quic_datagram.into_payload();

        let packet = match fragments.handle_incoming_packet(quic_payload) {
            Ok(DefragReceived::Reassembled(packet) | DefragReceived::Nonfragmented(packet)) => {
                packet
            }
            Ok(DefragReceived::Fragment) => continue,
            Err(err) => {
                log::trace!("Failed to reassemble incoming packet: {err}");
                continue;
            }
        };

        if let Err(err) = udp_socket.send(&packet).await {
            log::trace!("Failed to forward packet to UDP socket {err}");
        }
    }
}

/// Forward packets from `udp_socket` to `send_tx`, and fragment them if they exceed
/// `maximum_packet_size`.
async fn proxy_rx_task(
    stream_id: StreamId,
    quinn_conn: quinn::Connection,
    target_addr: SocketAddr,
    mtu: u16,
    udp_socket: impl AsRef<UdpSocket>,
    send_tx: mpsc::Sender<Bytes>,
) {
    const TOTAL_BUFFER_CAPACITY: usize = 100 * crate::MAX_UDP_SIZE;

    let stream_id_size = VarInt::from(stream_id).size() as u16;
    let udp_socket = udp_socket.as_ref();
    let mut proxy_recv_buf = BytesMut::with_capacity(TOTAL_BUFFER_CAPACITY);
    let mut fragment_id = 0u16;

    loop {
        if !proxy_recv_buf.try_reclaim(crate::MAX_UDP_SIZE) {
            // Allocate space for new packets
            proxy_recv_buf.reserve(TOTAL_BUFFER_CAPACITY);
        }
        crate::HTTP_MASQUE_DATAGRAM_CONTEXT_ID.encode(&mut proxy_recv_buf);

        let (_n, sender_addr) = match udp_socket.recv_buf_from(&mut proxy_recv_buf).await {
            Ok(recv) => recv,
            Err(err) => {
                log::error!("Failed to receive packet from proxy socket: {err}");
                continue;
            }
        };

        if sender_addr != target_addr {
            continue;
        }

        let mut received_packet = proxy_recv_buf.split().freeze();

        let max_udp_payload_size = compute_udp_payload_size(mtu, target_addr);

        // Maximum QUIC payload (including fragmentation headers)
        let maximum_packet_size = if let Some(max_datagram_size) = quinn_conn.max_datagram_size() {
            max_datagram_size as u16 - stream_id_size
        } else {
            max_udp_payload_size - QUIC_HEADER_SIZE - stream_id_size
        };

        if received_packet.len() < usize::from(maximum_packet_size) {
            if send_tx.send(received_packet).await.is_err() {
                break;
            };
        } else {
            // TODO: consider fragmenting packets on a different task

            let _ = VarInt::decode(&mut received_packet);
            let Ok(fragments) =
                fragment::fragment_packet(maximum_packet_size, &mut received_packet, fragment_id)
            else {
                continue;
            };
            fragment_id = fragment_id.wrapping_add(1);
            for payload in fragments {
                if send_tx.send(payload).await.is_err() {
                    break;
                }
            }
        };
    }
}

async fn handle_established_connection<T: BidiStream<Bytes>>(
    stream: &mut RequestStream<T, Bytes>,
) -> Result<()> {
    let response = http::Response::builder()
        .status(StatusCode::OK)
        .body(())
        .unwrap();
    stream
        .send_response(response)
        .await
        .map_err(Error::SendNegotiationResponse)?;
    Ok(())
}

async fn handle_invalid_auth<T: BidiStream<Bytes>>(mut stream: RequestStream<T, Bytes>) {
    let response = http::Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(())
        .unwrap();
    let _ = stream.send_response(response).await;
}

async fn handle_disallowed_ip<T: BidiStream<Bytes>>(mut stream: RequestStream<T, Bytes>) {
    let response = http::Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(())
        .unwrap();
    let _ = stream.send_response(response).await;
}

async fn handle_failed_socket<T: BidiStream<Bytes>>(mut stream: RequestStream<T, Bytes>) {
    let response = http::Response::builder()
        .status(StatusCode::BAD_GATEWAY)
        .body(())
        .unwrap();
    let _ = stream.send_response(response).await;
}

async fn respond_with_redirect<T: BidiStream<Bytes>>(
    mut stream: RequestStream<T, Bytes>,
    valid_uri: ProxyUri,
) {
    let uri = Uri::from(valid_uri).to_string();
    let response = http::Response::builder()
        .status(StatusCode::PERMANENT_REDIRECT)
        .header("Location", uri)
        .body(())
        .unwrap();
    let _ = stream.send_response(response).await;
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProxyUri {
    hostname: String,
    target_addr: SocketAddr,
}

impl From<ProxyUri> for Uri {
    fn from(proxy_uri: ProxyUri) -> Self {
        Uri::builder()
            .scheme("https")
            .authority(proxy_uri.hostname)
            .path_and_query(format!(
                "{MASQUE_WELL_KNOWN_PATH}/{ip}/{port}",
                ip = proxy_uri.target_addr.ip(),
                port = proxy_uri.target_addr.port(),
            ))
            .build()
            .unwrap()
    }
}

impl TryFrom<&Uri> for ProxyUri {
    type Error = anyhow::Error;

    fn try_from(uri: &Uri) -> std::result::Result<Self, Self::Error> {
        let host = uri.host().context("Expected a URI containing a host")?;

        let path = uri.path();
        let anyhow_path_err =
            || anyhow!("Expected `/.well-known/masque/udp/<ip>/<port>`, found `{path}`");
        let (addr_str, port_str) = path
            .strip_prefix(MASQUE_WELL_KNOWN_PATH)
            .with_context(anyhow_path_err)?
            .trim_start_matches('/')
            .split_once('/')
            .with_context(anyhow_path_err)?;

        let port_str = port_str.trim_end_matches('/');

        Ok(ProxyUri {
            hostname: host.to_string(),
            target_addr: SocketAddr::new(
                addr_str.parse().with_context(anyhow_path_err)?,
                port_str.parse().with_context(anyhow_path_err)?,
            ),
        })
    }
}

impl FromStr for ProxyUri {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        ProxyUri::try_from(&Uri::from_str(s)?)
    }
}

fn unspecified_addr(addr: IpAddr) -> IpAddr {
    match addr {
        IpAddr::V4(_) => Ipv4Addr::UNSPECIFIED.into(),
        IpAddr::V6(_) => Ipv6Addr::UNSPECIFIED.into(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_good_slashy_ocketaddr() {
        let addr: IpAddr = "192.168.1.1".parse().unwrap();
        let port: u16 = 7979;
        let expected = ProxyUri {
            hostname: "foo".to_string(),
            target_addr: SocketAddr::new(addr, port),
        };
        let good_path = format!("https://foo{MASQUE_WELL_KNOWN_PATH}///{addr}/{port}////");

        assert_eq!(ProxyUri::from_str(&good_path).unwrap(), expected)
    }

    #[test]
    fn test_get_bad_socketaddr() {
        let addr: IpAddr = "192.168.1.1".parse().unwrap();
        let port: u16 = 7979;
        let bad_path = format!("{MASQUE_WELL_KNOWN_PATH}{addr}adsfasd/asdfasdf/{port}");

        assert!(ProxyUri::from_str(&bad_path).is_err())
    }
}
