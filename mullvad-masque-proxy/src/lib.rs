use bytes::{Bytes, BytesMut};
use h3::{proto::varint::VarInt, quic::StreamId};
use std::{future::Future, io, net::SocketAddr, sync::Arc};
use tokio::task::JoinSet;

pub mod client;
pub mod fragment;
pub mod server;
mod stats;

pub const MASQUE_WELL_KNOWN_PATH: &str = "/.well-known/masque/udp/";

pub const HTTP_MASQUE_DATAGRAM_CONTEXT_ID: VarInt = VarInt::from_u32(0);
pub const HTTP_MASQUE_FRAGMENTED_DATAGRAM_CONTEXT_ID: VarInt = VarInt::from_u32(1);

/// Maximum possible buffer size UDP packets, plus context ID.
// 1 byte for size of HTTP_MASQUE_DATAGRAM_CONTEXT_ID
const MAX_UDP_SIZE: usize = (u16::MAX - UDP_HEADER_SIZE + 1) as usize;

/// Maximum number of inflight packets, in both directions.
pub const MAX_INFLIGHT_PACKETS: usize = 100;

/// Fragment headers size for fragmented packets
pub const FRAGMENT_HEADER_SIZE_FRAGMENTED: u16 = 5;

/// UDP header overhead
const UDP_HEADER_SIZE: u16 = 8;

/// QUIC header size. This is conservative, real overhead varies
const QUIC_HEADER_SIZE: u16 = 41;

/// The minimum allowed `max_udp_payload_size`-value allowed by the QUIC spec.
const MIN_MAX_UDP_PAYLOAD_SIZE: u16 = 1200;

/// This is the size of the payload that stores QUIC packets
/// MTU - IP header - UDP header
///
/// Note that [quinn::EndpointConfig] accepts a minimum value of 1200.
const fn compute_udp_payload_size(mtu: u16, target_addr: SocketAddr) -> u16 {
    let ip_overhead = if target_addr.is_ipv4() { 20 } else { 40 };
    let desired_max = mtu - ip_overhead - UDP_HEADER_SIZE;

    if desired_max < MIN_MAX_UDP_PAYLOAD_SIZE {
        MIN_MAX_UDP_PAYLOAD_SIZE
    } else {
        desired_max
    }
}

/// Minimum allowed MTU (IPv4)
///
/// QUIC defines that clients must support UDP payloads of at least 1200 bytes.
/// <https://datatracker.ietf.org/doc/html/rfc9000#section-8.1>
// 20 = IPv4 header (without optional fields)
pub const MIN_IPV4_MTU: u16 = 20 + UDP_HEADER_SIZE + 1200;

/// Minimum allowed MTU (IPv6)
///
/// QUIC defines that clients must support UDP payloads of at least 1200 bytes.
/// <https://datatracker.ietf.org/doc/html/rfc9000#section-8.1>
// 40 = IPv6 header
pub const MIN_IPV6_MTU: u16 = 40 + UDP_HEADER_SIZE + 1200;

/// Task stopped gracefully because the client or server was shutting down.
#[derive(Debug)]
pub(crate) struct Stopped;

