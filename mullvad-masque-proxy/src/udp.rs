use std::{io::Write, net::SocketAddr, sync::{atomic::AtomicUsize, Arc, }, time::Duration};

use bytes::{Buf, BufMut, BytesMut};
use tokio::{io, net::UdpSocket, sync::Notify};


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

pub fn fake_udp_pair() -> (FakeUdp, FakeUdp) {
    let inner1: Arc<tokio::sync::Mutex<FakeInner>> = Arc::default();
    let inner2: Arc<tokio::sync::Mutex<FakeInner>> = Arc::default();
    let udp1 = FakeUdp {
        me: inner1.clone(),
        other: inner2.clone(),
    };
    let udp2 = FakeUdp {
        me: inner2,
        other: inner1,
    };
    (udp1, udp2)
}

pub struct FakeUdp {
    me: Arc<tokio::sync::Mutex<FakeInner>>,
    other: Arc<tokio::sync::Mutex<FakeInner>>,
}

struct FakeInner {
    data: BytesMut,
    // FIXME
    addr: SocketAddr,
    signal: Arc<Notify>,
}

impl Default for FakeInner {
    fn default() -> Self {
        Self {
            data: BytesMut::with_capacity(8 * 1024 * 1024),
            addr: "0.0.0.0:0".parse().unwrap(),
            signal: Arc::default(),
        }
    }
}

impl UdpSocketTrait for FakeUdp {
    async fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        let mut other = self.other.lock().await;
        let n_write = other.data.remaining_mut().min(buf.len());
        
        other.data.put_slice(&buf[..n_write]);
        other.addr = addr;
        
        other.signal.notify_one();

        Ok(n_write)
    }

    async fn recv_buf_from(&self, buf: &mut BytesMut) -> io::Result<(usize, SocketAddr)> {
        loop {
            let mut me_ = self.me.lock().await;
            if me_.data.len() != 0 {
                let n_read = me_.data.len().min(buf.remaining_mut());
        
                buf.put(me_.data.split_to(n_read));
        
                return Ok((n_read, me_.addr));
            }
            let signal = me_.signal.clone();
            drop(me_);

            signal.notified().await;
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_fake_udp() {
        const SENDMSG: &[u8] = b"hello there!";
        let expected_addr: SocketAddr = "1.2.3.4:1234".parse().unwrap();

        let (udp1, udp2) = fake_udp_pair();

        udp1.send_to(SENDMSG, expected_addr).await.unwrap();

        let mut buf = BytesMut::new();
        let (nbytes, addr) = udp2.recv_buf_from(&mut buf).await.unwrap();

        assert_eq!(nbytes, SENDMSG.len());
        assert_eq!(buf.chunk(), SENDMSG);
        assert_eq!(addr, expected_addr);

        udp2.send_to(SENDMSG, expected_addr).await.unwrap();

        let mut buf = BytesMut::new();
        let (nbytes, addr) = udp1.recv_buf_from(&mut buf).await.unwrap();

        assert_eq!(nbytes, SENDMSG.len());
        assert_eq!(buf.chunk(), SENDMSG);
        assert_eq!(addr, expected_addr);
    }
}
