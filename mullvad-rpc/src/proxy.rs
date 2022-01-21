use crate::tls_stream::TlsStream;
use futures::Future;
use hyper::client::connect::{Connected, Connection};
use shadowsocks::relay::tcprelay::ProxyClientStream;
use std::{
    io,
    net::SocketAddr,
    pin::Pin,
    task::{self, Poll},
};
use talpid_types::net::openvpn::ShadowsocksProxySettings;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::TcpStream,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ProxyConfig {
    /// Connect directly to the target.
    Tls,
    /// Connect to the destination via a proxy.
    Proxied(ShadowsocksProxySettings),
}

impl ProxyConfig {
    /// Returns the remote address, or `None` for `ProxyConfig::Tls`.
    pub fn get_endpoint(&self) -> Option<SocketAddr> {
        match self {
            ProxyConfig::Proxied(ss) => Some(ss.peer),
            ProxyConfig::Tls => None,
        }
    }

    pub fn is_proxy(&self) -> bool {
        *self != ProxyConfig::Tls
    }
}

pub trait ProxyConfigProvider: Send + Sync {
    fn next(&self) -> Pin<Box<dyn Future<Output = ProxyConfig> + Send>>;
}

pub struct ProxyConfigProviderNoop(pub ());

impl ProxyConfigProvider for ProxyConfigProviderNoop {
    fn next(&self) -> Pin<Box<dyn Future<Output = ProxyConfig> + Send>> {
        Box::pin(async { ProxyConfig::Tls })
    }
}

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
