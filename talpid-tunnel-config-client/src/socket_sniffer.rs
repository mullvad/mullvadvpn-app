use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// A wrapper around a socket that upon drop logs the total bytes sent and received.
pub struct SocketSniffer<S> {
    socket: S,
    rx_bytes: u64,
    tx_bytes: u64,
    start_time: std::time::Instant,
}

impl<S> SocketSniffer<S> {
    /// Create a new socket sniffer wrapping the provided socket.
    pub fn new(socket: S) -> Self {
        Self {
            socket,
            rx_bytes: 0,
            tx_bytes: 0,
            start_time: std::time::Instant::now(),
        }
    }
}

impl<S> Drop for SocketSniffer<S> {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        log::debug!(
            "Tunnel config client connection ended. RX: {} bytes, TX: {} bytes, duration: {} s",
            self.rx_bytes,
            self.tx_bytes,
            duration.as_secs()
        );
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncRead for SocketSniffer<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let initial_data = buf.filled().len();
        let bytes = std::task::ready!(Pin::new(&mut self.socket).poll_read(cx, buf));
        if bytes.is_ok() {
            let read_bytes = buf.filled().len().saturating_sub(initial_data);
            self.rx_bytes += u64::try_from(read_bytes).unwrap();
        }
        Poll::Ready(bytes)
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncWrite for SocketSniffer<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let bytes = std::task::ready!(Pin::new(&mut self.socket).poll_write(cx, buf));
        if let Ok(bytes) = bytes {
            self.tx_bytes += u64::try_from(bytes).unwrap();
        }
        Poll::Ready(bytes)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.socket).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.socket).poll_shutdown(cx)
    }
}
