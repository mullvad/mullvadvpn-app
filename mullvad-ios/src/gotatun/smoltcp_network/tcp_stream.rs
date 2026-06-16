//! A TCP stream backed by a smoltcp TCP socket, surfaced to async consumers as
//! [`AsyncRead`]/[`AsyncWrite`].

use bytes::BytesMut;
use std::{
    future::Future,
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, ready},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::{Notify, mpsc},
};

/// A TCP stream backed by a smoltcp TCP socket.
///
/// Implements [`AsyncRead`] and [`AsyncWrite`] for use with tonic/gRPC
/// (ephemeral peer exchange) or any other async TCP consumer.
pub struct SmoltcpTcpStream {
    /// Visible to the poll loop's tests, which assert on delivered data.
    pub(super) read_rx: mpsc::Receiver<io::Result<Vec<u8>>>,
    write_tx: mpsc::Sender<Vec<u8>>,
    notify: Arc<Notify>,
    read_buf: BytesMut,
    in_flight_write: Option<Pin<Box<dyn Future<Output = io::Result<usize>> + Send>>>,
}

impl SmoltcpTcpStream {
    /// Create a stream over the channels wired to an active smoltcp TCP socket.
    pub(super) fn new(
        read_rx: mpsc::Receiver<io::Result<Vec<u8>>>,
        write_tx: mpsc::Sender<Vec<u8>>,
        notify: Arc<Notify>,
    ) -> Self {
        Self {
            read_rx,
            write_tx,
            notify,
            read_buf: BytesMut::new(),
            in_flight_write: None,
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

        match this.read_rx.poll_recv(cx) {
            Poll::Ready(Some(Ok(data))) => {
                // Freed a slot in the read channel; wake the poll loop so it can
                // promptly refill it from smoltcp's receive buffer rather than
                // waiting for the next inbound packet or poll-delay tick.
                this.notify.notify_one();
                let n = std::cmp::min(buf.remaining(), data.len());
                buf.put_slice(&data[..n]);
                if n < data.len() {
                    this.read_buf.extend_from_slice(&data[n..]);
                }
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Err(e)),
            Poll::Ready(None) => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "connection closed",
            ))),
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

        if let Some(ref mut fut) = this.in_flight_write {
            let result = ready!(fut.as_mut().poll(cx));
            this.in_flight_write = None;
            return Poll::Ready(result);
        }

        let data = buf.to_vec();
        let len = data.len();
        let tx = this.write_tx.clone();
        let notify = this.notify.clone();
        this.in_flight_write = Some(Box::pin(async move {
            tx.send(data)
                .await
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "connection closed"))?;
            notify.notify_one();
            Ok(len)
        }));
        cx.waker().wake_by_ref();
        Poll::Pending
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
