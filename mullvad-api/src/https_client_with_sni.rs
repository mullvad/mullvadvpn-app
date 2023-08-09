use crate::{
    abortable_stream::{AbortableStream, AbortableStreamHandle},
    proxy::{ApiConnection, ApiConnectionMode, ProxyConfig},
    tls_stream::TlsStream,
    AddressCache,
};
use futures::{channel::mpsc, future, pin_mut, StreamExt};
#[cfg(target_os = "android")]
use futures::{channel::oneshot, sink::SinkExt};
use http::uri::Scheme;
use hyper::{
    client::connect::dns::{GaiResolver, Name},
    service::Service,
    Uri,
};
use shadowsocks::{
    config::ServerType,
    context::{Context as SsContext, SharedContext},
    crypto::CipherKind,
    relay::tcprelay::ProxyClientStream,
    ServerConfig,
};
#[cfg(target_os = "android")]
use std::os::unix::io::{AsRawFd, RawFd};
use std::{
    fmt,
    future::Future,
    io,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    str::{self, FromStr},
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Duration,
};
use talpid_types::ErrorExt;

use tokio::{
    net::{TcpSocket, TcpStream},
    time::timeout,
};

#[cfg(feature = "api-override")]
use crate::{proxy::ConnectionDecorator, API};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct HttpsConnectorWithSniHandle {
    tx: mpsc::UnboundedSender<HttpsConnectorRequest>,
}

impl HttpsConnectorWithSniHandle {
    /// Stop all streams produced by this connector
    pub fn reset(&self) {
        let _ = self.tx.unbounded_send(HttpsConnectorRequest::Reset);
    }

    /// Change the proxy settings for the connector
    pub fn set_connection_mode(&self, proxy: ApiConnectionMode) {
        let _ = self
            .tx
            .unbounded_send(HttpsConnectorRequest::SetConnectionMode(proxy));
    }
}

enum HttpsConnectorRequest {
    Reset,
    SetConnectionMode(ApiConnectionMode),
}

#[derive(Clone)]
enum InnerConnectionMode {
    /// Connect directly to the target.
    Direct,
    /// Connect to the destination via a Shadowsocks proxy.
    Shadowsocks(ShadowsocksConfig),
    /// Connect to the destination via a Socks proxy.
    Socks5(SocksConfig),
}

impl InnerConnectionMode {
    async fn connect(
        &self,
        hostname: &str,
        addr: &SocketAddr,
    ) -> Result<ApiConnection, std::io::Error> {
        match self {
            InnerConnectionMode::Direct => handle_direct_connection(addr, hostname).await,
            InnerConnectionMode::Shadowsocks(config) => {
                handle_shadowsocks_connection(config.clone(), addr, hostname).await
            }
            InnerConnectionMode::Socks(proxy_config) => {
                handle_socks_connection(proxy_config.clone(), addr, hostname).await
            }
        }
    }
}

#[derive(Clone)]
struct ShadowsocksConfig {
    proxy_context: SharedContext,
    params: ParsedShadowsocksConfig,
}

#[derive(Clone)]
struct ParsedShadowsocksConfig {
    peer: SocketAddr,
    password: String,
    cipher: CipherKind,
}

impl From<ParsedShadowsocksConfig> for ServerConfig {
    fn from(config: ParsedShadowsocksConfig) -> Self {
        ServerConfig::new(config.peer, config.password, config.cipher)
    }
}

#[derive(Clone)]
struct SocksConfig {
    peer: SocketAddr,
}

#[derive(err_derive::Error, Debug)]
enum ProxyConfigError {
    #[error(display = "Unrecognized cipher selected: {}", _0)]
    InvalidCipher(String),
}

impl TryFrom<ApiConnectionMode> for InnerConnectionMode {
    type Error = ProxyConfigError;

    fn try_from(config: ApiConnectionMode) -> Result<Self, Self::Error> {
        Ok(match config {
            ApiConnectionMode::Direct => InnerConnectionMode::Direct,
            ApiConnectionMode::Proxied(ProxyConfig::Shadowsocks(config)) => {
                InnerConnectionMode::Shadowsocks(ShadowsocksConfig {
                    params: ParsedShadowsocksConfig {
                        peer: config.peer,
                        password: config.password,
                        cipher: CipherKind::from_str(&config.cipher)
                            .map_err(|_| ProxyConfigError::InvalidCipher(config.cipher))?,
                    },
                    proxy_context: SsContext::new_shared(ServerType::Local),
                })
            }
        })
    }
}

