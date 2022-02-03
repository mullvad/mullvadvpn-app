use crate::{
    abortable_stream::{AbortableStream, AbortableStreamHandle},
    proxy::{ApiConnectionMode, MaybeProxyStream, ProxyConfig},
    tls_stream::TlsStream,
};
use futures::{channel::mpsc, future, StreamExt};
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
    crypto::v1::CipherKind,
    relay::tcprelay::ProxyClientStream,
    ServerAddr, ServerConfig,
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
#[cfg(target_os = "android")]
use tokio::net::TcpSocket;

use tokio::{net::TcpStream, runtime::Handle, time::timeout};

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

    /// Stop all streams produced by this connector
    pub fn set_proxy(&self, proxy: ApiConnectionMode) {
        let _ = self
            .tx
            .unbounded_send(HttpsConnectorRequest::SetProxy(proxy));
    }
}

enum HttpsConnectorRequest {
    Reset,
    SetProxy(ApiConnectionMode),
}

#[derive(Clone)]
enum InnerProxyConfig {
    /// Connect directly to the target.
    Direct,
    /// Connect to the destination via a proxy.
    Proxied(ServerConfig),
}

#[derive(err_derive::Error, Debug)]
enum ProxyConfigError {
    #[error(display = "Unrecognized cipher selected: {}", _0)]
    InvalidCipher(String),
}

impl TryFrom<ApiConnectionMode> for InnerProxyConfig {
    type Error = ProxyConfigError;

    fn try_from(config: ApiConnectionMode) -> Result<Self, Self::Error> {
        Ok(match config {
            ApiConnectionMode::Direct => InnerProxyConfig::Direct,
            ApiConnectionMode::Proxied(ProxyConfig::Shadowsocks(config)) => {
                InnerProxyConfig::Proxied(ServerConfig::new(
                    ServerAddr::SocketAddr(config.peer),
                    config.password,
                    CipherKind::from_str(&config.cipher)
                        .map_err(|_| ProxyConfigError::InvalidCipher(config.cipher))?,
                ))
            }
        })
    }
}

/// A Connector for the `https` scheme.
#[derive(Clone)]
pub struct HttpsConnectorWithSni {
    inner: Arc<Mutex<HttpsConnectorWithSniInner>>,
    sni_hostname: Option<String>,
    abort_notify: Arc<tokio::sync::Notify>,
    proxy_context: SharedContext,
    #[cfg(target_os = "android")]
    socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
}

struct HttpsConnectorWithSniInner {
    stream_handles: Vec<AbortableStreamHandle>,
    proxy_config: InnerProxyConfig,
}

#[cfg(target_os = "android")]
pub type SocketBypassRequest = (RawFd, oneshot::Sender<()>);

impl HttpsConnectorWithSni {
    pub fn new(
        handle: Handle,
        sni_hostname: Option<String>,
        #[cfg(target_os = "android")] socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
    ) -> (Self, HttpsConnectorWithSniHandle) {
        let (tx, mut rx) = mpsc::unbounded();
        let abort_notify = Arc::new(tokio::sync::Notify::new());
        let inner = Arc::new(Mutex::new(HttpsConnectorWithSniInner {
            stream_handles: vec![],
            proxy_config: InnerProxyConfig::Direct,
        }));

        let inner_copy = inner.clone();
        let notify = abort_notify.clone();
        handle.spawn(async move {
            // Handle requests by `HttpsConnectorWithSniHandle`s
            while let Some(request) = rx.next().await {
                let handles = {
                    let mut inner = inner_copy.lock().unwrap();

                    if let HttpsConnectorRequest::SetProxy(config) = request {
                        match InnerProxyConfig::try_from(config) {
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
                abort_notify,
                proxy_context: SsContext::new_shared(ServerType::Local),
                #[cfg(target_os = "android")]
                socket_bypass_tx,
            },
            HttpsConnectorWithSniHandle { tx },
        )
    }

    #[cfg(not(target_os = "android"))]
    async fn open_socket(addr: SocketAddr) -> std::io::Result<TcpStream> {
        timeout(CONNECT_TIMEOUT, TcpStream::connect(addr))
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::TimedOut, err))?
    }

    #[cfg(target_os = "android")]
    async fn open_socket(
        addr: SocketAddr,
        socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
    ) -> std::io::Result<TcpStream> {
        let socket = match addr {
            SocketAddr::V4(_) => TcpSocket::new_v4()?,
            SocketAddr::V6(_) => TcpSocket::new_v6()?,
        };

        if let Some(mut tx) = socket_bypass_tx {
            let (done_tx, done_rx) = oneshot::channel();
            let _ = tx.send((socket.as_raw_fd(), done_tx)).await;
            if let Err(_) = done_rx.await {
                log::error!("Failed to bypass socket, connection might fail");
            }
        }

        timeout(CONNECT_TIMEOUT, socket.connect(addr))
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::TimedOut, err))?
    }

