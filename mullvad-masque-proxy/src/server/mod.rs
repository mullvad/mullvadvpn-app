use std::{
    collections::HashSet,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::Path,
    sync::Arc,
};

use bytes::{Bytes, BytesMut};
use h3::{
    proto::varint::VarInt,
    quic::BidiStream,
    server::{self, Connection, RequestStream},
};
use h3_datagram::datagram_traits::HandleDatagramsExt;
use http::{Request, StatusCode};
use quinn::{crypto::rustls::QuicServerConfig, Endpoint, Incoming};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio::net::UdpSocket;

#[derive(Debug)]
pub enum Error {
    BadTlsConfig(quinn::crypto::rustls::NoInitialCipherSuite),
    BindSocket(io::Error),
    SendNegotiationResponse(h3::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

const MASQUE_WELL_KNOWN_PATH: &str = "/.well-known/masque/udp/";

pub struct Server {
    endpoint: Endpoint,
    allowed_hosts: AllowedIps,
}

#[derive(Clone)]
struct AllowedIps {
    hosts: Arc<HashSet<IpAddr>>,
}

impl AllowedIps {
    fn ip_allowed(&self, ip: IpAddr) -> bool {
        self.hosts.is_empty() || self.hosts.contains(&ip)
    }
}

impl Server {
    pub fn bind(
        bind_addr: SocketAddr,
        allowed_hosts: HashSet<IpAddr>,
        tls_config: Arc<rustls::ServerConfig>,
    ) -> Result<Self> {
        let server_config = quinn::ServerConfig::with_crypto(Arc::new(
            QuicServerConfig::try_from(tls_config).map_err(Error::BadTlsConfig)?,
        ));

        let endpoint = Endpoint::server(server_config, bind_addr).map_err(Error::BindSocket)?;

        Ok(Self {
            endpoint,
            allowed_hosts: AllowedIps {
                hosts: Arc::new(allowed_hosts),
            },
        })
    }

    pub async fn run(self) -> Result<()> {
        while let Some(new_connection) = self.endpoint.accept().await {
            tokio::spawn(Self::handle_incoming_connection(
                new_connection,
                self.allowed_hosts.clone(),
            ));
        }
        Ok(())
    }

    async fn handle_incoming_connection(connection: Incoming, allowed_hosts: AllowedIps) {
        match connection.await {
            Ok(conn) => {
                println!("new connection established");

                let Ok(mut connection) = server::builder()
                    .enable_datagram(true)
                    .build(h3_quinn::Connection::new(conn))
                    .await
                else {
                    println!("Failed to construct a new H3 server connection");
                    return;
                };

                match connection.accept().await {
                    Ok(Some((req, stream))) => {
                        tokio::spawn(Self::handle_request(
                            connection,
                            req,
                            stream,
                            allowed_hosts.clone(),
                        ));
                    }

                    // indicating no more streams to be received
                    Ok(None) => {}

                    Err(err) => {
                        println!("error on accept {}", err);
                        return;
                    }
                }
            }
            Err(err) => {
                println!("accepting connection failed: {:?}", err);
            }
        }
    }

    async fn handle_request<T: BidiStream<Bytes>>(
        mut connection: Connection<h3_quinn::Connection, Bytes>,
        request: Request<()>,
        mut stream: RequestStream<T, Bytes>,
        allowed_hosts: AllowedIps,
    ) {
        let Some(target_addr) = get_target_socketaddr(request.uri().path()) else {
            return;
        };
        if !allowed_hosts.ip_allowed(target_addr.ip()) {
            return handle_disallowed_ip(stream).await;
        }

        let bind_addr = SocketAddr::new(unspecified_addr(target_addr.ip()), 0);
        let Ok(udp_socket) = UdpSocket::bind(bind_addr).await else {
            return handle_failed_socket(stream).await;
        };
        if let Err(err) = udp_socket.connect(target_addr).await {
            println!("Failed to set destination for UDP socket: {err}");
            return handle_failed_socket(stream).await;
        };

        if handle_established_connection(&mut stream).await.is_err() {
            return;
        }

        let stream_id = stream.id();
        let mut proxy_recv_buf = BytesMut::with_capacity(crate::PACKET_BUFFER_SIZE);

        // this is the variable ID used to signify UDP payloads in HTTP datagrams.
        let context_id: VarInt = h3::quic::StreamId::try_from(0)
            .expect("need to be able to create stream IDs with 0, no?")
            .into();

        context_id.encode(&mut proxy_recv_buf);
        loop {
            tokio::select! {
                client_send = connection.read_datagram() => {
                    match client_send {
                            Ok(Some(received_packet)) => {
                                if received_packet.stream_id() != stream_id {
                                    // log::trace!("Received unexpected stream ID from server");
                                    continue;
                                }
                                let mut payload = received_packet.into_payload();
                                let received_stream_id = VarInt::decode(&mut payload);

                                if received_stream_id  != Ok(context_id) {
                                    // probably an unsupported type of payload
                                    continue;
                                }
                                let _ = udp_socket.send_to(&payload, target_addr).await;
                            },
                            Ok(None) => {
                                return;
                            }
                            Err(_err)  => {
                                // client connection QUIC connection failed, should return now.
                                return;
                            },
                    }
                },
                recv_result = udp_socket.recv_buf_from(&mut proxy_recv_buf) => {
                    match recv_result {
                        Ok((_bytes_received, sender_addr)) => {
                            if sender_addr != target_addr {
                                continue
                            }
                            let send_buf = proxy_recv_buf.split();

                            let send_buf = send_buf.freeze();
                            if connection.send_datagram(stream_id, send_buf).is_err() {
                                return;
                            }

                            proxy_recv_buf.reserve(crate::PACKET_BUFFER_SIZE);
                            context_id.encode(&mut proxy_recv_buf);
                        },
                        Err(err) => {
                            println!("Failed to read from proxy target: {err}");
                            let _ = stream.finish().await;
                            return;
                        }
                    }
                },
            };
        }
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

fn get_target_socketaddr(request_path: &str) -> Option<SocketAddr> {
    // Establish if the URL path looks like `/.well-known/masque/udp/{ip}/{port}`
    if !request_path.starts_with(MASQUE_WELL_KNOWN_PATH) {
        return None;
    };
    let (addr_str, port_str) = request_path
        .strip_prefix(MASQUE_WELL_KNOWN_PATH)?
        .trim_start_matches('/')
        .split_once('/')?;
    let port_str = port_str.trim_end_matches('/');

    Some(SocketAddr::new(
        addr_str.trim_start_matches('/').parse().ok()?,
        port_str.parse().ok()?,
    ))
}

fn unspecified_addr(addr: IpAddr) -> IpAddr {
    match addr {
        IpAddr::V4(_) => Ipv4Addr::UNSPECIFIED.into(),
        IpAddr::V6(_) => Ipv6Addr::UNSPECIFIED.into(),
    }
}

#[test]
fn test_get_good_slashy_ocketaddr() {
    let addr: IpAddr = "192.168.1.1".parse().unwrap();
    let port: u16 = 7979;
    let expected_addr = SocketAddr::new(addr, port);
    let good_path = format!("{MASQUE_WELL_KNOWN_PATH}///{addr}/{port}////");

    assert_eq!(get_target_socketaddr(&good_path).unwrap(), expected_addr)
}

#[test]
fn test_get_bad_socketaddr() {
    let addr: IpAddr = "192.168.1.1".parse().unwrap();
    let port: u16 = 7979;
    let good_path = format!("{MASQUE_WELL_KNOWN_PATH}{addr}adsfasd/asdfasdf/{port}");

    assert_eq!(get_target_socketaddr(&good_path), None)
}
