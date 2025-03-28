use bytes::{Buf, BytesMut};
use rustls::client::danger::ServerCertVerified;
use std::{
    fs, future, io,
    net::{Ipv4Addr, SocketAddr},
    path::Path,
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::{net::UdpSocket, time::interval};

use h3::{client, ext::Protocol, proto::varint::VarInt, quic::StreamId};
use h3_datagram::datagram_traits::HandleDatagramsExt;
use http::{header, uri::Scheme, Response, StatusCode};
use quinn::{crypto::rustls::QuicClientConfig, ClientConfig, Endpoint, TransportConfig};

use crate::fragment::{self, Fragments};

const MAX_HEADER_SIZE: u64 = 8192;

const LE_ROOT_CERT: &[u8] = include_bytes!("../../../mullvad-api/le_root_cert.pem");

pub struct Client {
    client_socket: UdpSocket,
    /// QUIC connection, used to send the actual HTTP datagrams
    connection: h3::client::Connection<h3_quinn::Connection, bytes::Bytes>,
    /// Send stream over a QUIC connection - this needs to be kept alive to not close the HTTP
    /// QUIC stream.
    _send_stream: client::SendRequest<h3_quinn::OpenStreams, bytes::Bytes>,
    /// Request stream for the currently open request, must not be dropped, otherwise proxy
    /// connection is terminated
    request_stream: client::RequestStream<h3_quinn::BidiStream<bytes::Bytes>, bytes::Bytes>,
    /// Packet fragments
    fragments: Fragments,
    /// Maximum packet size
    maximum_packet_size: u16,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Bind(io::Error),
    Connect(quinn::ConnectError),
    Connection(quinn::ConnectionError),
    /// Connection closed while sending request to initiate proxying
    ConnectionClosedPrematurely,
    /// QUIC connection failed while sending request to initiate proxying
    ConnectionFailed(h3::Error),
    /// Request failed to illicit a response.
    RequestError(h3::Error),
    /// Received response was not a 200.
    UnexpectedStatus(http::StatusCode),
    /// Failed to receive data from client socket
    ClientRead(io::Error),
    /// Failed to send data to client socket
    ClientWrite(io::Error),
    /// Failed to receive data from server socket
    ServerRead(h3::Error),
    /// Failed to create a client
    CreateClient(h3::Error),
    /// Failed to receive good response from proxy
    ProxyResponse(h3::Error),
    /// Failed to construct a URI
    Uri(http::Error),
    /// Failed to send datagram to proxy
    SendDatagram(h3::Error),
    /// Failed to read certificates
    ReadCerts(io::Error),
    /// Failed to parse certificates
    ParseCerts,
    /// Failed to fragment a packet - it is too large
    PacketTooLarge(fragment::PacketTooLarge),
}

impl Client {
    pub async fn connect(
        client_socket: UdpSocket,
        server_addr: SocketAddr,
        local_addr: SocketAddr,
        target_addr: SocketAddr,
        server_host: &str,
        maximum_packet_size: u16,
    ) -> Result<Self> {
        Self::connect_with_tls_config(
            client_socket,
            server_addr,
            local_addr,
            target_addr,
            server_host,
            default_tls_config(),
            maximum_packet_size,
        )
        .await
    }

    pub async fn connect_with_tls_config(
        client_socket: UdpSocket,
        server_addr: SocketAddr,
        local_addr: SocketAddr,
        target_addr: SocketAddr,
        server_host: &str,
        tls_config: Arc<rustls::ClientConfig>,
        maximum_packet_size: u16,
    ) -> Result<Self> {
        let quic_client_config = QuicClientConfig::try_from(tls_config)
            .expect("Failed to construct a valid TLS configuration");

        let mut client_config = ClientConfig::new(Arc::new(quic_client_config));
        let transport_config = TransportConfig::default();
        // TODO: Set datagram_receive_buffer_size  if needed
        // TODO: Set datagram_send_buffer_size if needed
        // When would it be needed? If we need to buffer more packets or buffer less packets for
        // better performance.
        client_config.transport_config(Arc::new(transport_config));
        Self::connect_with_local_addr(
            client_socket,
            server_addr,
            local_addr,
            target_addr,
            server_host,
            client_config,
            maximum_packet_size,
        )
        .await
    }

    async fn connect_with_local_addr(
        client_socket: UdpSocket,
        server_addr: SocketAddr,
        local_addr: SocketAddr,
        target_addr: SocketAddr,
        server_host: &str,
        client_config: ClientConfig,
        maximum_packet_size: u16,
    ) -> Result<Self> {
        let endpoint = Endpoint::client(local_addr).map_err(Error::Bind)?;

        let connecting = endpoint
            .connect_with(client_config, server_addr, server_host)
            .map_err(Error::Connect)?;

        let connection = connecting.await.map_err(Error::Connection)?;

        let (connection, send_stream, request_stream) =
            Self::setup_h3_connection(connection, target_addr, server_host, maximum_packet_size)
                .await?;

        Ok(Self {
            connection,
            client_socket,
            request_stream,
            fragments: Fragments::default(),
            _send_stream: send_stream,
            maximum_packet_size,
        })
    }

    // Returns an h3 connection that is ready to be used for sending UDP datagrams.
    async fn setup_h3_connection(
        connection: quinn::Connection,
        target: SocketAddr,
        server_host: &str,
        maximum_packet_size: u16,
    ) -> Result<(
        client::Connection<h3_quinn::Connection, bytes::Bytes>,
        client::SendRequest<h3_quinn::OpenStreams, bytes::Bytes>,
        client::RequestStream<h3_quinn::BidiStream<bytes::Bytes>, bytes::Bytes>,
    )> {
        let (mut connection, mut send_stream) = client::builder()
            .max_field_section_size(MAX_HEADER_SIZE)
            .enable_datagram(true)
            .send_grease(true)
            .build(h3_quinn::Connection::new(connection))
            .await
            .map_err(Error::CreateClient)?;

        let request = new_connect_request(target, &server_host, maximum_packet_size)?;

        let request_future = async move {
            let mut request_stream = send_stream.send_request(request).await?;
            let response = request_stream.recv_response().await?;
            Ok((response, send_stream, request_stream))
        };

        tokio::select! {
            closed = future::poll_fn(|cx| connection.poll_close(cx)) => {
                match closed {
                    Ok(()) => Err(Error::ConnectionClosedPrematurely),
                    Err(err) => Err(Error::ConnectionFailed(err)),
                }
            },
            response = request_future => {
                let (response, send_stream, request_stream) = response.map_err(Error::RequestError)?;
                handle_response(response)?;
                Ok((connection, send_stream, request_stream))
            },
        }
    }

    pub async fn run(mut self) -> Result<()> {
        let stream_id: StreamId = self.request_stream.id();
        // this is the variable ID used to signify UDP payloads in HTTP datagrams.
        let mut client_read_buf = BytesMut::with_capacity(crate::PACKET_BUFFER_SIZE * 1024);
        crate::HTTP_MASQUE_DATAGRAM_CONTEXT_ID.encode(&mut client_read_buf);

        let mut return_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0);
        let mut fragment_id = 1u16;
        let mut interval = interval(Duration::from_secs(3));

        loop {
            tokio::select! {
                client_read = self.client_socket.recv_buf_from(&mut client_read_buf) => {
                    let (_bytes_received, recv_addr) = client_read.map_err(Error::ClientRead)?;
                    return_addr = recv_addr;

                    let mut send_buf = client_read_buf.split().freeze();
                    if send_buf.len() < (Into::<usize>::into(self.maximum_packet_size) - 100usize) {
                        self.connection
                            .send_datagram(stream_id, send_buf)
                            .map_err(Error::SendDatagram)?;
                    } else {
                        // drop the added context ID, since packet will have to be fragmented.
                        {
                            let _ = VarInt::decode(&mut send_buf);
                        }
                        for fragment in fragment::fragment_packet(
                                self.maximum_packet_size,
                                &mut send_buf,
                                fragment_id)
                            .map_err(Error::PacketTooLarge)
                            ? {
                                self.connection.send_datagram(stream_id, fragment).map_err(Error::SendDatagram)?;
                            }
                        fragment_id = fragment_id.wrapping_add(1);
                    }

                    client_read_buf.reserve(crate::PACKET_BUFFER_SIZE);
                    crate::HTTP_MASQUE_DATAGRAM_CONTEXT_ID.encode(&mut client_read_buf);
                },
                server_response = self.connection.read_datagram() => {
                    match server_response {
                        Ok(Some(response)) => {
                            if response.stream_id() != stream_id {
                                // log::trace!("Received datagram with an unexpected stream ID");
                                continue;
                            }
                            let mut payload = response.into_payload();
                            let context = VarInt::decode(&mut payload);
                            match  context {
                                Ok(crate::HTTP_MASQUE_DATAGRAM_CONTEXT_ID) => {
                                    self.client_socket
                                        .send_to(payload.as_ref(), return_addr)
                                        .await
                                        .map_err(Error::ClientWrite)?;
                                }
                                Ok(crate::HTTP_MASQUE_FRAGMENTED_DATAGRAM_CONTEXT_ID) => {
                                    if let Ok(Some(payload)) = self.fragments.handle_incoming_packet(payload) {
                                        self.client_socket
                                            .send_to(payload.chunk(), return_addr)
                                            .await
                                            .map_err(Error::ClientWrite)?;
                                    }
                                },
                                _ => (),

                            }
                        }
                        Ok(None) => {
                            return Ok(());
                        }
                        Err(err) => {
                            return Err(Error::ProxyResponse(err));
                        }
                    }
                },
                _ = interval.tick() => {
                    self.fragments.clear_old_fragments(
                        Duration::from_secs(3)
                    );
                },
            };
        }
    }
}

