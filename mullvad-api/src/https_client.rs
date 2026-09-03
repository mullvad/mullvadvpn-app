use crate::{
    proxy::{ApiConnection, ApiConnectionMode, ProxyConfig},
    tls_stream::TlsStream,
};
use domain_fronting::client::ProxyConnection;
use futures::future;
#[cfg(target_os = "android")]
use futures::{channel::oneshot, mpsc, sink::SinkExt};
use hyper_util::rt::TokioIo;
use mullvad_encrypted_dns_proxy::Forwarder as EncryptedDNSForwarder;
use shadowsocks::{
    ServerConfig,
    config::{ServerConfigError, ServerType},
    context::Context as SsContext,
    crypto::CipherKind,
    relay::tcprelay::ProxyClientStream,
};
#[cfg(target_os = "android")]
use std::os::unix::io::{AsRawFd, RawFd};
use std::{
    future::Future,
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str,
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpSocket, TcpStream},
};
use tracing::{Level, instrument};

#[cfg(any(feature = "api-override", test))]
use crate::proxy::ConnectionDecorator;

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

/// A connector for TLS streams for use with HTTP clients. See [`HttpsConnector::connect`].
#[derive(Clone, Debug)]
pub struct HttpsConnector {
    connection_mode: ApiConnectionMode,
    #[cfg(target_os = "android")]
    socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
    #[cfg(any(feature = "api-override", test))]
    disable_tls: bool,
}

#[cfg(target_os = "android")]
pub type SocketBypassRequest = (RawFd, oneshot::Sender<()>);

impl HttpsConnector {
    /// Create a new [`HttpsConnector`].
    pub fn new(
        connection_mode: ApiConnectionMode,
        #[cfg(target_os = "android")] socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
        #[cfg(any(feature = "api-override", test))] disable_tls: bool,
    ) -> Self {
        HttpsConnector {
            connection_mode,
            #[cfg(target_os = "android")]
            socket_bypass_tx,
            #[cfg(any(feature = "api-override", test))]
            disable_tls,
        }
    }

    /// Change the proxy settings for the connector
    pub fn set_connection_mode(&mut self, mode: ApiConnectionMode) {
        self.connection_mode = mode;
    }

    /// Get the idle timeout of the curren connection method.
    pub fn idle_timeout(&self) -> Duration {
        let default = Duration::from_secs(60);
        match &self.connection_mode {
            ApiConnectionMode::Proxied(ProxyConfig::DomainFronting(..)) => {
                crate::domain_fronting::IDLE_TIMEOUT
            }
            ApiConnectionMode::Proxied(..) => default,
            ApiConnectionMode::Direct => default,
        }
    }

    /// Establish a TLS [`ApiConnection`] to `addr`, with HTTP host `host`.
    ///
    /// The connection will use the [`ApiConnectionMode`] specified in [`Self::new`] or
    /// [`Self::set_connection_mode`].
    pub async fn connect(
        &mut self,
        addr: SocketAddr,
        host: &str,
    ) -> io::Result<TokioIo<ApiConnection>> {
        struct Proxyer<'a> {
            host: &'a str,
            #[cfg(target_os = "android")]
            socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
            #[cfg(any(feature = "api-override", test))]
            disable_tls: bool,
        }

        impl Proxyer<'_> {
            /// Create an [`ApiConnection`] from a [`TcpStream`].
            ///
            /// The `make_proxy_stream` closure receives a [`TcpStream`] and produces a
            /// stream which can send to and receive data from some server using any
            /// or no proxy protocol.
            async fn connect_proxied<ProxyFactory, ProxyFuture, Proxy>(
                self,
                first_hop: SocketAddr,
                make_proxy_stream: ProxyFactory,
            ) -> io::Result<ApiConnection>
            where
                ProxyFactory: FnOnce(TcpStream) -> ProxyFuture,
                ProxyFuture: Future<Output = io::Result<Proxy>>,
                Proxy: AsyncRead + AsyncWrite + Unpin + Send + 'static,
            {
                let Proxyer {
                    host,
                    #[cfg(target_os = "android")]
                    socket_bypass_tx,
                    #[cfg(any(feature = "api-override", test))]
                    disable_tls,
                } = self;

                let socket = open_socket(
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

                let tls_stream = TlsStream::connect_https(proxy, host).await?;
                Ok(ApiConnection::new(Box::new(tls_stream)))
            }
        }