/// A Connector for the `https` scheme.
#[derive(Clone)]
pub struct HttpsConnectorWithSni {
    inner: Arc<Mutex<HttpsConnectorWithSniInner>>,
    sni_hostname: Option<String>,
    address_cache: AddressCache,
    abort_notify: Arc<tokio::sync::Notify>,
    #[cfg(target_os = "android")]
    socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
}

struct HttpsConnectorWithSniInner {
    stream_handles: Vec<AbortableStreamHandle>,
    proxy_config: InnerConnectionMode,
}

#[cfg(target_os = "android")]
pub type SocketBypassRequest = (RawFd, oneshot::Sender<()>);

impl HttpsConnectorWithSni {
    pub fn new(
        sni_hostname: Option<String>,
        address_cache: AddressCache,
        #[cfg(target_os = "android")] socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
    ) -> (Self, HttpsConnectorWithSniHandle) {
        let (tx, mut rx) = mpsc::unbounded();
        let abort_notify = Arc::new(tokio::sync::Notify::new());
        let inner = Arc::new(Mutex::new(HttpsConnectorWithSniInner {
            stream_handles: vec![],
            proxy_config: InnerConnectionMode::Direct,
        }));

        let inner_copy = inner.clone();
        let notify = abort_notify.clone();
        tokio::spawn(async move {
            // Handle requests by `HttpsConnectorWithSniHandle`s
            while let Some(request) = rx.next().await {
                let handles = {
                    let mut inner = inner_copy.lock().unwrap();

                    if let HttpsConnectorRequest::SetConnectionMode(config) = request {
                        match InnerConnectionMode::try_from(config) {
                            Ok(config) => {
                                inner.proxy_config = config;
                            }
                            Err(error) => {
                                log::error!(
                                    "{}",
                                    error.display_chain_with_msg(
                                        "Failed to parse new API proxy config"
                                    )
                                );
                            }
                        }
                    }

                    std::mem::take(&mut inner.stream_handles)
                };
                for handle in handles {
                    handle.close();
                }
                notify.notify_waiters();
            }
        });

        (
            HttpsConnectorWithSni {
                inner,
                sni_hostname,
                address_cache,
                abort_notify,
                #[cfg(target_os = "android")]
                socket_bypass_tx,
            },
            HttpsConnectorWithSniHandle { tx },
        )
    }

    /// Establishes a TCP connection with a peer at the specified socket address.
    ///
    /// Will timeout after [`CONNECT_TIMEOUT`] seconds.
    async fn open_socket(
        addr: SocketAddr,
        #[cfg(target_os = "android")] socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
    ) -> std::io::Result<TcpStream> {
        let socket = match addr {
            SocketAddr::V4(_) => TcpSocket::new_v4()?,
            SocketAddr::V6(_) => TcpSocket::new_v6()?,
        };

        #[cfg(target_os = "android")]
        if let Some(mut tx) = socket_bypass_tx {
            let (done_tx, done_rx) = oneshot::channel();
            let _ = tx.send((socket.as_raw_fd(), done_tx)).await;
            if done_rx.await.is_err() {
                log::error!("Failed to bypass socket, connection might fail");
            }
        }

        timeout(CONNECT_TIMEOUT, socket.connect(addr))
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::TimedOut, err))?
    }

    async fn resolve_address(address_cache: AddressCache, uri: Uri) -> io::Result<SocketAddr> {
        const DEFAULT_PORT: u16 = 443;

        let hostname = uri.host().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "invalid url, missing host")
        })?;
        let port = uri.port_u16();
        if let Ok(addr) = hostname.parse::<IpAddr>() {
            return Ok(SocketAddr::new(addr, port.unwrap_or(DEFAULT_PORT)));
        }

        // Preferentially, use cached address.
        //
        if let Some(addr) = address_cache.resolve_hostname(hostname).await {
            return Ok(SocketAddr::new(
                addr.ip(),
                port.unwrap_or_else(|| addr.port()),
            ));
        }

        // Use getaddrinfo as a fallback
        //
        let mut addrs = GaiResolver::new()
            .call(
                Name::from_str(hostname)
                    .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?,
            )
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        let addr = addrs
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Empty DNS response"))?;
        Ok(SocketAddr::new(addr.ip(), port.unwrap_or(DEFAULT_PORT)))
    }
}

