//! Tunnels datagrams to the remote inside a TCP stream, each one prefixed with its length.

use std::{io, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use talpid_net::bypass::{BypassGuard, BypassSocket, SocketBypass};
use tokio::{
    net::TcpSocket,
    sync::{Mutex, mpsc},
};
use tokio_util::task::AbortOnDropHandle;
use udp_over_tcp::{DatagramSocket, HEADER_LEN, process_udp_over_tcp};

use crate::transport::ObfuscatedTransport;

/// How many datagrams may be waiting in either direction between us and the forwarder.
///
/// Sending blocks once this many are queued.
const DATAGRAM_QUEUE_LEN: usize = 128;

#[derive(Debug, Clone)]
pub struct Settings {
    pub peer: SocketAddr,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to create the TCP socket
    #[error("Failed to create the TCP socket")]
    CreateTcpSocket(#[source] io::Error),

    /// Failed to connect to the remote
    #[error("Failed to connect to the remote")]
    ConnectTcp(#[source] io::Error),

    /// Failed to disable the Nagle algorithm
    #[error("Failed to disable the Nagle algorithm")]
    SetNodelay(#[source] io::Error),
}

/// Forwards datagrams over a TCP stream, using the `udp-over-tcp` protocol.
///
/// The framing is done by `udp-over-tcp`'s forwarder, which owns the TCP stream and runs as a
/// background task. It picks datagrams up from, and delivers them to, a pair of in-memory queues,
/// so [Self::send] and [Self::recv] only ever touch those queues.
pub struct Udp2Tcp {
    /// Datagrams destined for the remote. Picked up by [Queues::recv].
    outgoing: mpsc::Sender<Box<[u8]>>,

    /// Datagrams that arrived from the remote, put there by [Queues::send].
    ///
    /// The [Mutex] is uncontended in practice, since whoever drives this transport receives from
    /// a single task.
    incoming: Mutex<mpsc::Receiver<Box<[u8]>>>,

    /// Stops the forwarder when this transport is dropped.
    _forwarder: AbortOnDropHandle<()>,

    /// Keeps the TCP socket excluded from tunnel traffic.
    _bypass: BypassGuard,

    peer: SocketAddr,
}

impl Udp2Tcp {
    pub fn new(bypass: Arc<dyn SocketBypass>, settings: &Settings) -> crate::Result<Self> {
        let tcp_socket = match settings.peer {
            SocketAddr::V4(..) => TcpSocket::new_v4(),
            SocketAddr::V6(..) => TcpSocket::new_v6(),
        }
        .map_err(Error::CreateTcpSocket)
        .map_err(crate::Error::CreateUdp2TcpObfuscator)?;

        // Disables the Nagle algorithm on the TCP socket. Improves performance
        tcp_socket
            .set_nodelay(true)
            .map_err(Error::SetNodelay)
            .map_err(crate::Error::CreateUdp2TcpObfuscator)?;

        let BypassSocket {
            socket: tcp_socket,
            guard: _bypass,
        } = BypassSocket::new(bypass, tcp_socket).map_err(crate::Error::Bypass)?;

        let (outgoing, outgoing_rx) = mpsc::channel(DATAGRAM_QUEUE_LEN);
        let (incoming_tx, incoming) = mpsc::channel(DATAGRAM_QUEUE_LEN);
        let queues = Queues {
            incoming: incoming_tx,
            outgoing: Mutex::new(outgoing_rx),
        };

        let peer = settings.peer;
        let forwarder = tokio::spawn(async move {
            if let Err(err) = forward(queues, tcp_socket, peer).await {
                log::error!("The udp-over-tcp forwarder stopped: {err}");
            }
        });

        Ok(Self {
            outgoing,
            incoming: Mutex::new(incoming),
            _forwarder: AbortOnDropHandle::new(forwarder),
            _bypass,
            peer,
        })
    }
}

#[async_trait]
impl ObfuscatedTransport for Udp2Tcp {
    async fn send(&self, packet: &mut [u8]) -> io::Result<()> {
        self.outgoing
            .send(Box::from(&packet[..]))
            .await
            .map_err(|_| stopped())
    }

    async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let datagram = self
            .incoming
            .lock()
            .await
            .recv()
            .await
            .ok_or_else(stopped)?;
        copy_datagram(&datagram, buf)
    }

    fn endpoint(&self) -> SocketAddr {
        self.peer
    }

    fn packet_overhead(&self) -> u16 {
        let max_tcp_header_len = 60; // https://datatracker.ietf.org/doc/html/rfc9293#section-3.1-6.22.1
        let udp_header_len = 8; // https://datatracker.ietf.org/doc/html/rfc768

        let overhead = max_tcp_header_len - udp_header_len + HEADER_LEN;

        u16::try_from(overhead).expect("packet overhead is less than u16::MAX")
    }
}

/// Connect to `peer` and shuttle datagrams between `queues` and the stream until it closes.
///
/// Dropping `queues` on the way out closes both of them, which is what makes [Udp2Tcp::send] and
/// [Udp2Tcp::recv] fail once the connection is gone, rather than wait for a stream that nobody is
/// reading.
async fn forward(queues: Queues, tcp_socket: TcpSocket, peer: SocketAddr) -> Result<(), Error> {
    let tcp_stream = tcp_socket.connect(peer).await.map_err(Error::ConnectTcp)?;
    log::debug!("Connected to {peer}");

    process_udp_over_tcp(queues, tcp_stream, None).await;
    Ok(())
}

/// The forwarder's end of the queues that [Udp2Tcp] holds the other end of.
///
/// This stands in for the UDP socket that `udp-over-tcp` would otherwise read datagrams from and
/// write them to, which lets the datagrams stay in memory rather than take a trip through the
/// kernel.
struct Queues {
    /// Datagrams that arrived from the remote, handed to [Udp2Tcp::recv].
    incoming: mpsc::Sender<Box<[u8]>>,

    /// Datagrams destined for the remote, put there by [Udp2Tcp::send].
    outgoing: Mutex<mpsc::Receiver<Box<[u8]>>>,
}

impl DatagramSocket for Queues {
    async fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.incoming
            .send(Box::from(buf))
            .await
            .map_err(|_| stopped())?;
        Ok(buf.len())
    }

    async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let datagram = self
            .outgoing
            .lock()
            .await
            .recv()
            .await
            .ok_or_else(stopped)?;
        copy_datagram(&datagram, buf)
    }
}

fn copy_datagram(datagram: &[u8], buf: &mut [u8]) -> io::Result<usize> {
    buf.get_mut(..datagram.len())
        .ok_or_else(|| io::Error::other("datagram does not fit in the receive buffer"))?
        .copy_from_slice(datagram);
    Ok(datagram.len())
}

fn stopped() -> io::Error {
    io::Error::new(
        io::ErrorKind::BrokenPipe,
        "the udp-over-tcp forwarder stopped",
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use talpid_net::bypass::NoopBypass;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    /// Send a packet in each direction and assert that it appears on the TCP stream with the
    /// length header that the `udp-over-tcp` protocol prescribes.
    #[tokio::test]
    async fn test_length_prefixed_framing() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let peer = listener.local_addr().unwrap();

        let udp2tcp = Udp2Tcp::new(Arc::new(NoopBypass), &Settings { peer }).unwrap();
        let (mut remote, _) = listener.accept().await.unwrap();

        udp2tcp.send(&mut [1, 2, 3]).await.unwrap();

        let mut framed = [0u8; 5];
        remote.read_exact(&mut framed).await.unwrap();
        assert_eq!(framed, [0, 3, 1, 2, 3]);

        remote.write_all(&[0, 2, 9, 8]).await.unwrap();

        let mut buf = [0u8; crate::transport::MAX_DATAGRAM_SIZE];
        let n = udp2tcp.recv(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], &[9, 8]);
    }

    /// Constructing the transport must not wait for the TCP handshake. A failure to connect
    /// surfaces from `recv` instead.
    #[tokio::test]
    async fn test_connects_in_the_background() {
        // Nothing is listening on this address once the listener is dropped.
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let peer = listener.local_addr().unwrap();
        drop(listener);

        let udp2tcp = Udp2Tcp::new(Arc::new(NoopBypass), &Settings { peer }).unwrap();

        let mut buf = [0u8; crate::transport::MAX_DATAGRAM_SIZE];
        udp2tcp.recv(&mut buf).await.unwrap_err();
    }

    /// The transport must report an error, rather than wait forever, once the remote hangs up.
    #[tokio::test]
    async fn test_recv_fails_when_remote_disconnects() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let peer = listener.local_addr().unwrap();

        let udp2tcp = Udp2Tcp::new(Arc::new(NoopBypass), &Settings { peer }).unwrap();
        let (remote, _) = listener.accept().await.unwrap();
        drop(remote);

        let mut buf = [0u8; crate::transport::MAX_DATAGRAM_SIZE];
        udp2tcp.recv(&mut buf).await.unwrap_err();
    }
}