    async fn resolve_address(uri: &Uri) -> io::Result<SocketAddr> {
        let hostname = uri.host().ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid url, missing host",
        ))?;
        let port = uri.port_u16().unwrap_or(443);

        if let Some(addr) = hostname.parse::<IpAddr>().ok() {
            return Ok(SocketAddr::new(addr, port));
        }

        let mut addrs = GaiResolver::new()
            .call(
                Name::from_str(&hostname)
                    .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?,
            )
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        let addr = addrs
            .next()
            .ok_or(io::Error::new(io::ErrorKind::Other, "Empty DNS response"))?;
        Ok(SocketAddr::new(addr.ip(), port))
    }
}

impl fmt::Debug for HttpsConnectorWithSni {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpsConnectorWithSni").finish()
    }
}

impl Service<Uri> for HttpsConnectorWithSni {
    type Response = AbortableStream<MaybeProxyStream>;
    type Error = io::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
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
        let proxy_context = self.proxy_context.clone();
        #[cfg(target_os = "android")]
        let socket_bypass_tx = self.socket_bypass_tx.clone();

        let fut = async move {
            if uri.scheme() != Some(&Scheme::HTTPS) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid url, not https",
                ));
            }

            let hostname = sni_hostname?;
            let addr = Self::resolve_address(&uri).await?;

            // Loop until we have established a connection. This starts over if a new endpoint
            // is selected while connecting.
            let stream = loop {
                let config = { inner.lock().unwrap().proxy_config.clone() };
                let hostname_copy = hostname.clone();
                let addr_copy = addr.clone();
                let context = proxy_context.clone();
                #[cfg(target_os = "android")]
                let socket_bypass_tx_copy = socket_bypass_tx.clone();

                let stream_fut: Pin<
                    Box<dyn Future<Output = Result<MaybeProxyStream, io::Error>> + Send>,
                > = Box::pin(async move {
                    match config {
                        InnerProxyConfig::Direct => {
                            let socket = Self::open_socket(
                                addr_copy,
                                #[cfg(target_os = "android")]
                                socket_bypass_tx_copy,
                            )
                            .await?;
                            let tls_stream =
                                TlsStream::connect_https(socket, &hostname_copy).await?;
                            Ok(MaybeProxyStream::Tls(tls_stream))
                        }
                        InnerProxyConfig::Proxied(proxy_config) => {
                            let proxy_addr = if let ServerAddr::SocketAddr(sockaddr) =
                                proxy_config.external_addr()
                            {
                                *sockaddr
                            } else {
                                return Err(io::Error::new(
                                    io::ErrorKind::InvalidInput,
                                    "proxy address must be socket address",
                                ));
                            };
                            let socket = Self::open_socket(
                                proxy_addr,
                                #[cfg(target_os = "android")]
                                socket_bypass_tx_copy,
                            )
                            .await?;
                            let proxy = ProxyClientStream::from_stream(
                                context,
                                socket,
                                &proxy_config,
                                addr,
                            );
                            let tls_stream =
                                TlsStream::connect_https(proxy, &hostname_copy).await?;
                            Ok(MaybeProxyStream::Proxied(tls_stream))
                        }
                    }
                });

                // Wait for connection. Abort and retry if we switched to a different server.
                if let future::Either::Left((stream, _)) =
                    future::select(stream_fut, Box::pin(abort_notify.notified())).await
                {
                    break stream?;
                }
            };

            let (stream, socket_handle) = AbortableStream::new(stream);

            {
                let mut inner = inner.lock().unwrap();
                inner.stream_handles.retain(|handle| !handle.is_closed());
                inner.stream_handles.push(socket_handle);
            }

            Ok(stream)
        };

        Box::pin(fut)
    }
}
