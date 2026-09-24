#![expect(unused)]

use crate::tunnel_adapter::BoundUdpTransports;
use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{
        UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams,
        socket::{UdpSocket, UdpSocketFactory},
    },
};
use std::{
    future::poll_fn,
    io,
    net::SocketAddr,
    num::NonZero,
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    task::{Context, Poll},
};
use tokio::io::ReadBuf;

/// An abstraction over udp sockets that allows for using multiple IP addresses (and sockets) for the
/// same relay.
#[derive(Clone)]
pub struct Multiplex {
    send_index: Arc<AtomicU64>,
    multiplexers: Vec<(SocketAddr, SocketMultiplexer)>,
}
impl Multiplex {
    pub async fn new(socket_count: NonZero<u8>, destinations: Vec<SocketAddr>) -> io::Result<Self> {
        let mut pairs = Vec::new();

        for target in destinations {
            let mut sockets = Vec::new();
            for _ in 0..socket_count.get() {
                let trans = BoundUdpTransports::bind().await?;
                sockets.push(trans.socket.lock().await.clone());
            }
            pairs.push((
                target,
                SocketMultiplexer {
                    sockets,
                    recv: 0,
                    send: Arc::new(AtomicUsize::new(0)),
                },
            ));
        }

        Ok(Self {
            send_index: Default::default(),
            multiplexers: pairs,
        })
    }

    /// Rebind existing socket. It is expected that the associated GotaTun device will be suspended
    /// whilst the socket is rebound.
    pub async fn rebind(&mut self) -> io::Result<()> {
        for (_, state) in self.multiplexers.iter_mut() {
            for socket in &mut state.sockets {
                let (new_socket, _recv) = UdpSocketFactory::default()
                    .bind(&BoundUdpTransports::params())
                    .await?;
                *socket = new_socket;
            }
        }
        Ok(())
    }

    fn poll_recv(
        &mut self,
        cx: &mut Context<'_>,
        read_buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<SocketAddr>> {
        for (_, state) in self.multiplexers.iter_mut() {
            if let poll @ Poll::Ready(..) = state.poll_recv(cx, read_buf) {
                return poll;
            }
        }
        Poll::Pending
    }
}
impl UdpTransportFactory for Multiplex {
    type Send = Self;

    type Recv = Self;

    async fn bind(
        &mut self,
        _params: &UdpTransportFactoryParams,
    ) -> io::Result<(Self::Send, Self::Recv)> {
        Ok((self.clone(), self.clone()))
    }
}
impl UdpSend for Multiplex {
    type SendManyBuf = <UdpSocket as UdpSend>::SendManyBuf;

    async fn send_to(&self, packet: Packet, _destination: SocketAddr) -> io::Result<()> {
        let index =
            self.send_index.fetch_add(1, Ordering::SeqCst) as usize % self.multiplexers.len();
        let (target, state) = &self.multiplexers[index];
        state.send_to(packet, *target).await
    }
}
impl UdpRecv for Multiplex {
    type RecvManyBuf = <UdpSocket as UdpRecv>::RecvManyBuf;

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        let mut packet = pool.get();
        let mut read_buf = ReadBuf::new(&mut packet);
        match poll_fn(|cx| self.poll_recv(cx, &mut read_buf)).await {
            Ok(addr) => {
                let len = read_buf.filled().len();
                packet.truncate(len);
                Ok((packet, addr))
            }
            Err(e) => Err(e),
        }
    }
}

/// An abstraction over UDP sockets allowing to use multiple sockets to send to/recv from a destination.
#[derive(Clone)]
struct SocketMultiplexer {
    sockets: Vec<UdpSocket>,
    recv: usize,
    send: Arc<AtomicUsize>,
}
impl UdpSend for SocketMultiplexer {
    type SendManyBuf = <UdpSocket as UdpSend>::SendManyBuf;

    async fn send_to(&self, packet: Packet, destination: SocketAddr) -> io::Result<()> {
        let index = self.send.fetch_add(1, Ordering::SeqCst) % self.sockets.len();
        self.sockets[index].send_to(packet, destination).await
    }
}
impl SocketMultiplexer {
    fn poll_recv(
        &mut self,
        cx: &mut Context<'_>,
        read_buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<SocketAddr>> {
        let mut i = self.recv;
        let poll = loop {
            let socket = self.sockets[i].socket();

            if let poll @ Poll::Ready(..) = socket.poll_recv_from(cx, read_buf) {
                break Some(poll);
            }
            i = (i + 1) % self.sockets.len();
            if i == self.recv {
                break None;
            }
        };
        self.recv = (self.recv + 1) % self.sockets.len();
        if let Some(p) = poll { p } else { Poll::Pending }
    }
}
impl UdpRecv for SocketMultiplexer {
    type RecvManyBuf = <UdpSocket as UdpSend>::SendManyBuf;

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        let mut packet = pool.get();
        let mut read_buf = ReadBuf::new(&mut packet);
        match poll_fn(|cx| self.poll_recv(cx, &mut read_buf)).await {
            Ok(addr) => {
                let len = read_buf.filled().len();
                packet.truncate(len);
                Ok((packet, addr))
            }
            Err(e) => Err(e),
        }
    }
}