fn new_connect_request(
    socket_addr: SocketAddr,
    authority: &dyn AsRef<str>,
    maximum_packet_size: u16,
) -> Result<http::Request<()>> {
    let host = socket_addr.ip();
    let port = socket_addr.port();
    let path = format!("/.well-known/masque/udp/{host}/{port}/");
    let uri = http::uri::Builder::new()
        .scheme(Scheme::HTTPS)
        .authority(authority.as_ref())
        .path_and_query(&path)
        .build()
        .map_err(Error::Uri)?;

    let mut request = http::Request::builder()
        .method(http::method::Method::CONNECT)
        .uri(uri)
        .header(b"Capsule-Protocol".as_slice(), b"?1".as_slice())
        .header(header::AUTHORIZATION, b"Bearer test".as_slice())
        .header(header::HOST, authority.as_ref())
        .header(
            b"X-Mullvad-Uplink-Mtu".as_slice(),
            format!("{maximum_packet_size}"),
        )
        .body(())
        .expect("failed to construct a body");

    request.extensions_mut().insert(Protocol::CONNECT_UDP);
    Ok(request)
}

fn handle_response(response: Response<()>) -> Result<()> {
    if response.status() != StatusCode::OK {
        return Err(Error::UnexpectedStatus(response.status()));
    }
    Ok(())
}

// TODO: resuse the same TLS code from `mullvad-api` maybe
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
    config.key_log = Arc::new(rustls::KeyLogFile::new());
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

#[test]
fn test_zero_stream_id() {
    h3::quic::StreamId::try_from(0).expect("need to be able to create stream IDs with 0, no?");
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
