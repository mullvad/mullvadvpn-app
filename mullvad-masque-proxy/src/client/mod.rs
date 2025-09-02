use anyhow::{Context, anyhow};
use bytes::{Buf, Bytes, BytesMut};
use rustls::client::danger::ServerCertVerified;
use std::{
    fs::{self},
    future, io,
    net::{Ipv4Addr, SocketAddr},
    path::Path,
    str::FromStr as _,
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::{
    net::UdpSocket,
    select,
    sync::{broadcast, mpsc},
};
use typed_builder::TypedBuilder;

use h3::{client, ext::Protocol, proto::varint::VarInt, quic::StreamId};
use h3_datagram::{datagram::Datagram, datagram_traits::HandleDatagramsExt};
use http::{StatusCode, Uri, header, uri::Scheme};
use quinn::{
    Endpoint, EndpointConfig, IdleTimeout, TokioRuntime, TransportConfig,
    crypto::rustls::QuicClientConfig,
};
use socket2::Socket;

use crate::{
    MASQUE_WELL_KNOWN_PATH, MAX_INFLIGHT_PACKETS, MIN_IPV4_MTU, MIN_IPV6_MTU, QUIC_HEADER_SIZE,
    compute_udp_payload_size,
    fragment::{self, DefragReceived, Fragments},
    stats::Stats,
};

const MAX_HEADER_SIZE: u64 = 8192;

const MAX_REDIRECT_COUNT: usize = 1;

const LE_ROOT_CERT: &[u8] = include_bytes!("../../../mullvad-api/le_root_cert.pem");

pub struct Client {
    client_socket: Arc<UdpSocket>,

    /// QUIC endpoint
    quinn_conn: quinn::Connection,

    /// QUIC connection, used to send the actual HTTP datagrams
    connection: h3::client::Connection<h3_quinn::Connection, bytes::Bytes>,

    /// Send stream over a QUIC connection - this needs to be kept alive to not close the HTTP
    /// QUIC stream.
    _send_stream: client::SendRequest<h3_quinn::OpenStreams, bytes::Bytes>,

    /// Request stream for the currently open request, must not be dropped, otherwise proxy
    /// connection is terminated
    request_stream: client::RequestStream<h3_quinn::BidiStream<bytes::Bytes>, bytes::Bytes>,

    /// Maximum UDP payload size (packet size including QUIC overhead)
    max_udp_payload_size: u16,

    stats: Arc<Stats>,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to bind local socket")]
    Bind(#[source] io::Error),
    #[error("Failed to setup a QUIC endpoint")]
    Endpoint(#[source] io::Error),
    #[error("Failed to begin connecting to QUIC endpoint")]
    Connect(#[from] quinn::ConnectError),
    #[error("Failed to connect to QUIC endpoint")]
    Connection(#[from] quinn::ConnectionError),
    #[error("Invalid MTU: must be at least {min_mtu}")]
    InvalidMtu { min_mtu: u16 },
    #[error("Invalid max_udp_payload_size")]
    InvalidMaxUdpPayload(#[source] quinn::ConfigError),
    #[error("Connection closed while sending request to initiate proxying")]
    ConnectionClosedPrematurely,
    #[error("QUIC connection failed while sending request to initiate proxying")]
    ConnectionFailed(#[source] h3::Error),
    #[error("Request failed to illicit a response.")]
    RequestError(#[source] h3::Error),
    #[error("Received response was not a 200: {}", .0)]
    UnexpectedStatus(http::StatusCode),
    #[error("Failed to receive data from client socket")]
    ClientRead(#[source] io::Error),
    #[error("Failed to send data to client socket")]
    ClientWrite(#[source] io::Error),
    #[error("Failed to receive data from server socket")]
    ServerRead(#[source] h3::Error),
    #[error("Failed to create a client")]
    CreateClient(#[source] h3::Error),
    #[error("Failed to receive good response from proxy")]
    ProxyResponse(#[source] h3::Error),
    #[error("Failed to construct a URI")]
    Uri(#[source] http::Error),
    #[error("Failed to send datagram to proxy")]
    SendDatagram(#[source] h3::Error),
    #[error("Failed to read certificates")]
    ReadCerts(#[source] io::Error),
    #[error("Failed to parse certificates")]
    ParseCerts,
    #[error("Failed to fragment a packet - it is too large")]
    PacketTooLarge(#[from] fragment::PacketTooLarge),
    #[error("The provided idle timeout was invalid")]
    InvalidIdleTimeout(quinn::VarIntBoundsExceeded),
    #[error("The server returned an invalid HTTP redirect")]
    InvalidHttpRedirect(#[source] anyhow::Error),
}

#[derive(TypedBuilder, Debug)]
pub struct ClientConfig {
    /// Socket that accepts proxy clients
    pub client_socket: UdpSocket,

    /// Socket to bind the QUIC endpoint socket to
    ///
    /// May be created by [Self::create_masque_socket].
    pub quinn_socket: std::net::UdpSocket,

    /// Destination to which traffic is forwarded
    pub target_addr: SocketAddr,

    /// Remote QUIC endpoint address
    pub server_addr: SocketAddr,

    /// Remote QUIC endpoint hostname
    pub server_host: String,

    /// MTU (includes IP header)
    #[builder(default = 1500)]
    pub mtu: u16,

    /// QUIC TLS config
    #[builder(default = default_tls_config())]
    pub tls_config: Arc<rustls::ClientConfig>,

    /// Optional timeout when no data is sent in the proxy.
    #[builder(default)]
    pub idle_timeout: Option<Duration>,

    /// Set the authorization header to use in the CONNECT-UDP request.
    #[builder(default)]
    pub auth_header: Option<String>,
}

impl Client {
    pub async fn connect(config: ClientConfig) -> Result<Self> {
        let quic_client_config = QuicClientConfig::try_from(config.tls_config)
            .expect("Failed to construct a valid TLS configuration");

        let mut client_config = quinn::ClientConfig::new(Arc::new(quic_client_config));
        let mut transport_config = TransportConfig::default();
        transport_config.max_idle_timeout(
            config
                .idle_timeout
                .map(IdleTimeout::try_from)
                .transpose()
                .map_err(Error::InvalidIdleTimeout)?,
        );

        // TODO: Set datagram_receive_buffer_size  if needed
        // TODO: Set datagram_send_buffer_size if needed
        // When would it be needed? If we need to buffer more packets or buffer less packets for
        // better performance.
        client_config.transport_config(Arc::new(transport_config));

        Self::validate_mtu(config.mtu, config.target_addr)?;

        let max_udp_payload_size = compute_udp_payload_size(config.mtu, config.server_addr);

        let endpoint = Self::setup_quic_endpoint(config.quinn_socket, max_udp_payload_size)?;

        let connecting =
            endpoint.connect_with(client_config, config.server_addr, &config.server_host)?;

        let connection = connecting.await?;

        let (h3_connection, send_stream, request_stream) = Self::setup_h3_connection(
            connection.clone(),
            config.target_addr,
            &config.server_host,
            max_udp_payload_size,
            config.auth_header,
        )
        .await?;

        Ok(Self {
            quinn_conn: connection,
            connection: h3_connection,
            client_socket: Arc::new(config.client_socket),
            request_stream,
            _send_stream: send_stream,
            max_udp_payload_size,
            stats: Arc::default(),
        })
    }

    const fn validate_mtu(mtu: u16, target_addr: SocketAddr) -> Result<()> {
        let min_mtu = if target_addr.is_ipv4() {
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

    /// Create & bind a new UDP socket to `local_addr`.
    pub fn create_masque_socket(local_addr: SocketAddr) -> io::Result<Socket> {
        // family
        let domain = match &local_addr {
            SocketAddr::V4(_) => socket2::Domain::IPV4,
            SocketAddr::V6(_) => socket2::Domain::IPV6,
        };
        let ty = socket2::Type::DGRAM;
        let protocol = Some(socket2::Protocol::UDP);
        let socket = socket2::Socket::new(domain, ty, protocol)?;
        socket.bind(&socket2::SockAddr::from(local_addr))?;
        Ok(socket)
    }

    // `socket` is a UDP socket which quinn will read/write from/to.
    fn setup_quic_endpoint(
        socket: std::net::UdpSocket,
        max_udp_payload_size: u16,
    ) -> Result<Endpoint> {
        let endpoint_config = {
            let mut endpoint_config = EndpointConfig::default();
            endpoint_config
                .max_udp_payload_size(max_udp_payload_size)
                .map_err(Error::InvalidMaxUdpPayload)?;
            endpoint_config
        };

        Endpoint::new(endpoint_config, None, socket, Arc::new(TokioRuntime))
            .map_err(Error::Endpoint)
    }

    /// Returns an h3 connection that is ready to be used for sending UDP datagrams.
    async fn setup_h3_connection(
        connection: quinn::Connection,
        target: SocketAddr,
        server_host: &str,
        mtu: u16,
        auth_header: Option<String>,
    ) -> Result<(
        client::Connection<h3_quinn::Connection, bytes::Bytes>,
        client::SendRequest<h3_quinn::OpenStreams, bytes::Bytes>,
        client::RequestStream<h3_quinn::BidiStream<bytes::Bytes>, bytes::Bytes>,
    )> {
        let (connection, send_stream) = client::builder()
            .max_field_section_size(MAX_HEADER_SIZE)
            .enable_datagram(true)
            .send_grease(true)
            .build(h3_quinn::Connection::new(connection))
            .await
            .map_err(Error::CreateClient)?;

        Self::send_connect_request(
            connection,
            send_stream,
            server_host,
            target,
            mtu,
            0,
            auth_header,
        )
        .await
    }

    /// Send an HTTP CONNECT request and set up the h3 connection for sending datagrams.
    ///
    /// This function will follow HTTP redirects up to [MAX_REDIRECT_COUNT].
    async fn send_connect_request(
        mut connection: client::Connection<h3_quinn::Connection, bytes::Bytes>,
        mut send_stream: client::SendRequest<h3_quinn::OpenStreams, bytes::Bytes>,
        server_host: &str,
        target: SocketAddr,
        mtu: u16,
        redirect_count: usize,
        auth_header: Option<String>,
    ) -> Result<(
        client::Connection<h3_quinn::Connection, bytes::Bytes>,
        client::SendRequest<h3_quinn::OpenStreams, bytes::Bytes>,
        client::RequestStream<h3_quinn::BidiStream<bytes::Bytes>, bytes::Bytes>,
    )> {
        let request = new_connect_request(target, &server_host, mtu, auth_header.as_deref())?;

        let request_future = async move {
            let mut request_stream = send_stream.send_request(request).await?;
            let response = request_stream.recv_response().await?;
            Ok((response, send_stream, request_stream))
        };

        let response = tokio::select! {
            response = request_future => response,
            // TODO: this arm completes first when the connection is gracefully terminated from the
            // peer, but ideally we want to be able to handle the response above in that case.
            closed = future::poll_fn(|cx| connection.poll_close(cx)) => {
                return match closed {
                    Ok(()) => Err(Error::ConnectionClosedPrematurely),
                    Err(err) => Err(Error::ConnectionFailed(err)),
                };
            },
        };

        let (response, send_stream, request_stream) = response.map_err(Error::RequestError)?;

        match response.status() {
            StatusCode::OK => Ok((connection, send_stream, request_stream)),

            // If we are trying to connect with the wrong `host` in the HTTP URI, then the masque
            // server will redirect us to the URI with the correct `host`.
            status @ StatusCode::PERMANENT_REDIRECT => {
                if redirect_count >= MAX_REDIRECT_COUNT {
                    log::error!("Too many redirects (redirect loop?)");
                    return Err(anyhow!("Too many redirects")).map_err(Error::InvalidHttpRedirect);
                }

                let server_host = response
                    .headers()
                    .get("Location")
                    .and_then(|header| header.to_str().ok())
                    .and_then(|location| Uri::from_str(location).ok())
                    .inspect(|location| log::info!("Redirected to {location:?} (HTTP {status})"))
                    .and_then(|location| location.host().map(String::from))
                    .context("Failed to decode `Location` HTTP header")
                    .map_err(Error::InvalidHttpRedirect)?;

                // Repeat the request, but using the new host
                //
                // We are re-using the same h3 connection for this HTTP request, meaning that we
                // will never redirect to a *different* server. We are only re-issuing the same
                // HTTP request, using the same connection, but with a different URI.
                Box::pin(Self::send_connect_request(
                    connection,
                    send_stream,
                    &server_host,
                    target,
                    mtu,
                    redirect_count + 1,
                    auth_header,
                ))
                .await
            }

            status => Err(Error::UnexpectedStatus(status)),
        }
    }

    pub async fn run(self) -> Result<()> {
        let stream_id: StreamId = self.request_stream.id();

        let (client_tx, client_rx) = mpsc::channel(MAX_INFLIGHT_PACKETS);
        let (server_tx, server_rx) = mpsc::channel(MAX_INFLIGHT_PACKETS);
        let (return_addr_tx, return_addr_rx) = broadcast::channel(1);

        let mut client_socket_rx_task = tokio::task::spawn(client_socket_rx_task(
            self.client_socket.clone(),
            client_tx,
            return_addr_tx,
        ));

        let mut client_socket_tx_task = tokio::task::spawn(client_socket_tx_task(
            stream_id,
            server_rx,
            return_addr_rx,
            self.client_socket.clone(),
            Arc::clone(&self.stats),
        ));

        let mut server_socket_task = tokio::task::spawn(server_socket_task(
            stream_id,
            self.max_udp_payload_size,
            self.quinn_conn,
            self.connection,
            server_tx,
            client_rx,
            Arc::clone(&self.stats),
        ));

        let result = select! {
            result = &mut client_socket_tx_task => result,
            result = &mut client_socket_rx_task => result,
            result = &mut server_socket_task => result,
        };

        client_socket_tx_task.abort();
        client_socket_rx_task.abort();
        server_socket_task.abort();

        result.expect("proxy routine panicked")
    }
}

async fn server_socket_task(
    stream_id: StreamId,
    max_udp_payload_size: u16,
    quinn_conn: quinn::Connection,
    mut connection: h3::client::Connection<h3_quinn::Connection, bytes::Bytes>,
    server_tx: mpsc::Sender<Datagram>,
    mut client_rx: mpsc::Receiver<Bytes>,
    stats: Arc<Stats>,
) -> Result<()> {
    let mut fragment_id = 0u16;
    let stream_id_size = VarInt::from(stream_id).size() as u16;

    loop {
        let packet = select! {
            datagram = connection.read_datagram() => {
                match datagram {
                    Ok(Some(response)) => {
                        if server_tx.send(response).await.is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(err) => return Err(Error::ProxyResponse(err)),
                }

                continue;
            }
            packet = client_rx.recv() => packet,
        };

        let Some(mut packet) = packet else { break };

        // Maximum QUIC payload (including fragmentation headers)
        let maximum_packet_size = if let Some(max_datagram_size) = quinn_conn.max_datagram_size() {
            max_datagram_size as u16 - stream_id_size
        } else {
            max_udp_payload_size - QUIC_HEADER_SIZE - stream_id_size
        };

        if packet.len() <= usize::from(maximum_packet_size) {
            stats.tx(packet.len(), false);
            connection
                .send_datagram(stream_id, packet)
                .map_err(Error::SendDatagram)?;
        } else {
            // drop the added context ID, since packet will have to be fragmented.
            let _ = VarInt::decode(&mut packet);

            for fragment in fragment::fragment_packet(maximum_packet_size, &mut packet, fragment_id)
                .map_err(Error::PacketTooLarge)?
            {
                debug_assert!(fragment.len() <= maximum_packet_size as usize);

                stats.tx(fragment.len(), true);
                connection
                    .send_datagram(stream_id, fragment)
                    .map_err(Error::SendDatagram)?;
            }
            fragment_id = fragment_id.wrapping_add(1);
        }
    }

    Result::Ok(())
}

async fn client_socket_rx_task(
    client_socket: Arc<UdpSocket>,
    client_tx: mpsc::Sender<Bytes>,
    return_addr_tx: broadcast::Sender<SocketAddr>,
) -> Result<()> {
    let mut client_read_buf = BytesMut::with_capacity(100 * crate::PACKET_BUFFER_SIZE);
    let mut return_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0);

    loop {
        client_read_buf.reserve(crate::PACKET_BUFFER_SIZE);

        // this is the variable ID used to signify UDP payloads in HTTP datagrams.
        crate::HTTP_MASQUE_DATAGRAM_CONTEXT_ID.encode(&mut client_read_buf);

        let (_bytes_received, recv_addr) = client_socket
            .recv_buf_from(&mut client_read_buf)
            .await
            .map_err(Error::ClientRead)?;

        if recv_addr != return_addr {
            return_addr = recv_addr;
            if return_addr_tx.send(return_addr).is_err() {
                break;
            }
        }
        let packet = client_read_buf.split().freeze();

        if client_tx.send(packet).await.is_err() {
            break;
        };
    }

    Ok(())
}

async fn client_socket_tx_task(
    stream_id: StreamId,
    mut server_rx: mpsc::Receiver<Datagram>,
    mut return_addr_rx: broadcast::Receiver<SocketAddr>,
    client_socket: Arc<UdpSocket>,
    stats: Arc<Stats>,
) -> Result<()> {
    let mut fragments = Fragments::default();

    let mut return_addr = loop {
        match return_addr_rx.recv().await {
            Ok(addr) => break addr,
            Err(broadcast::error::RecvError::Lagged(..)) => continue,
            Err(broadcast::error::RecvError::Closed) => return Ok(()),
        }
    };

    loop {
        let Some(response) = server_rx.recv().await else {
            break;
        };

        match return_addr_rx.try_recv() {
            Ok(new_addr) => return_addr = new_addr,
            Err(broadcast::error::TryRecvError::Empty) => {}
            Err(..) => break,
        }

        if response.stream_id() != stream_id {
            log::debug!("Received datagram with an unexpected stream ID");
            continue;
        }
        let payload = response.into_payload();
        let original_payload_len = payload.len();

        let send = async |payload: &[u8]| -> Result<()> {
            client_socket
                .send_to(payload, return_addr)
                .await
                .map_err(Error::ClientWrite)?;
            Ok(())
        };

        match fragments.handle_incoming_packet(payload) {
            Ok(DefragReceived::Nonfragmented(payload)) => {
                stats.rx(payload.len(), false);
                send(payload.chunk()).await?;
            }
            Ok(DefragReceived::Reassembled(reassembled_payload)) => {
                stats.rx(original_payload_len, true);
                send(reassembled_payload.chunk()).await?;
            }
            Ok(DefragReceived::Fragment) => stats.rx(original_payload_len, true),
            Err(e) => {
                use log::Level::*;
                let level = if cfg!(debug_assertions) { Debug } else { Trace };
                log::log!(level, "Packet reassembly failed: {e}");
            }
        }
    }

    Result::Ok(())
}

fn new_connect_request(
    socket_addr: SocketAddr,
    authority: &dyn AsRef<str>,
    mtu: u16,
    authorization: Option<&str>,
) -> Result<http::Request<()>> {
    let host = socket_addr.ip();
    let port = socket_addr.port();
    let path = format!("{MASQUE_WELL_KNOWN_PATH}{host}/{port}/");
    let uri = http::uri::Builder::new()
        .scheme(Scheme::HTTPS)
        .authority(authority.as_ref())
        .path_and_query(&path)
        .build()
        .map_err(Error::Uri)?;

    let mut builder = http::Request::builder()
        .method(http::method::Method::CONNECT)
        .uri(uri)
        .header(b"Capsule-Protocol".as_slice(), b"?1".as_slice())
        .header(header::HOST, authority.as_ref());

    if let Some(auth) = authorization {
        builder = builder.header(header::AUTHORIZATION, auth);
    }

    let mut request = builder
        // TODO: Not needed since we set the max_udp_payload_size transport param
        .header(
            b"X-Mullvad-Uplink-Mtu".as_slice(),
            format!("{mtu}"),
        )
        .body(())
        .expect("failed to construct a body");

    request.extensions_mut().insert(Protocol::CONNECT_UDP);
    Ok(request)
}

pub fn default_tls_config() -> Arc<rustls::ClientConfig> {
    static TLS_CONFIG: LazyLock<Arc<rustls::ClientConfig>> =
        LazyLock::new(|| client_tls_config_with_certs(read_cert_store()));

    TLS_CONFIG.clone()
}

fn client_tls_config_with_certs(certs: rustls::RootCertStore) -> Arc<rustls::ClientConfig> {
    let mut config = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_protocol_versions(&[&rustls::version::TLS13])
    .expect("ring crypt-prover should support TLS 1.3")
    .with_root_certificates(certs)
    .with_no_client_auth();
    config.alpn_protocols = vec![b"h3".to_vec()];

    let approver = Approver {};
    config
        .dangerous()
        .set_certificate_verifier(Arc::new(approver));
    Arc::new(config)
}

fn read_cert_store() -> rustls::RootCertStore {
    read_cert_store_from_reader(&mut std::io::BufReader::new(LE_ROOT_CERT))
        .expect("failed to read built-in cert store")
}

pub fn client_tls_config_from_cert_path(path: &Path) -> Result<Arc<rustls::ClientConfig>> {
    let certs = read_cert_store_from_path(path)?;
    Ok(client_tls_config_with_certs(certs))
}

fn read_cert_store_from_path(path: &Path) -> Result<rustls::RootCertStore> {
    let cert_path = fs::File::open(path).map_err(Error::ReadCerts)?;
    read_cert_store_from_reader(&mut std::io::BufReader::new(cert_path))
}

fn read_cert_store_from_reader(reader: &mut dyn io::BufRead) -> Result<rustls::RootCertStore> {
    let mut cert_store = rustls::RootCertStore::empty();

    let certs = rustls_pemfile::certs(reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(Error::ReadCerts)?;
    let (num_certs_added, num_failures) = cert_store.add_parsable_certificates(certs);
    if num_failures > 0 || num_certs_added == 0 {
        return Err(Error::ParseCerts);
    }

    Ok(cert_store)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_zero_stream_id() {
        h3::quic::StreamId::try_from(0).expect("need to be able to create stream IDs with 0, no?");
    }
}

#[derive(Debug)]
struct Approver {}

impl rustls::client::danger::ServerCertVerifier for Approver {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}