/// Error from a joined task.
#[derive(Debug, thiserror::Error)]
pub enum ProxyTaskError {
    #[error("Error sending QUIC datagram")]
    SendDatagram(#[source] h3::Error),
    #[error("Error reading QUIC datagram")]
    ReadDatagram(#[source] h3::Error),
    #[error("Failed to read from UDP socket")]
    UdpRead(#[source] io::Error),
    #[error("Failed to write to UDP socket")]
    UdpWrite(#[source] io::Error),
    #[error("Unexpected stream ID")]
    UnexpectedStreamId,
    #[error("Packet too large to fragment")]
    PacketTooLarge(#[from] fragment::PacketTooLarge),
}

pub(crate) type TaskResult = Result<Stopped, ProxyTaskError>;

/// Error from a joined task.
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("Task returned error")]
    Task(#[source] ProxyTaskError),
    #[error("Task panicked")]
    Panicked(#[source] anyhow::Error),
}

/// A set of tasks that logs task creation and aborts its remaining tasks when one finishes.
pub(crate) struct Tasks(JoinSet<TaskResult>);

impl Default for Tasks {
    fn default() -> Self {
        Self(JoinSet::new())
    }
}

impl Tasks {
    #[track_caller]
    pub(crate) fn spawn_task(&mut self, future: impl Future<Output = TaskResult> + Send + 'static) {
        let location = std::panic::Location::caller();
        self.0.spawn(async move {
            log::trace!("Task spawned at {location}");
            future.await
        });
    }

    /// Wait for all tasks to finish, returning the first error, if any.
    ///
    /// After the first task completes, the rest are aborted, so this returns
    /// once all tasks have stopped.
    pub(crate) async fn join_and_abort(mut self) -> Result<(), TaskError> {
        let result = self.0.join_next().await.expect("JoinSet is not empty");
        let mut results = vec![result];

        self.0.abort_all();

        while let Some(result) = self.0.join_next().await {
            results.push(result);
        }

        results.into_iter().try_for_each(|result| match result {
            Err(join_err) if join_err.is_panic() => Err(TaskError::Panicked(anyhow::anyhow!(
                "{:?}",
                join_err.into_panic()
            ))),
            Ok(inner) => match inner {
                Ok(Stopped) => Ok(()),
                Err(e) => Err(TaskError::Task(e)),
            },
            Err(_cancelled) => Ok(()),
        })
    }
}

/// Buffers outgoing packets and fragments them to fit the QUIC datagram size limit.
pub(crate) struct DatagramFragmentor {
    quinn_conn: quinn::Connection,
    stream_id_size: u16,
    max_udp_payload_size: u16,
    stats: Arc<stats::Stats>,
    fragment_id: u16,
    read_buf: BytesMut,
    fragments_buf: Vec<Bytes>,
}

impl DatagramFragmentor {
    const TOTAL_BUFFER_CAPACITY: usize = 100 * MAX_UDP_SIZE;

    pub(crate) fn new(
        quinn_conn: quinn::Connection,
        stream_id: StreamId,
        max_udp_payload_size: u16,
        stats: Arc<stats::Stats>,
    ) -> Self {
        Self {
            quinn_conn,
            stream_id_size: VarInt::from(stream_id).size() as u16,
            max_udp_payload_size,
            stats,
            fragment_id: 0,
            read_buf: BytesMut::with_capacity(Self::TOTAL_BUFFER_CAPACITY),
            fragments_buf: Vec::new(),
        }
    }

    /// Get a writable buffer with the MASQUE context ID already encoded.
    pub(crate) fn get_read_buf(&mut self) -> &mut BytesMut {
        if !self.read_buf.try_reclaim(MAX_UDP_SIZE) {
            self.read_buf.reserve(Self::TOTAL_BUFFER_CAPACITY);
        }
        HTTP_MASQUE_DATAGRAM_CONTEXT_ID.encode(&mut self.read_buf);
        &mut self.read_buf
    }

    /// Discard the contents of the read buffer.
    pub(crate) fn discard(&mut self) {
        self.read_buf.clear();
    }

    /// Split the read buffer into datagrams, fragmenting it if necessary.
    pub(crate) fn fragment(
        &mut self,
    ) -> Result<impl Iterator<Item = Bytes> + '_, fragment::PacketTooLarge> {
        let packet = self.read_buf.split().freeze();
        let maximum_packet_size =
            if let Some(max_datagram_size) = self.quinn_conn.max_datagram_size() {
                max_datagram_size as u16 - self.stream_id_size
            } else {
                self.max_udp_payload_size - QUIC_HEADER_SIZE - self.stream_id_size
            };

        self.fragments_buf.clear();
        if packet.len() <= usize::from(maximum_packet_size) {
            self.stats.tx(packet.len(), false);
            self.fragments_buf.push(packet);
        } else {
            let mut stripped = packet;
            let _ = VarInt::decode(&mut stripped);
            for fragment in
                fragment::fragment_packet(maximum_packet_size, &stripped, self.fragment_id)?
            {
                debug_assert!(fragment.len() <= maximum_packet_size as usize);
                self.stats.tx(fragment.len(), true);
                self.fragments_buf.push(fragment);
            }
            self.fragment_id = self.fragment_id.wrapping_add(1);
        }
        Ok(self.fragments_buf.drain(..))
    }
}
