use crate::tls_stream::TlsStream;
use hyper::client::connect::{Connected, Connection};
use shadowsocks::relay::tcprelay::ProxyClientStream;
use std::{
    io,
    pin::Pin,
    task::{self, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::TcpStream,
};

/// Stream that is either a regular TLS stream or TLS via shadowsocks
pub enum MaybeProxyStream {
    Tls(TlsStream<TcpStream>),
    Proxied(TlsStream<ProxyClientStream<TcpStream>>),
}

impl AsyncRead for MaybeProxyStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match Pin::get_mut(self) {
            MaybeProxyStream::Tls(s) => Pin::new(s).poll_read(cx, buf),
            MaybeProxyStream::Proxied(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for MaybeProxyStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match Pin::get_mut(self) {
            MaybeProxyStream::Tls(s) => Pin::new(s).poll_write(cx, buf),
            MaybeProxyStream::Proxied(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        match Pin::get_mut(self) {
            MaybeProxyStream::Tls(s) => Pin::new(s).poll_flush(cx),
            MaybeProxyStream::Proxied(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        match Pin::get_mut(self) {
            MaybeProxyStream::Tls(s) => Pin::new(s).poll_shutdown(cx),
            MaybeProxyStream::Proxied(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

impl Connection for MaybeProxyStream {
    fn connected(&self) -> Connected {
        match self {
            MaybeProxyStream::Tls(s) => s.connected(),
            MaybeProxyStream::Proxied(s) => s.connected(),
        }
    }
}
