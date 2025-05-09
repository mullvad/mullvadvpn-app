use std::{net::SocketAddr, sync::{atomic::AtomicUsize, Arc, Mutex}};

use bytes::BytesMut;
use tokio::{io, net::UdpSocket};


pub trait UdpSocketTrait: Send + Sync + 'static {
    fn send_to(
        &self,
        buf: &[u8],
        addr: SocketAddr,
    ) -> impl std::future::Future<Output = io::Result<usize>> + Send;
    fn recv_buf_from(
        &self,
        buf: &mut BytesMut,
    ) -> impl std::future::Future<Output = io::Result<(usize, SocketAddr)>> + Send;
}

impl UdpSocketTrait for UdpSocket {
    async fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        UdpSocket::send_to(self, buf, addr).await
    }

    async fn recv_buf_from(&self, buf: &mut BytesMut) -> io::Result<(usize, SocketAddr)> {
        UdpSocket::recv_buf_from(self, buf).await
    }
}

// TODO
/*
pub fn fake_udp_pair() -> (FakeUdpSender, FakeUdpReceiver) {
    todo!()
}

pub struct FakeUdpSender(FakeInner);

pub struct FakeUdpReceiver(FakeInner);

struct FakeInner {
    rx: Vec<u8>,
}

impl UdpSocketTrait for FakeUdpSender {
    async fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        UdpSocket::send_to(self, buf, addr).await
    }

    async fn recv_buf_from(&self, buf: &mut BytesMut) -> io::Result<(usize, SocketAddr)> {
        UdpSocket::recv_buf_from(self, buf).await
    }
}


impl UdpSocketTrait for FakeUdpReceiver {
    async fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        UdpSocket::send_to(self, buf, addr).await
    }

    async fn recv_buf_from(&self, buf: &mut BytesMut) -> io::Result<(usize, SocketAddr)> {
        UdpSocket::recv_buf_from(self, buf).await
    }
}
 */
