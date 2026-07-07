//! A TCP stream backed by a smoltcp TCP socket, surfaced to async consumers as
//! [`AsyncRead`]/[`AsyncWrite`].

use bytes::{Buf, BytesMut};
use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, ready},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::{Notify, mpsc},
};
use tokio_util::sync::PollSender;

/// A TCP stream backed by a smoltcp TCP socket.
///
/// Implements [`AsyncRead`] and [`AsyncWrite`]
pub struct SmoltcpTcpStream {
    /// Channel on which data from upstream TCP peer is delivered
    pub(super) upstream_rx: mpsc::Receiver<io::Result<BytesMut>>,
    /// Channel used to deliver data to upstream TCP peer
    upstream_tx: PollSender<Vec<u8>>,
    /// Notifies smoltcp's _device_ about new writes to [[`upstream_tx]]
    notify: Arc<Notify>,
    /// Buffer received bytes
    read_buf: BytesMut,
}

impl SmoltcpTcpStream {
    /// Create a stream over the channels wired to an active smoltcp TCP socket.
    pub(super) fn new(
        read_rx: mpsc::Receiver<io::Result<BytesMut>>,
        write_tx: mpsc::Sender<Vec<u8>>,
        notify: Arc<Notify>,
    ) -> Self {
        Self {
            upstream_rx: read_rx,
            upstream_tx: PollSender::new(write_tx),
            notify,
            read_buf: BytesMut::new(),
        }
    }
}

impl AsyncRead for SmoltcpTcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();

        // Return buffered data first
        if !this.read_buf.is_empty() {
            let n = std::cmp::min(buf.remaining(), this.read_buf.len());
            buf.put_slice(&this.read_buf.split_to(n));
            return Poll::Ready(Ok(()));
        }

        match this.upstream_rx.poll_recv(cx) {
            Poll::Ready(Some(Ok(mut data))) => {
                // Freed a slot in the read channel; wake the poll loop so it can
                // promptly refill it from smoltcp's receive buffer rather than
                // waiting for the next inbound packet or poll-delay tick.
                this.notify.notify_one();
                let n = std::cmp::min(buf.remaining(), data.len());
                buf.put_slice(&data[..n]);
                // `read_buf` is empty here (we only poll the channel once it is),
                // so retain any tail by moving the buffer rather than copying it.
                data.advance(n);
                if !data.is_empty() {
                    this.read_buf = data;
                }
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Err(e)),
            Poll::Ready(None) => Poll::Ready(Err(broken_pipe())),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for SmoltcpTcpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();

        ready!(this.upstream_tx.poll_reserve(cx)).map_err(|_| broken_pipe())?;
        let n = buf.len();
        this.upstream_tx
            .send_item(buf.to_vec())
            .map_err(|_| broken_pipe())?;
        this.notify.notify_one();
        Poll::Ready(Ok(n))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Writes are handed off to the channel synchronously in `poll_write`;
        // there is nothing buffered in the stream itself to flush.
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Dropping the sender closes the write half; the poll loop observes the
        // closed channel and closes the smoltcp socket.
        self.get_mut().upstream_tx.close();
        Poll::Ready(Ok(()))
    }
}

fn broken_pipe() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "connection closed")
}
