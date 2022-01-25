//! Wrapper around a stream to make it abortable. This allows in-flight requests to be cancelled
//! immediately instead of after the socket times out.

use futures::channel::oneshot;
use hyper::client::connect::{Connected, Connection};
use std::{
    future::Future,
    io,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

#[derive(Clone, Debug)]
pub struct AbortableStreamHandle {
    tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl AbortableStreamHandle {
    pub fn close(self) {
        if let Some(tx) = self.tx.lock().unwrap().take() {
            let _ = tx.send(());
        }
    }
}

pub struct AbortableStream<S: Unpin> {
    stream: S,
    shutdown_tx: Option<oneshot::Sender<()>>,
    shutdown_rx: oneshot::Receiver<()>,
}

impl<S> AbortableStream<S>
where
    S: Unpin + Send + 'static,
{
    pub fn new(
        stream: S,
        shutdown_tx: Option<oneshot::Sender<()>>,
    ) -> (Self, AbortableStreamHandle) {
        let (tx, rx) = oneshot::channel();
        let stream_handle = AbortableStreamHandle {
            tx: Arc::new(Mutex::new(Some(tx))),
        };
        (
            Self {
                stream,
                shutdown_tx,
                shutdown_rx: rx,
            },
            stream_handle,
        )
    }
}

impl<S> Drop for AbortableStream<S>
where
    S: Unpin,
{
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl<S> AsyncWrite for AbortableStream<S>
where
    S: AsyncWrite + Unpin + Send + 'static,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        if let Poll::Ready(_) = Pin::new(&mut self.shutdown_rx).poll(cx) {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "stream is closed",
            )));
        }
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        if let Poll::Ready(_) = Pin::new(&mut self.shutdown_rx).poll(cx) {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "stream is closed",
            )));
        }
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

impl<S> AsyncRead for AbortableStream<S>
where
    S: AsyncRead + Unpin + Send + 'static,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if let Poll::Ready(_) = Pin::new(&mut self.shutdown_rx).poll(cx) {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "stream is closed",
            )));
        }
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl<S> Connection for AbortableStream<S>
where
    S: Connection + Unpin,
{
    fn connected(&self) -> Connected {
        self.stream.connected()
    }
}
