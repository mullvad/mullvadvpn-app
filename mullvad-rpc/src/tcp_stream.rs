//! Wrapper around [`tokio::net::TcpStream`]. This allows in-flight requests to be cancelled
//! immediately instead of after the socket times out.

use futures::channel::oneshot;
use hyper::client::connect::{Connected, Connection};
use std::{
    io,
    net::Shutdown,
    pin::Pin,
    sync::{Arc, Mutex, Weak},
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::TcpStream as TokioTcpStream,
};

#[derive(Debug)]
pub struct TcpStreamHandle {
    inner: Weak<Mutex<Option<StreamInner>>>,
}

impl TcpStreamHandle {
    pub fn close(self) {
        if let Some(inner_lock) = self.inner.upgrade() {
            if let Ok(Some(inner)) = inner_lock.lock().map(|mut inner| inner.take()) {
                if let Err(err) = flatten_result(
                    inner
                        .stream
                        .into_std()
                        .map(|stream| stream.shutdown(Shutdown::Both)),
                ) {
                    log::error!("Failed to shut down TCP socket: {}", err);
                }
            }
        }
    }
}

pub struct TcpStream {
    inner: Arc<Mutex<Option<StreamInner>>>,
}

impl TcpStream {
    pub fn new(
        stream: TokioTcpStream,
        shutdown_tx: Option<oneshot::Sender<()>>,
    ) -> (Self, TcpStreamHandle) {
        let inner = Arc::new(Mutex::new(Some(StreamInner {
            stream,
            shutdown_tx,
        })));
        let stream_handle = TcpStreamHandle {
            inner: Arc::downgrade(&inner),
        };
        (Self { inner }, stream_handle)
    }

    fn do_stream<T>(
        &self,
        mut stream_fn: impl FnMut(&mut TokioTcpStream) -> T,
        closed_value: T,
    ) -> T {
        let mut inner = self.inner.lock().expect("TCP lock poisoned");
        if let Some(inner) = &mut *inner {
            stream_fn(&mut inner.stream)
        } else {
            closed_value
        }
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        if let Ok(Some(mut inner)) = self.inner.lock().map(|mut inner| inner.take()) {
            if let Some(tx) = inner.shutdown_tx.take() {
                let _ = tx.send(());
            }
        }
    }
}

impl AsyncWrite for TcpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.do_stream(
            |stream| Pin::new(stream).poll_write(cx, buf),
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "socket is closed",
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

impl AsyncRead for TcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.do_stream(
            |stream| Pin::new(stream).poll_read(cx, buf),
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "socket is closed",
            ))),
        )
    }
}

impl Connection for TcpStream {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

#[derive(Debug)]
struct StreamInner {
    stream: TokioTcpStream,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

fn flatten_result<T, E>(result: Result<Result<T, E>, E>) -> Result<T, E> {
    match result {
        Ok(value) => value,
        Err(err) => Err(err),
    }
}
