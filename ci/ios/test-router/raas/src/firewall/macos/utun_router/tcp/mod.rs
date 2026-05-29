//! A transparent TCP proxy for the traffic arriving on the tunnel device.
//!
//! smoltcp terminates the client's connection locally by listening on the address the client was
//! trying to reach, while a real OS socket carries the payload to that address for real. The two
//! are coupled so that neither side can outrun the other:
//!
//! ```text
//! client --> tunnel --> device rx queue --> smoltcp rx buffer --> upstream write channel --> upstream
//! client <-- tunnel <-- egress queue    <-- smoltcp tx buffer <-- upstream read channel  <-- upstream
//! ```
//!
//! Every stage is bounded, and a stage that cannot accept more leaves the bytes in the stage
//! before it rather than dropping them. Backed up far enough, the TCP window on the stalled side
//! closes and the sender stops, which is the whole point of terminating the connection here.

use std::{net::SocketAddr, sync::Arc, time::Duration};

use bytes::Bytes;
use smoltcp::{
    time::Instant as SmoltcpInstant,
    wire::{Ipv4Packet, TcpPacket},
};
use tokio::sync::{Notify, mpsc};

mod device;
mod poll_loop;
mod stack;
mod upstream_socket;

/// Packets buffered between the tunnel reader and the TCP stack.
const DOWNSTREAM_PACKET_CHANNEL_CAPACITY: usize = 256;
/// Upstream control events buffered for the stack. Payloads travel per connection instead, so this
/// only has to absorb connection setup and teardown.
const UPSTREAM_EVENT_CHANNEL_CAPACITY: usize = 64;
/// Payload chunks buffered per connection, in each direction.
const TCP_DATA_CHANNEL_CAPACITY: usize = 4;
/// Largest payload moved between a socket and the stack in one go.
const UPSTREAM_READ_CHUNK: usize = 16 * 1024;
/// Packets buffered for smoltcp to process on its next poll.
const DEVICE_RX_QUEUE_DEPTH: usize = 512;
/// Packets held per connection while its upstream connection is still being established.
const PENDING_PACKET_QUEUE_DEPTH: usize = 4;
/// Size of the TCP socket receive and send buffers within smoltcp.
const TCP_BUFFER_SIZE: usize = 65535;

/// How long to wait for an upstream connection before giving up on it.
///
/// The client's SYN goes unanswered until this resolves, so it has to expire well inside the
/// client's own connect timeout. Otherwise the client gives up before we can tell it, with a RST,
/// that the destination is unreachable, and it retries instead of failing over.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Seconds between keep-alive probes sent to an idle client.
const KEEP_ALIVE_INTERVAL: u64 = 60;
/// Seconds without a response from a client before its connection is aborted.
const IDLE_TIMEOUT: u64 = 600;
/// How long to wait before retrying ingress that smoltcp could not accept.
const BLOCKED_INGRESS_RETRY: Duration = Duration::from_millis(5);

/// One proxied connection, identified by the endpoints the client used.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct TcpConnectionId {
    downstream: SocketAddr,
    upstream: SocketAddr,
}

/// Start the TCP router.
///
/// Returns the channel to feed it the client's TCP packets on, and the notify the tunnel writer
/// must signal after draining `egress_tx`: segments smoltcp could not hand over while that channel
/// was full are only retried once the stack is woken.
pub fn spawn(
    egress_tx: mpsc::Sender<Bytes>,
    mtu: u16,
) -> (mpsc::Sender<Ipv4Packet<Bytes>>, Arc<Notify>) {
    let (packet_tx, packet_rx) = mpsc::channel(DOWNSTREAM_PACKET_CHANNEL_CAPACITY);
    let (event_tx, event_rx) = mpsc::channel(UPSTREAM_EVENT_CHANNEL_CAPACITY);
    let notify = Arc::new(Notify::new());

    let stack = stack::SmoltcpStack::new(
        mtu,
        egress_tx,
        event_tx,
        notify.clone(),
        SmoltcpInstant::from_millis(0),
    );

    tokio::spawn(poll_loop::run(stack, packet_rx, event_rx, notify.clone()));

    (packet_tx, notify)
}

fn downstream_packet_identifier(packet: &Ipv4Packet<Bytes>) -> Option<TcpConnectionId> {
    let tcp_packet = tcp_packet_from_ip_packet(packet)?;

    Some(TcpConnectionId {
        downstream: SocketAddr::new(packet.src_addr().into(), tcp_packet.src_port()),
        upstream: SocketAddr::new(packet.dst_addr().into(), tcp_packet.dst_port()),
    })
}

/// Whether the packet opens a connection rather than continuing one.
fn tcp_packet_is_syn(packet: &Ipv4Packet<Bytes>) -> bool {
    tcp_packet_from_ip_packet(packet).is_some_and(|tcp| tcp.syn() && !tcp.ack())
}

/// Borrow the TCP segment carried by an IP packet.
fn tcp_packet_from_ip_packet(packet: &Ipv4Packet<Bytes>) -> Option<TcpPacket<&[u8]>> {
    let view = Ipv4Packet::new_checked(packet.as_ref()).ok()?;
    TcpPacket::new_checked(view.payload()).ok()
}
