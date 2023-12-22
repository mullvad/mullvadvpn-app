use futures::Stream;
use hyper::client::connect::Connected;
use mullvad_types::access_method;
use serde::{Deserialize, Serialize};
use std::{
    fmt, io,
    path::Path,
    pin::Pin,
    task::{self, Poll},
};
use talpid_types::{
    net::{Endpoint, TransportProtocol},
    ErrorExt,
};
use tokio::{
    fs,
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf},
};

const CURRENT_CONFIG_FILENAME: &str = "api-endpoint.json";

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum ApiConnectionMode {
    /// Connect directly to the target.
    Direct,
    /// Connect to the destination via a proxy.
    Proxied(ProxyConfig),
}

impl fmt::Display for ApiConnectionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ApiConnectionMode::Direct => write!(f, "unproxied"),
            ApiConnectionMode::Proxied(settings) => settings.fmt(f),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum ProxyConfig {
    Shadowsocks(access_method::Shadowsocks),
    Socks(access_method::Socks5),
}

impl ProxyConfig {
    /// Returns the remote endpoint describing how to reach the proxy.
    fn get_endpoint(&self) -> Endpoint {
        match self {
            ProxyConfig::Shadowsocks(shadowsocks) => {
                Endpoint::from_socket_address(shadowsocks.peer, TransportProtocol::Tcp)
            }
            ProxyConfig::Socks(socks) => match socks {
                access_method::Socks5::Local(local) => local.remote_endpoint,
                access_method::Socks5::Remote(remote) => {
                    Endpoint::from_socket_address(remote.peer, TransportProtocol::Tcp)
                }
            },
        }
    }
}

impl fmt::Display for ProxyConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let endpoint = self.get_endpoint();
        match self {
            ProxyConfig::Shadowsocks(_) => write!(f, "Shadowsocks {}", endpoint),
            ProxyConfig::Socks(socks) => match socks {
                access_method::Socks5::Remote(_) => write!(f, "Socks5 {}", endpoint),
                access_method::Socks5::Local(local) => {
                    write!(f, "Socks5 {} via localhost:{}", endpoint, local.local_port)
                }
            },
        }
    }
}

impl ApiConnectionMode {
    /// Reads the proxy config from `CURRENT_CONFIG_FILENAME`.
    /// This returns `ApiConnectionMode::Direct` if reading from disk fails for any reason.
    pub async fn try_from_cache(cache_dir: &Path) -> Self {
        Self::from_cache(cache_dir).await.unwrap_or_else(|error| {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to read API endpoint cache")
            );
            ApiConnectionMode::Direct
        })
    }

    /// Reads the proxy config from `CURRENT_CONFIG_FILENAME`.
    /// If the file does not exist, this returns `Ok(ApiConnectionMode::Direct)`.
    async fn from_cache(cache_dir: &Path) -> io::Result<Self> {
        let path = cache_dir.join(CURRENT_CONFIG_FILENAME);
        match fs::read_to_string(path).await {
            Ok(s) => serde_json::from_str(&s).map_err(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg(&format!(
                        "Failed to deserialize \"{CURRENT_CONFIG_FILENAME}\""
                    ))
                );
                io::Error::new(io::ErrorKind::Other, "deserialization failed")
            }),
            Err(error) => {
                if error.kind() == io::ErrorKind::NotFound {
                    Ok(ApiConnectionMode::Direct)
                } else {
                    Err(error)
                }
            }
        }
    }

    /// Stores this config to `CURRENT_CONFIG_FILENAME`.
    pub async fn save(&self, cache_dir: &Path) -> io::Result<()> {
        let mut file = mullvad_fs::AtomicFile::new(cache_dir.join(CURRENT_CONFIG_FILENAME)).await?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "serialization failed"))?;
        file.write_all(json.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.finalize().await
    }

    /// Attempts to remove `CURRENT_CONFIG_FILENAME`, if it exists.
    pub async fn try_delete_cache(cache_dir: &Path) {
        let path = cache_dir.join(CURRENT_CONFIG_FILENAME);
        if let Err(err) = fs::remove_file(path).await {
            if err.kind() != std::io::ErrorKind::NotFound {
                log::error!(
                    "{}",
                    err.display_chain_with_msg("Failed to remove old API config")
                );
            }
        }
    }

    /// Returns the remote endpoint required to reach the API, or `None` for
    /// `ApiConnectionMode::Direct`.
    pub fn get_endpoint(&self) -> Option<Endpoint> {
        match self {
            ApiConnectionMode::Direct => None,
            ApiConnectionMode::Proxied(proxy_config) => Some(proxy_config.get_endpoint()),
        }
    }

    pub fn is_proxy(&self) -> bool {
        *self != ApiConnectionMode::Direct
    }

    /// Convenience function that returns a stream that repeats
    /// this config forever.
    pub fn into_repeat(self) -> impl Stream<Item = ApiConnectionMode> {
        futures::stream::repeat(self)
    }
}

/// Implements `hyper::client::connect::Connection` by wrapping a type.
pub struct ConnectionDecorator<T: AsyncRead + AsyncWrite>(pub T);

impl<T: AsyncRead + AsyncWrite + Unpin> AsyncRead for ConnectionDecorator<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for ConnectionDecorator<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

impl<T: AsyncRead + AsyncWrite> hyper::client::connect::Connection for ConnectionDecorator<T> {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

trait Connection: AsyncRead + AsyncWrite + Unpin + hyper::client::connect::Connection + Send {}

impl<T: AsyncRead + AsyncWrite + Unpin + hyper::client::connect::Connection + Send> Connection
    for T
{
}

/// Stream that represents a Mullvad API connection
pub struct ApiConnection(Box<dyn Connection>);

impl ApiConnection {
    pub fn new<
        T: AsyncRead + AsyncWrite + Unpin + hyper::client::connect::Connection + Send + 'static,
    >(
        conn: Box<T>,
    ) -> Self {
        Self(conn)
    }
}

impl AsyncRead for ApiConnection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for ApiConnection {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

impl hyper::client::connect::Connection for ApiConnection {
    fn connected(&self) -> Connected {
        self.0.connected()
    }
}
