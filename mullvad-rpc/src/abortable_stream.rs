//! Wrapper around a stream to make it abortable.

use futures::channel::oneshot;
use hyper::client::connect::{Connected, Connection};
use std::{
    io,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};

#[derive(Debug)]
pub struct AbortableStreamHandle {
    tx: oneshot::Sender<()>,
}

impl AbortableStreamHandle {
    pub fn close(self) {
        let _ = self.tx.send(());
    }
}

pub struct AbortableStream<S: AsyncRead + AsyncWrite + Connection + Unpin> {
    inner: Arc<Mutex<Option<StreamInner<S>>>>,
}

impl<S> AbortableStream<S>
where
    S: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub fn new(
        stream: S,
        shutdown_tx: Option<oneshot::Sender<()>>,
    ) -> (Self, AbortableStreamHandle) {
        let inner = Arc::new(Mutex::new(Some(StreamInner {
            stream,
            shutdown_tx,
        })));

        let (tx, rx) = oneshot::channel();
        let inner_copy = Arc::downgrade(&inner);

        tokio::spawn(async move {
            let _ = rx.await;

            if let Some(inner_lock) = inner_copy.upgrade() {
                let inner = { inner_lock.lock().unwrap().take() };
                if let Some(mut inner) = inner {
                    if let Err(error) = inner.stream.shutdown().await {
                        log::error!("Failed to shut down stream: {}", error);
                    }
                }
            }
        });

        let stream_handle = AbortableStreamHandle { tx };
        (Self { inner }, stream_handle)
    }

    fn do_stream<T>(&self, mut stream_fn: impl FnMut(&mut S) -> T, closed_value: T) -> T {
        let mut inner = self.inner.lock().expect("Stream lock poisoned");
        if let Some(inner) = &mut *inner {
            stream_fn(&mut inner.stream)
        } else {
            closed_value
        }
    }
}

impl<S> Drop for AbortableStream<S>
where
    S: AsyncRead + AsyncWrite + Connection + Unpin,
{
    fn drop(&mut self) {
        if let Ok(Some(mut inner)) = self.inner.lock().map(|mut inner| inner.take()) {
            if let Some(tx) = inner.shutdown_tx.take() {
                let _ = tx.send(());
            }
        }
    }
}

impl<S> AsyncWrite for AbortableStream<S>
where
    S: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.do_stream(
            |stream| Pin::new(stream).poll_write(cx, buf),
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "stream is closed",
            ))),
        )
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.do_stream(
            |stream| Pin::new(stream).poll_flush(cx),
            Poll::Ready(Ok(())),
        )
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.do_stream(
            |stream| Pin::new(stream).poll_shutdown(cx),
            Poll::Ready(Ok(())),
        )
    }
}

impl<S> AsyncRead for AbortableStream<S>
where
    S: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.do_stream(
            |stream| Pin::new(stream).poll_read(cx, buf),
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "stream is closed",
            ))),
        )
    }
}

impl<S> Connection for AbortableStream<S>
where
    S: AsyncRead + AsyncWrite + Connection + Unpin,
{
    fn connected(&self) -> Connected {
        if let Some(inner) = &*self.inner.lock().unwrap() {
            inner.stream.connected()
        } else {
            Connected::new()
        }
    }
}

#[derive(Debug)]
struct StreamInner<S: AsyncRead + AsyncWrite + Connection + Unpin> {
    stream: S,
    shutdown_tx: Option<oneshot::Sender<()>>,
}
