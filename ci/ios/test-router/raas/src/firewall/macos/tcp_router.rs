use std::{collections::VecDeque, sync::Arc};

use bytes::BytesMut;
use smoltcp::{
    iface::{Config, Interface, SocketSet},
    phy::{self, Device, DeviceCapabilities, Medium},
    time::Instant as SmoltcpInstant,
};
use tokio::sync::mpsc;
use tun_rs::AsyncDevice;

struct SmoltcpStack {
    device: SmoltcpDevice,
    interface: Interface,
    sockets: SocketSet<'static>,
}

impl SmoltcpStack {
    pub fn new(
        rx_receiver: mpsc::Receiver<Ipv4Packet<Bytes>>,
        tunnel: Arc<AsyncDevice>,
        mtu: u16,
        now: Instant,
    ) -> Self {
        let interface_config = Config::new(smoltcp::wire::HardwareAddress::Ip);

        let mut device = SmoltcpDevice {
            tunnel,
            receive_buffer: VecDeque::new(),
            mtu,
        };

        let interface = Interface::new(interface_config, &mut device, now);

        Self {
            device,
            sockets: SocketSet::new(Vec::new()),
            interface,
        }
    }

    async fn run(self) {

    }
}

struct SmoltcpDevice {
    tunnel: Arc<AsyncDevice>,
    receive_buffer: VecDeque<Vec<u8>>,
    mtu: u16,
}

impl SmoltcpDevice {
    fn enqueue_packet(&mut self, packet: Vec<u8>) {
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

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let rx = self
            .receive_buffer
            .pop_front()
            .map(|packet| SmoltcpRxToken { packet });
        let tx = self.tx_token();
        rx.map(|rx| (rx, tx))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
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
    packet: Vec<u8>,
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
