use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    mem,
    net::SocketAddr,
    sync::Arc,
};

use bytes::{Bytes, BytesMut};
use smoltcp::{
    iface::{Config, Interface, SocketHandle, SocketSet},
    phy::{self, Device, DeviceCapabilities, Medium},
    socket::tcp::{self, ListenError},
    time::Instant as SmoltcpInstant,
    wire::{Ipv4Packet, TcpPacket},
};
use tokio::sync::mpsc;
use tun_rs::AsyncDevice;
use upstream_socket::{UpstreamSocketCommand, UpstreamSocketHandle, UpstreamSocketMsg};

mod upstream_socket;

/// Size of TCP socket receive and send buffers within smoltcp.
const TCP_BUFFER_SIZE: usize = 65535;

pub struct SmoltcpStack {
    device: SmoltcpDevice,
    interface: Interface,
    sockets: SocketSet<'static>,
    upstream_sockets: BTreeMap<TcpConnectionId, UpstreamSocketHandle>,
    serviceable_sockets: BTreeSet<TcpConnectionId>,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
struct TcpConnectionId {
    downstream: SocketAddr,
    upstream: SocketAddr,
}

impl SmoltcpStack {
    pub fn new(tunnel: Arc<AsyncDevice>, mtu: u16) -> Self {
        let interface_config = Config::new(smoltcp::wire::HardwareAddress::Ip);

        let mut device = SmoltcpDevice {
            tunnel,
            receive_buffer: VecDeque::new(),
            mtu,
        };

        let mut interface = Interface::new(interface_config, &mut device, SmoltcpInstant::now());

        interface.set_any_ip(true);
        interface
            .routes_mut()
            .add_default_ipv4_route(smoltcp::wire::Ipv4Address::new(0, 0, 0, 1))
            .expect("enough space for default IPv4 route");
        interface
            .routes_mut()
            .add_default_ipv6_route(smoltcp::wire::Ipv6Address::new(0, 0, 0, 0, 0, 0, 0, 1))
            .expect("enough space for default IPv6 route");

        Self {
            device,
            sockets: SocketSet::new(Vec::new()),
            interface,
            upstream_sockets: Default::default(),
            serviceable_sockets: Default::default(),
        }
    }

    pub async fn run(
        mut packet_rx: mpsc::Receiver<Ipv4Packet<Bytes>>,
        tunnel: Arc<AsyncDevice>,
        mtu: u16,
    ) {
        let mut stack = Self::new(tunnel, mtu);
        let (new_socket_tx, mut new_socket_rx) = mpsc::channel::<UpstreamSocketMsg>(1024);

        // This future must multiplex between:
        //  - raw traffic from tunnel device that is expected to be TCP packets:
        //      if packet matches an existing upstream connection (TcpConnectionId), process it by
        //      passing it on
        //      if packet doesn't match, initiate a new connection
        //
        //  - events from upstream connections:
        //      - successful connection: upstream socket already exists, but smoltcp socket may
        //          not exist yet in this state, so we must create a smoltcp socket to receive this
        //          traffic if needed and then drain all buffered packets
        //      - failed connections: remove socket from upstream_sockets, destroy smoltcp socket
        //          it this was the only associated socket
        //      - received payloads from upstream: push upstream payload into smoltcp socket
        //      - successfully sent payloads to upstream: push packet into smoltcp stack, drain
        //          smoltcp socket
        //  - timer for smoltcp to wake up to drive timer driven traffic (retries)
        //
        //  Any interactions with the smoltcp stack within the smoltcp should only enqueue packets
        //  to be drained later. Same goes for smoltcp sockets.
        tokio::select! {
            socket_msg = new_socket_rx.recv() => {
                if let Some(msg) = socket_msg {
                    stack.handle_upstream_socket_cmd(msg);
                }
            },
            downstream_packet = packet_rx.recv() => {
                if let Some(packet) = downstream_packet {
                    stack.handle_incoming_downstream_packet(packet, new_socket_tx.clone());
                } else {
                    log::debug!("Upstream writer closed");
                    return;
                }
            }
        }

        // TODO
        // What's missing here:
        //  - push buffered upstream payeloads into smoltcp sockets (achieved by pump_tcp_writes in
        //  poll_loop.rs)
        //  - drain smoltcp buffers into tunnel device, achieved by poll() in poll_loop.rs
        //  - if we abstract away the async device with a trait (AsyncWrite + AsyncRead), we
        //  can make this very testable.
        //      * listen on a localhost TCP socket
        //      * supply dummy tun device
        //      * push in valid TCP syn packet destined to a localhost listener
        //      * validate that a TCP connection is made to the localhost socket
        //      * push in valid synack packet
        //      * push TCP payload into localhost socket
        //      * validate that dummy tun device receives a TCP payload packet
        //
        //  - debug everything
        //
        // See mullvad-ios/src/gotatun/smoltcp_network/poll_loop.rs for inspiration
        //
    }

