use crate::{
    DnsResolver,
    abortable_stream::{AbortableStream, AbortableStreamHandle},
    domain_fronting::DfConfigResolved,
    proxy::{ApiConnection, ApiConnectionMode, ProxyConfig},
    tls_stream::TlsStream,
};
use domain_fronting::client::ProxyConnection;
use futures::{FutureExt, future, select};
#[cfg(target_os = "android")]
use futures::{channel::oneshot, sink::SinkExt};
use http::uri::Scheme;
use hyper::Uri;
use hyper_util::rt::TokioIo;
use mullvad_encrypted_dns_proxy::{
    Forwarder as EncryptedDNSForwarder, config::ProxyConfig as EncryptedDNSConfig,
};
use shadowsocks::{
    ServerConfig,
    config::{ServerConfigError, ServerType},
    context::{Context as SsContext, SharedContext},
    crypto::CipherKind,
    relay::tcprelay::ProxyClientStream,
};
#[cfg(target_os = "android")]
use std::os::unix::io::{AsRawFd, RawFd};
use std::{
    fmt,
    future::Future,
    io, mem,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    str,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Duration,
};
use talpid_types::net::proxy;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpSocket, TcpStream},
    sync::Notify,
    time::timeout,
};
use tower::Service;
use tracing::{Instrument, Level, instrument, trace_span};

#[cfg(any(feature = "api-override", test))]
use crate::proxy::ConnectionDecorator;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Communicate asynchronously with an [HttpsConnector] actor.
/// Created via [HttpsConnector::spawn].
#[derive(Clone)]
pub struct HttpsConnectorHandle {
    state: Arc<Mutex<HttpsConnectorInner>>,
}

impl HttpsConnectorHandle {
    /// Stop all streams produced by the connector
    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        state.reset();
    }

    /// Change the proxy settings for the connector
    pub fn set_connection_mode(&self, proxy: ApiConnectionMode) {
        let mut state = self.state.lock().unwrap();
        state.proxy_config = InnerConnectionMode::from(proxy);
        state.reset();
    }
}

#[derive(Clone, Debug)]
pub(crate) enum InnerConnectionMode {
    /// Connect directly to the target.
    Direct,
    /// Connect to the destination via a Shadowsocks proxy.
    Shadowsocks(ShadowsocksConfig),
    /// Connect to the destination via a Socks proxy.
    Socks5(SocksConfig),
    /// Connect to the destination via Mullvad Encrypted DNS proxy.
    /// See [`mullvad_encrypted_dns_proxy`] for how the proxy works.
    EncryptedDnsProxy(EncryptedDNSConfig),
    /// Connect to the destination via domain fronting.
    DomainFronting(DfConfigResolved),
}