impl fmt::Debug for HttpsConnectorWithSni {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpsConnectorWithSni").finish()
    }
}

/// Set up a TCP-socket connection.
async fn handle_direct_connection(
    addr: &SocketAddr,
    hostname: &str,
) -> Result<ApiConnection, io::Error> {
    let socket = HttpsConnectorWithSni::open_socket(
        *addr,
        #[cfg(target_os = "android")]
        socket_bypass_tx.clone(),
    )
    .await?;
    #[cfg(feature = "api-override")]
    if API.disable_tls {
        return Ok(ApiConnection::new(Box::new(socket)));
    }

    let tls_stream = TlsStream::connect_https(socket, hostname).await?;
    Ok(ApiConnection::new(Box::new(tls_stream)))
}

/// Set up a shadowsocks-socket connection.
async fn handle_shadowsocks_connection(
    shadowsocks: ShadowsocksConfig,
    addr: &SocketAddr,
    hostname: &str,
) -> Result<ApiConnection, io::Error> {
    let socket = HttpsConnectorWithSni::open_socket(
        shadowsocks.params.peer,
        #[cfg(target_os = "android")]
        socket_bypass_tx.clone(),
    )
    .await?;
    let proxy = ProxyClientStream::from_stream(
        shadowsocks.proxy_context,
        socket,
        &ServerConfig::from(shadowsocks.params),
        *addr,
    );

    #[cfg(feature = "api-override")]
    if API.disable_tls {
        return Ok(ApiConnection::new(Box::new(ConnectionDecorator(proxy))));
    }

    let tls_stream = TlsStream::connect_https(proxy, hostname).await?;
    Ok(ApiConnection::new(Box::new(tls_stream)))
}

/// Set up a SOCKS5-socket connection.
///
/// TODO: Handle case where the proxy-address is `localhost`.
async fn handle_socks_connection(
    proxy_config: SocksConfig,
    addr: &SocketAddr,
    hostname: &str,
) -> Result<ApiConnection, io::Error> {
    let socket = HttpsConnectorWithSni::open_socket(
        proxy_config.peer,
        #[cfg(target_os = "android")]
        socket_bypass_tx.clone(),
    )
    .await?;
    let proxy = tokio_socks::tcp::Socks5Stream::connect_with_socket(socket, addr)
        .await
        .map_err(|error| io::Error::new(io::ErrorKind::Other, format!("SOCKS error: {error}")))?;

    #[cfg(feature = "api-override")]
    if API.disable_tls {
        return Ok(ApiConnection::new(Box::new(ConnectionDecorator(proxy))));
    }

    let tls_stream = TlsStream::connect_https(proxy, hostname).await?;
    Ok(ApiConnection::new(Box::new(tls_stream)))
}

impl Service<Uri> for HttpsConnectorWithSni {
    type Response = AbortableStream<ApiConnection>;
    type Error = io::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut inner = self.inner.lock().unwrap();
        inner.stream_handles.retain(|handle| !handle.is_closed());
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let sni_hostname = self
            .sni_hostname
            .clone()
            .or_else(|| uri.host().map(str::to_owned))
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "invalid url, missing host")
            });
        let inner = self.inner.clone();
        let abort_notify = self.abort_notify.clone();
        #[cfg(target_os = "android")]
        let socket_bypass_tx = self.socket_bypass_tx.clone();
        let address_cache = self.address_cache.clone();

        let fut = async move {
            if uri.scheme() != Some(&Scheme::HTTPS) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid url, not https",
                ));
            }

            let hostname = sni_hostname?;
            let addr = Self::resolve_address(address_cache, uri).await?;

            // Loop until we have established a connection. This starts over if a new endpoint
            // is selected while connecting.
            let stream = loop {
                let notify = abort_notify.notified();
                let proxy_config = { inner.lock().unwrap().proxy_config.clone() };
                let stream_fut = proxy_config.connect(&hostname, &addr);

                pin_mut!(stream_fut);
                pin_mut!(notify);

                // Wait for connection. Abort and retry if we switched to a different server.
                if let future::Either::Left((stream, _)) = future::select(stream_fut, notify).await
                {
                    break stream?;
                }
            };

            let (stream, socket_handle) = AbortableStream::new(stream);

            {
                let mut inner = inner.lock().unwrap();
                inner.stream_handles.push(socket_handle);
            }

            Ok(stream)
        };

        Box::pin(fut)
    }
}
