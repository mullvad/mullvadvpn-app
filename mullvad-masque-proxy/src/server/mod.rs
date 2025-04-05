use std::{
    collections::HashSet,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use bytes::{Bytes, BytesMut};
use h3::{
    proto::varint::VarInt,
    quic::{BidiStream, StreamId},
    server::{self, Connection, RequestStream},
};
use h3_datagram::{datagram::Datagram, datagram_traits::HandleDatagramsExt};
use http::{Request, StatusCode};
use quinn::{crypto::rustls::QuicServerConfig, Endpoint, Incoming};
use tokio::{net::UdpSocket, time::interval};

use crate::fragment::{self, Fragments};

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
    max_packet_size: u16,
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
        max_packet_size: u16,
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
            max_packet_size,
        })
    }

    pub async fn run(self) -> Result<()> {
        while let Some(new_connection) = self.endpoint.accept().await {
            tokio::spawn(Self::handle_incoming_connection(
                new_connection,
                self.allowed_hosts.clone(),
                self.max_packet_size,
            ));
        }
        Ok(())
    }

    async fn handle_incoming_connection(
        connection: Incoming,
        allowed_hosts: AllowedIps,
        maximum_packet_size: u16,
    ) {
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
                        tokio::spawn(Self::handle_proxy_request(
                            connection,
                            req,
                            stream,
                            allowed_hosts.clone(),
                            maximum_packet_size,
                        ));
                    }

                    // indicating no more streams to be received
                    Ok(None) => {}

                    Err(err) => {
                        println!("error on accept {}", err);
                    }
                }
            }
            Err(err) => {
                println!("accepting connection failed: {:?}", err);
            }
        }
    }

    async fn handle_proxy_request<T: BidiStream<Bytes>>(
        mut connection: Connection<h3_quinn::Connection, Bytes>,
        request: Request<()>,
        mut stream: RequestStream<T, Bytes>,
        allowed_hosts: AllowedIps,
        maximum_packet_size: u16,
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

        let mut fragments = Fragments::default();
        let mut fragment_id = 0u16;

        let mut interval = interval(Duration::from_secs(3));
        crate::HTTP_MASQUE_DATAGRAM_CONTEXT_ID.encode(&mut proxy_recv_buf);

        loop {
            tokio::select! {
                client_send = connection.read_datagram() => {
                    match client_send {
                            Ok(Some(received_packet)) => {
                                handle_client_packet(received_packet, stream_id, &mut fragments, &udp_socket, target_addr).await;
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

                            let mut received_packet = proxy_recv_buf.split().freeze();

                            if received_packet.len() < maximum_packet_size.into() {
                                if connection.send_datagram(stream_id, received_packet).is_err() {
                                    return;
                                }
                            } else {
                                let _ = VarInt::decode(&mut received_packet);
                                let Ok(fragments) = fragment::fragment_packet(maximum_packet_size, &mut received_packet, fragment_id) else { continue; };
                                fragment_id += 1;
                                for payload in fragments {
                                    if connection.send_datagram(stream_id, payload).is_err() {
                                        return;
                                    }
                                }
                            };

                            proxy_recv_buf.reserve(crate::PACKET_BUFFER_SIZE);
                            crate::HTTP_MASQUE_DATAGRAM_CONTEXT_ID.encode(&mut proxy_recv_buf);
                        },
                        Err(err) => {
                            println!("Failed to receive packet from proxy connection: {err}");
                            let _ = stream.finish().await;
                            return;
                        }
                    }
                },
                _ = interval.tick() => {
                    fragments.clear_old_fragments(
                        Duration::from_secs(3)
                    );
                },
            };
        }
    }
}

async fn handle_client_packet(
    received_packet: Datagram,
    stream_id: StreamId,
    fragments: &mut Fragments,
    proxy_socket: &UdpSocket,
    target_addr: SocketAddr,
) {
    if received_packet.stream_id() != stream_id {
        // log::trace!("Received unexpected stream ID from server");
        return;
    }

    if let Ok(Some(payload)) = fragments.handle_incoming_packet(received_packet.into_payload()) {
        let _ = proxy_socket.send_to(&payload, target_addr).await;
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