impl InnerConnectionMode {
    #[instrument(level = Level::TRACE, skip(socket_bypass_tx, disable_tls), ret)]
    async fn connect(
        self,
        hostname: &str,
        addr: &SocketAddr,
        #[cfg(target_os = "android")] socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
        #[cfg(any(feature = "api-override", test))] disable_tls: bool,
    ) -> Result<ApiConnection, std::io::Error> {
        match self {
            // Set up a TCP-socket connection.
            InnerConnectionMode::Direct => {
                let first_hop = *addr;
                let make_proxy_stream = |tcp_stream| async { Ok(tcp_stream) };
                Self::connect_proxied(
                    first_hop,
                    hostname,
                    make_proxy_stream,
                    #[cfg(target_os = "android")]
                    socket_bypass_tx,
                    #[cfg(any(feature = "api-override", test))]
                    disable_tls,
                )
                .await
            }
            // Set up a Shadowsocks-connection.
            InnerConnectionMode::Shadowsocks(shadowsocks) => {
                let first_hop = shadowsocks.params.peer;
                let make_proxy_stream = |tcp_stream| async {
                    Ok(ProxyClientStream::from_stream(
                        shadowsocks.proxy_context,
                        tcp_stream,
                        &ServerConfig::try_from(shadowsocks.params)
                            .map_err(|_| std::io::Error::other("Invalid shadowsocks config"))?,
                        *addr,
                    ))
                };
                Self::connect_proxied(
                    first_hop,
                    hostname,
                    make_proxy_stream,
                    #[cfg(target_os = "android")]
                    socket_bypass_tx,
                    #[cfg(any(feature = "api-override", test))]
                    disable_tls,
                )
                .await
            }
            // Set up a SOCKS5-connection.
            InnerConnectionMode::Socks5(socks) => {
                let first_hop = socks.peer;
                let make_proxy_stream = |tcp_stream| async {
                    match socks.authentication {
                        None => {
                            tokio_socks::tcp::Socks5Stream::connect_with_socket(tcp_stream, addr)
                                .await
                        }
                        Some(credentials) => {
                            tokio_socks::tcp::Socks5Stream::connect_with_password_and_socket(
                                tcp_stream,
                                addr,
                                credentials.username(),
                                credentials.password(),
                            )
                            .await
                        }
                    }
                    .map_err(|error| io::Error::other(format!("SOCKS error: {error}")))
                };
                Self::connect_proxied(
                    first_hop,
                    hostname,
                    make_proxy_stream,
                    #[cfg(target_os = "android")]
                    socket_bypass_tx,
                    #[cfg(any(feature = "api-override", test))]
                    disable_tls,
                )
                .await
            }
            InnerConnectionMode::EncryptedDnsProxy(proxy_config) => {
                let first_hop = SocketAddr::V4(proxy_config.addr);
                let make_proxy_stream = |tcp_stream| async {
                    let forwarder = EncryptedDNSForwarder::from_stream(&proxy_config, tcp_stream);
                    Ok(forwarder)
                };
                Self::connect_proxied(
                    first_hop,
                    hostname,
                    make_proxy_stream,
                    #[cfg(target_os = "android")]
                    socket_bypass_tx,
                    #[cfg(any(feature = "api-override", test))]
                    disable_tls,
                )
                .await
            }
            InnerConnectionMode::DomainFronting(config) => {
                let use_http2 = crate::domain_fronting::USE_HTTP2;
                let domain_fronting = &config.config();
                let connect_tls = async || {
                    let tcp_stream = HttpsConnector::open_socket(
                        config.addr,
                        #[cfg(target_os = "android")]
                        socket_bypass_tx.clone(),
                    )
                    .await?;
                    crate::domain_fronting::cdn_tls_connect(
                        tcp_stream,
                        domain_fronting.front_host(),
                        use_http2,
                    )
                    .await
                };

                let proxy = if use_http2 {
                    let stream = connect_tls().await?;
                    ProxyConnection::http2_from_stream(stream, domain_fronting)
                        .await
                        .map_err(std::io::Error::other)?
                } else {
                    let (stream1, stream2) = future::try_join(connect_tls(), connect_tls()).await?;
                    ProxyConnection::http1_1_from_streams(stream1, stream2, domain_fronting)
                        .await
                        .map_err(std::io::Error::other)?
                };

                #[cfg(any(feature = "api-override", test))]
                if disable_tls {
                    return Ok(ApiConnection::new(Box::new(ConnectionDecorator(proxy))));
                }

                let tls_stream = TlsStream::connect_https(proxy, hostname).await?;
                Ok(ApiConnection::new(Box::new(tls_stream)))
            }
        }
    }

    /// Create an [`ApiConnection`] from a [`TcpStream`].
    ///
    /// The `make_proxy_stream` closure receives a [`TcpStream`] and produces a
    /// stream which can send to and receive data from some server using any
    /// proxy protocol. The only restriction is that this stream must implement
    /// [`tokio::io::AsyncRead`] and [`tokio::io::AsyncWrite`], as well as
    /// [`Unpin`] and [`Send`].
    ///
    /// If a direct connection is to be established (i.e. the stream will not be
    /// using any proxy protocol) `make_proxy_stream` may return the
    /// [`TcpStream`] itself. See for example how a connection is established
    /// from connection mode [`InnerConnectionMode::Direct`].
    async fn connect_proxied<ProxyFactory, ProxyFuture, Proxy>(
        first_hop: SocketAddr,
        hostname: &str,
        make_proxy_stream: ProxyFactory,
        #[cfg(target_os = "android")] socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
        #[cfg(any(feature = "api-override", test))] disable_tls: bool,
    ) -> Result<ApiConnection, io::Error>
    where
        ProxyFactory: FnOnce(TcpStream) -> ProxyFuture,
        ProxyFuture: Future<Output = io::Result<Proxy>>,
        Proxy: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let socket = HttpsConnector::open_socket(
            first_hop,
            #[cfg(target_os = "android")]
            socket_bypass_tx,
        )
        .await?;

        let proxy = make_proxy_stream(socket).await?;

