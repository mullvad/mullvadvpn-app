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

    /// Returns whether the stream has already stopped on its own.
    pub fn is_closed(&self) -> bool {
        self.tx
            .lock()
            .unwrap()
            .as_ref()
            .map(|tx| tx.is_canceled())
            .unwrap_or(true)
    }
}

pub struct AbortableStream<S: Unpin> {
    stream: S,
    shutdown_rx: oneshot::Receiver<()>,
}

impl<S> AbortableStream<S>
where
    S: Unpin + Send + 'static,
{
    pub fn new(stream: S) -> (Self, AbortableStreamHandle) {
        let (tx, rx) = oneshot::channel();
        let stream_handle = AbortableStreamHandle {
            tx: Arc::new(Mutex::new(Some(tx))),
        };
        (
            Self {
                stream,
                shutdown_rx: rx,
            },
            stream_handle,
        )
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

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;
    use tokio::io::AsyncReadExt;

    /// Test whether the abort handle stops the stream.
    #[test]
    fn test_abort() {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to initialize runtime");

        let (client, _server) = tokio::io::duplex(64);

        runtime.block_on(async move {
            let (mut stream, abort_handle) = AbortableStream::new(client);

            let stream_task = tokio::spawn(async move {
                let mut buf = vec![];
                stream.read_to_end(&mut buf).await
            });

            abort_handle.close();
            let result = tokio::time::timeout(Duration::from_secs(1), stream_task)
                .await
                .unwrap();
            assert!(
                matches!(result, Ok(Err(error)) if error.kind() == io::ErrorKind::ConnectionReset)
            );
        });
    }

    /// Test the `AbortableStreamHandle::is_closed` method when explicitly closed.
    #[test]
    fn test_shutdown_signal() {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to initialize runtime");

        let (client, _server) = tokio::io::duplex(64);

        runtime.block_on(async move {
            let (_stream, abort_handle) = AbortableStream::new(client);
            let abort_handle_2 = abort_handle.clone();
            assert!(!abort_handle_2.is_closed());
            abort_handle.close();
            assert!(abort_handle_2.is_closed());
        });
    }

    /// Test the `AbortableStreamHandle::is_closed` method when the stream stops on its own.
    #[test]
    fn test_shutdown_signal_normal() {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to initialize runtime");

        let (client, server) = tokio::io::duplex(64);

        runtime.block_on(async move {
            let (mut stream, abort_handle) = AbortableStream::new(client);

            assert!(!abort_handle.is_closed());

            let stream_task = tokio::spawn(async move {
                drop(server);
                let mut buf = vec![];
                stream.read_to_end(&mut buf).await
            });

            assert!(tokio::time::timeout(Duration::from_secs(1), stream_task)
                .await
                .unwrap()
                .is_ok());
            assert!(abort_handle.is_closed());
        });
    }
}
