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
    inner: Weak<Mutex<StreamInner>>,
}

impl TcpStreamHandle {
    pub fn close(self) {
        if let Some(inner_lock) = self.inner.upgrade() {
            if let Ok(mut inner) = inner_lock.lock() {
                if let Err(err) = inner.stream.shutdown(Shutdown::Both) {
                    log::error!("Failed to shut down TCP socket: {}", err);
                }
                let _ = inner.shutdown_tx.take();
            }
        }
    }
}


pub struct TcpStream {
    inner: Arc<Mutex<StreamInner>>,
}

impl TcpStream {
    pub fn new(
        stream: TokioTcpStream,
        id: usize,
        shutdown_tx: Option<oneshot::Sender<()>>,
    ) -> (Self, TcpStreamHandle) {
        let inner = Arc::new(Mutex::new(StreamInner {
            id,
            stream,
            shutdown_tx,
        }));
        (
            Self {
                inner: inner.clone(),
            },
            TcpStreamHandle {
                inner: Arc::downgrade(&inner),
            },
        )
    }

    fn do_stream<T>(&self, mut stream_fn: impl FnMut(&mut TokioTcpStream) -> T) -> T {
        let mut inner = self.inner.lock().expect("TCP lock poisoned");
        stream_fn(&mut inner.stream)
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        if let Ok(mut inner) = self.inner.lock() {
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
        self.do_stream(|stream| Pin::new(stream).poll_write(cx, buf))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.do_stream(|stream| Pin::new(stream).poll_flush(cx))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.do_stream(|stream| Pin::new(stream).poll_shutdown(cx))
    }
}

impl AsyncRead for TcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.do_stream(|stream| Pin::new(stream).poll_read(cx, buf))
    }
}

impl Connection for TcpStream {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

#[derive(Debug)]
struct StreamInner {
    id: usize,
    stream: TokioTcpStream,
    shutdown_tx: Option<oneshot::Sender<()>>,
}