        #[cfg(any(feature = "api-override", test))]
        if disable_tls {
            return Ok(ApiConnection::new(Box::new(ConnectionDecorator(proxy))));
        }

        let tls_stream = TlsStream::connect_https(proxy, hostname).await?;
        Ok(ApiConnection::new(Box::new(tls_stream)))
    }

    /// How long a connection may sit idle before it should be evicted.
    pub const fn idle_timeout(&self) -> Duration {
        let default = Duration::from_secs(60);
        match self {
            InnerConnectionMode::Direct => default,
            InnerConnectionMode::Shadowsocks(..) => default,
            InnerConnectionMode::Socks5(..) => default,
            InnerConnectionMode::EncryptedDnsProxy(..) => default,
            InnerConnectionMode::DomainFronting(..) => crate::domain_fronting::IDLE_TIMEOUT,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ShadowsocksConfig {
    proxy_context: SharedContext,
    params: ParsedShadowsocksConfig,
}

#[derive(Clone, Debug)]
struct ParsedShadowsocksConfig {
    peer: SocketAddr,
    password: String,
    cipher: CipherKind,
}

impl TryFrom<ParsedShadowsocksConfig> for ServerConfig {
    type Error = ServerConfigError;

    fn try_from(config: ParsedShadowsocksConfig) -> Result<Self, Self::Error> {
        ServerConfig::new(config.peer, config.password, config.cipher)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SocksConfig {
    peer: SocketAddr,
    authentication: Option<proxy::SocksAuth>,
}

impl From<ApiConnectionMode> for InnerConnectionMode {
    fn from(config: ApiConnectionMode) -> Self {
        use std::net::Ipv4Addr;
        match config {
            ApiConnectionMode::Direct => InnerConnectionMode::Direct,
            ApiConnectionMode::Proxied(proxy_settings) => match proxy_settings {
                ProxyConfig::Shadowsocks(config) => {
                    InnerConnectionMode::Shadowsocks(ShadowsocksConfig {
                        params: ParsedShadowsocksConfig {
                            peer: config.endpoint,
                            password: config.plaintext_password().to_string(),
                            cipher: config.cipher.kind(),
                        },
                        proxy_context: SsContext::new_shared(ServerType::Local),
                    })
                }
                ProxyConfig::Socks5Local(config) => InnerConnectionMode::Socks5(SocksConfig {
                    peer: SocketAddr::new(IpAddr::from(Ipv4Addr::LOCALHOST), config.local_port),
                    authentication: None,
                }),
                ProxyConfig::Socks5Remote(config) => InnerConnectionMode::Socks5(SocksConfig {
                    peer: config.endpoint,
                    authentication: config.auth,
                }),
                ProxyConfig::EncryptedDnsProxy(config) => {
                    InnerConnectionMode::EncryptedDnsProxy(config)
                }
                ProxyConfig::DomainFronting(config) => InnerConnectionMode::DomainFronting(config),
            },
        }
    }
}

/// A Connector for the `https` scheme.
#[derive(Clone)]
pub struct HttpsConnector {
    state: Arc<Mutex<HttpsConnectorInner>>,
    abort_notify: Arc<Notify>,
    dns_resolver: Arc<dyn DnsResolver>,
    #[cfg(target_os = "android")]
    socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
    #[cfg(any(feature = "api-override", test))]
    disable_tls: bool,
}

struct HttpsConnectorInner {
    stream_handles: Vec<AbortableStreamHandle>,
    proxy_config: InnerConnectionMode,
    abort_notify: Arc<Notify>,
}

impl HttpsConnectorInner {
    /// Stop all streams produced by this connector
    fn reset(&mut self) {
        for handle in mem::take(&mut self.stream_handles) {
            handle.close();
        }
        self.abort_notify.notify_waiters();
    }
}

#[cfg(target_os = "android")]
pub type SocketBypassRequest = (RawFd, oneshot::Sender<()>);

impl HttpsConnector {
    /// Creates a new [HttpsConnector] actor instance, which notably implements [`Service<Uri>`].
    ///
    /// Call [HttpsConnector::spawn] to start listening for [events](HttpsConnectorRequest).
    pub fn new(
        dns_resolver: Arc<dyn DnsResolver>,
        proxy_config: InnerConnectionMode,
        #[cfg(target_os = "android")] socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
        #[cfg(any(feature = "api-override", test))] disable_tls: bool,
    ) -> Self {
        let abort_notify = Arc::new(tokio::sync::Notify::new());
        let connector = Arc::new(Mutex::new(HttpsConnectorInner {
            proxy_config,
            stream_handles: Default::default(),
            abort_notify: abort_notify.clone(),
        }));

        HttpsConnector {
            state: connector,
            abort_notify,
            dns_resolver,
            #[cfg(target_os = "android")]
            socket_bypass_tx,
            #[cfg(any(feature = "api-override", test))]
            disable_tls,
        }
    }

    /// Start listening for incoming [HttpsConnectorRequest]s from the created [HttpsConnectorHandle].
    pub fn spawn(&self) -> HttpsConnectorHandle {
        let state = self.state.clone();
        HttpsConnectorHandle { state }
    }

    /// Establishes a TCP connection with a peer at the specified socket address.
    ///
    /// Will timeout after [`CONNECT_TIMEOUT`] seconds.
    #[instrument(level = Level::TRACE, skip(socket_bypass_tx), ret)]
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
                tracing::error!("Failed to bypass socket, connection might fail");
            }
        }

        timeout(CONNECT_TIMEOUT, socket.connect(addr))
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::TimedOut, err))?
    }

    /// Resolve the provided `uri` to an IP and port. If the URI contains an IP, that IP will be
    /// used. Otherwise `dns_resolver` will be used as a fallback.
    /// If the URI contains a port, then that port will be used.
    #[instrument(level = Level::TRACE, skip(dns_resolver), ret)]
    async fn resolve_address(dns_resolver: &dyn DnsResolver, uri: Uri) -> io::Result<SocketAddr> {
        const DEFAULT_PORT: u16 = 443;

        let hostname = uri.host().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "invalid url, missing host")
        })?;
        let port = uri.port_u16();
        if let Ok(addr) = hostname.parse::<IpAddr>() {
            return Ok(SocketAddr::new(addr, port.unwrap_or(DEFAULT_PORT)));
        }

        let addrs = dns_resolver.resolve(hostname.to_owned()).await?;
        let addr = addrs
            .first()
            .ok_or_else(|| io::Error::other("Empty DNS response"))?;
        let port = match (addr.port(), port) {
            (_, Some(port)) => port,
            (0, None) => DEFAULT_PORT,
            (addr_port, None) => addr_port,
        };
        Ok(SocketAddr::new(addr.ip(), port))
    }
}

