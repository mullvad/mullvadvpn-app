use std::{
    collections::HashSet,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

use anyhow::{ensure, Context};
use bytes::{Bytes, BytesMut};
use h3::{
    proto::varint::VarInt,
    quic::{BidiStream, StreamId},
    server::{self, Connection, RequestStream},
};
use h3_datagram::{datagram::Datagram, datagram_traits::HandleDatagramsExt};
use http::{Request, StatusCode};
use quinn::{crypto::rustls::QuicServerConfig, Endpoint, Incoming};
use tokio::{net::UdpSocket, select, sync::mpsc, task};

use crate::{
    compute_udp_payload_size,
    fragment::{self, Fragments},
    MAX_INFLIGHT_PACKETS, MIN_IPV4_MTU, MIN_IPV6_MTU, QUIC_HEADER_SIZE,
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

const MASQUE_WELL_KNOWN_PATH: &str = "/.well-known/masque/udp/";

pub struct Server {
    endpoint: Endpoint,
    allowed_hosts: AllowedIps,
    mtu: u16,
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
        mtu: u16,
    ) -> Result<Self> {
        Self::validate_mtu(mtu, bind_addr)?;

        let server_config = quinn::ServerConfig::with_crypto(Arc::new(
            QuicServerConfig::try_from(tls_config).map_err(Error::BadTlsConfig)?,
        ));

        let endpoint = Endpoint::server(server_config, bind_addr).map_err(Error::BindSocket)?;

        Ok(Self {
            endpoint,
            allowed_hosts: AllowedIps {
                hosts: Arc::new(allowed_hosts),
            },
            mtu,
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
                self.allowed_hosts.clone(),
                self.mtu,
            ));
        }
        Ok(())
    }

    async fn handle_incoming_connection(connection: Incoming, allowed_hosts: AllowedIps, mtu: u16) {
        match connection.await {
            Ok(conn) => {
                println!("new connection established");

                let quinn_conn = conn.clone();

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
                            quinn_conn,
                            req,
                            stream,
                            allowed_hosts.clone(),
                            mtu,
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
        connection: Connection<h3_quinn::Connection, Bytes>,
        quinn_conn: quinn::Connection,
        request: Request<()>,
        mut stream: RequestStream<T, Bytes>,
        allowed_hosts: AllowedIps,
        mtu: u16,
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
        let udp_socket = Arc::new(udp_socket);
        let (client_tx, client_rx) = mpsc::channel(MAX_INFLIGHT_PACKETS);
        let (send_tx, send_rx) = mpsc::channel(MAX_INFLIGHT_PACKETS);

        let mut connection_task =
            task::spawn(connection_task(stream_id, connection, send_rx, client_tx));
        let mut proxy_rx_task = task::spawn(proxy_rx_task(
            stream_id,
            quinn_conn,
            target_addr,
            mtu,
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
            Ok(Some(packet)) => packet,
            Ok(None) => continue,
            Err(_defrag_err) => {
                // TODO: log::trace!()
                continue;
            }
        };

        if let Err(_err) = udp_socket.send(&packet).await {
            // TODO: log::trace!()
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
    let stream_id_size = VarInt::from(stream_id).size() as u16;
    let udp_socket = udp_socket.as_ref();
    let mut proxy_recv_buf = BytesMut::with_capacity(100 * crate::PACKET_BUFFER_SIZE);
    let mut fragment_id = 0u16;

    loop {
        proxy_recv_buf.reserve(crate::PACKET_BUFFER_SIZE);
        crate::HTTP_MASQUE_DATAGRAM_CONTEXT_ID.encode(&mut proxy_recv_buf);

        let (_n, sender_addr) = match udp_socket.recv_buf_from(&mut proxy_recv_buf).await {
            Ok(recv) => recv,
            Err(err) => {
                println!("Failed to receive packet from proxy socket: {err}");
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
            fragment_id += 1;
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

#[cfg(test)]
mod test {
    use super::*;

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
}