    fn handle_upstream_socket_cmd(&mut self, socket_cmd: UpstreamSocketMsg) {
        let Some(connection) = self.upstream_sockets.get_mut(&socket_cmd.id) else {
            log::debug!(
                "Received upstream socket message about a dead connection {:?}",
                socket_cmd.id
            );
            return;
        };

        match socket_cmd.cmd {
            UpstreamSocketCommand::Connected => {
                connection.is_connected = true;
                self.serviceable_sockets.insert(socket_cmd.id);
                match Self::open_local_listener(socket_cmd.id.upstream) {
                    Ok(socket) => {
                        let handle = self.sockets.add(socket);
                        connection.smoltcp_socket_handle = Some(handle);
                    }
                    Err(err) => {
                        log::error!("Failed to open a listening socket: {err}");
                        connection.recv_task.abort();

                        self.upstream_sockets.remove(&socket_cmd.id);
                        self.serviceable_sockets.remove(&socket_cmd.id);
                    }
                }
            }
            UpstreamSocketCommand::Failed => {
                connection.recv_task.abort();
                self.serviceable_sockets.remove(&socket_cmd.id);
                if let Some(handle) = connection.smoltcp_socket_handle.take() {
                    self.sockets.remove(handle);
                }

                self.upstream_sockets.remove(&socket_cmd.id);
            }
            UpstreamSocketCommand::ReceivedPayload(payload) => {
                connection.upstream_buffer.push_back(payload);
                self.serviceable_sockets.insert(socket_cmd.id);
            }
            UpstreamSocketCommand::SentPayload(packet) => {
                self.device.enqueue_packet(packet.into_inner());
            }
        }
    }

    fn open_local_listener(
        listening_address: SocketAddr,
    ) -> Result<tcp::Socket<'static>, ListenError> {
        let rx_buffer = tcp::SocketBuffer::new(vec![0u8; TCP_BUFFER_SIZE]);
        let tx_buffer = tcp::SocketBuffer::new(vec![0u8; TCP_BUFFER_SIZE]);
        let mut socket = tcp::Socket::new(rx_buffer, tx_buffer);
        socket.listen(listening_address)?;

        Ok(socket)
    }

    fn handle_incoming_downstream_packet(
        &mut self,
        packet: Ipv4Packet<Bytes>,
        socket_msg_tx: mpsc::Sender<UpstreamSocketMsg>,
    ) {
        let Some(connection_id) = downstream_packet_identifier(&packet) else {
            return;
        };

        if let Some(connection) = self.upstream_sockets.get_mut(&connection_id) {
            if tcp_packet_has_payload(&packet) {
                if let Err(err) = connection.downstream_payload_tx.try_send(packet) {
                    log::error!(
                        "{connection_id:?} - failed to send packet to upstream, dropping payload: {err}"
                    );
                };
                return;
            }
            self.device.receive_buffer.push_back(packet.into_inner());
            return;
        }

        // if no connection has been registered, we should create one.
        let handle = UpstreamSocketHandle::new(connection_id, socket_msg_tx);
        self.upstream_sockets.insert(connection_id, handle);
    }
}

//
fn downstream_packet_identifier(packet: &Ipv4Packet<Bytes>) -> Option<TcpConnectionId> {
    let tcp_header = tcp_packet_from_ip_packet(packet)?;
    let source_address = SocketAddr::new(packet.src_addr().into(), tcp_header.src_port());
    let destination_address = SocketAddr::new(packet.dst_addr().into(), tcp_header.dst_port());

    Some(TcpConnectionId {
        downstream: source_address,
        upstream: destination_address,
    })
}

fn tcp_packet_has_payload(packet: &Ipv4Packet<Bytes>) -> bool {
    let Some(tcp) = tcp_packet_from_ip_packet(packet) else {
        return false;
    };
    !tcp.payload().is_empty()
}

fn tcp_packet_from_ip_packet(packet: &Ipv4Packet<Bytes>) -> Option<TcpPacket<&[u8]>> {
    let tcp_bytes = packet
        .as_ref()
        .split_off(smoltcp::wire::IPV4_HEADER_LEN..)?;
    TcpPacket::new_checked(tcp_bytes).ok()
}

struct SmoltcpDevice {
    tunnel: Arc<AsyncDevice>,
    receive_buffer: VecDeque<Bytes>,
    mtu: u16,
}

impl SmoltcpDevice {
    fn enqueue_packet(&mut self, packet: Bytes) {
        self.receive_buffer.push_back(packet);
    }

    fn tx_token<'a>(&'a self) -> SmoltcpTxToken<'a> {
        let device = &self.tunnel;
        SmoltcpTxToken { device }
    }
}

impl Device for SmoltcpDevice {
    type RxToken<'a> = SmoltcpRxToken;

    type TxToken<'a> = SmoltcpTxToken<'a>;

    fn receive(
        &mut self,
        _timestamp: SmoltcpInstant,
    ) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let rx = self
            .receive_buffer
            .pop_front()
            .map(|packet| SmoltcpRxToken { packet });
        let tx = self.tx_token();
        rx.map(|rx| (rx, tx))
    }

    fn transmit(&mut self, _timestamp: SmoltcpInstant) -> Option<Self::TxToken<'_>> {
        Some(self.tx_token())
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ip;
        caps.max_transmission_unit = self.mtu.into();
        caps
    }
}

pub struct SmoltcpRxToken {
    packet: Bytes,
}

impl phy::RxToken for SmoltcpRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.packet)
    }
}

pub struct SmoltcpTxToken<'a> {
    device: &'a Arc<AsyncDevice>,
}

impl phy::TxToken for SmoltcpTxToken<'_> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buf = BytesMut::zeroed(len);
        let result = f(&mut buf);
        let device = self.device.clone();

        tokio::spawn(async move {
            // drop return, doesn't matter if it doesn't work anymore
            let _ = device.send(&buf).await;
        });
        result
    }
}