impl fmt::Debug for HttpsConnector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpsConnector").finish()
    }
}

impl Service<Uri> for HttpsConnector {
    type Response = TokioIo<AbortableStream<ApiConnection>>;
    type Error = io::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut inner = self.state.lock().unwrap();
        inner.stream_handles.retain(|handle| !handle.is_closed());
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let inner = self.state.clone();
        let abort_notify = self.abort_notify.clone();
        #[cfg(target_os = "android")]
        let socket_bypass_tx = self.socket_bypass_tx.clone();
        let dns_resolver = self.dns_resolver.clone();

        #[cfg(any(feature = "api-override", test))]
        let disable_tls = self.disable_tls;

        let span = trace_span!("HttpsConnector::call");
        span.record("uri", format_args!("{uri}"));

        let return_span = span.clone();
        let fut = async move {
            if uri.scheme() != Some(&Scheme::HTTPS) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid url, not https",
                ));
            }
            let Some(hostname) = uri.host().map(str::to_owned) else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid url, missing host",
                ));
            };
            let addr = Self::resolve_address(&*dns_resolver, uri).await?;

            // Loop until we have established a connection. This starts over if a new endpoint
            // is selected while connecting.
            let stream = loop {
                let should_abort = abort_notify.notified();
                let proxy_config = { inner.lock().unwrap().proxy_config.clone() };
                let stream_fut = proxy_config.connect(
                    &hostname,
                    &addr,
                    #[cfg(target_os = "android")]
                    socket_bypass_tx.clone(),
                    #[cfg(any(feature = "api-override", test))]
                    disable_tls,
                );

                // Wait for connection. Abort and retry if we switched to a different server.
                select! {
                    _ = should_abort.fuse() => continue,
                    stream = stream_fut.fuse() => break stream?,
                }
            };

            let (stream, socket_handle) = AbortableStream::new(stream);

            {
                let mut inner = inner.lock().unwrap();
                inner.stream_handles.push(socket_handle);
            }

            Ok(TokioIo::new(stream))
        }
        .inspect(move |result| {
            return_span.record("return", format_args!("{result:?}"));
        })
        .instrument(span);

        Box::pin(fut)
    }
}