        let proxyer = Proxyer {
            host,
            #[cfg(target_os = "android")]
            socket_bypass_tx: self.socket_bypass_tx,
            #[cfg(any(feature = "api-override", test))]
            disable_tls: self.disable_tls,
        };

        let stream = match &self.connection_mode {
            // Set up a TCP-socket connection.
            ApiConnectionMode::Direct => {
                let first_hop = addr;
                let make_proxy_stream = |tcp_stream| async { Ok(tcp_stream) };
                proxyer
                    .connect_proxied(first_hop, make_proxy_stream)
                    .await?
            }
            // Set up a Shadowsocks-connection.
            ApiConnectionMode::Proxied(ProxyConfig::Shadowsocks(shadowsocks)) => {
                let first_hop = shadowsocks.endpoint;
                let proxy_context = SsContext::new_shared(ServerType::Local);
                let make_proxy_stream = |tcp_stream| async {
                    Ok(ProxyClientStream::from_stream(
                        proxy_context,
                        tcp_stream,
                        &ServerConfig::new(
                            shadowsocks.endpoint,
                            shadowsocks.plaintext_password(),
                            shadowsocks.cipher.clone().kind(),
                        )
                        .map_err(|_| std::io::Error::other("Invalid shadowsocks config"))?,
                        addr,
                    ))
                };
                proxyer
                    .connect_proxied(first_hop, make_proxy_stream)
                    .await?
            }
            // Set up a SOCKS5-connection.
            ApiConnectionMode::Proxied(ProxyConfig::Socks5Local(config)) => {
                let first_hop =
                    SocketAddr::new(IpAddr::from(Ipv4Addr::LOCALHOST), config.local_port);
                let make_proxy_stream = |tcp_stream| async {
                    tokio_socks::tcp::Socks5Stream::connect_with_socket(tcp_stream, addr)
                        .await
                        .map_err(|error| io::Error::other(format!("SOCKS error: {error}")))
                };
                proxyer
                    .connect_proxied(first_hop, make_proxy_stream)
                    .await?
            }
            ApiConnectionMode::Proxied(ProxyConfig::Socks5Remote(socks)) => {
                let first_hop = socks.endpoint;
                let make_proxy_stream = |tcp_stream| async {
                    match &socks.auth {
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
                proxyer
                    .connect_proxied(first_hop, make_proxy_stream)
                    .await?
            }
            ApiConnectionMode::Proxied(ProxyConfig::EncryptedDnsProxy(proxy_config)) => {
                let first_hop = SocketAddr::V4(proxy_config.addr);
                let make_proxy_stream = |tcp_stream| async {
                    let forwarder = EncryptedDNSForwarder::from_stream(proxy_config, tcp_stream);
                    Ok(forwarder)
                };
                proxyer
                    .connect_proxied(first_hop, make_proxy_stream)
                    .await?
            }
            ApiConnectionMode::Proxied(ProxyConfig::DomainFronting(config)) => {
                let use_http2 = crate::domain_fronting::USE_HTTP2;
                let domain_fronting = &config.config();
                let connect_tls = async || {
                    let tcp_stream = open_socket(
                        config.addr,
                        #[cfg(target_os = "android")]
                        self.socket_bypass_tx.clone(),
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
                if self.disable_tls {
                    return Ok(TokioIo::new(ApiConnection::new(Box::new(
                        ConnectionDecorator(proxy),
                    ))));
                }

                let tls_stream = TlsStream::connect_https(proxy, host).await?;
                ApiConnection::new(Box::new(tls_stream))
            }
        };

        Ok(TokioIo::new(stream))
    }
}

/// Establishes a TCP connection with a peer at the specified socket address.
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

    socket.connect(addr).await
}
