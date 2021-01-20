use crate::{rest::RequestCommand, tcp_stream::TcpStream};
use futures::{
    channel::{mpsc, oneshot},
    sink::SinkExt,
};
use http::uri::Scheme;
use hyper::{
    client::connect::dns::{GaiResolver, Name},
    service::Service,
    Uri,
};
use hyper_rustls::MaybeHttpsStream;
#[cfg(target_os = "android")]
use std::os::unix::io::{AsRawFd, RawFd};
use std::{
    fmt,
    future::Future,
    io::{self, BufReader},
    net::{IpAddr, SocketAddr},
    pin::Pin,
    str::{self, FromStr},
    sync::Arc,
    task::{Context, Poll},
};

use tokio::{net::TcpStream as TokioTcpStream, runtime::Handle};
use tokio_rustls::rustls;
use webpki::DNSNameRef;

// Old LetsEncrypt root certificate
const OLD_ROOT_CERT: &[u8] = include_bytes!("../old_le_root_cert.pem");
// New LetsEncrypt root certificate
const NEW_ROOT_CERT: &[u8] = include_bytes!("../new_le_root_cert.pem");

/// A Connector for the `https` scheme.
#[derive(Clone)]
pub struct HttpsConnectorWithSni {
    next_socket_id: usize,
    handle: Handle,
    sni_hostname: Option<String>,
    service_tx: Option<mpsc::Sender<RequestCommand>>,
    #[cfg(target_os = "android")]
    socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
    tls: Arc<rustls::ClientConfig>,
}

#[cfg(target_os = "android")]
pub type SocketBypassRequest = (RawFd, oneshot::Sender<()>);

impl HttpsConnectorWithSni {
    /// Construct a new HttpsConnectorWithSni.
    ///
    /// Takes number of DNS worker threads.
    ///
    /// This uses hyper's default `HttpConnector`, and default `TlsConnector`.
    /// If you wish to use something besides the defaults, use `From::from`.
    pub fn new(
        handle: Handle,
        sni_hostname: Option<String>,
        #[cfg(target_os = "android")] socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
    ) -> Self {
        let mut config = rustls::ClientConfig::new();
        config.enable_sni = true;
        config.root_store = Self::read_cert_store();

        HttpsConnectorWithSni {
            next_socket_id: 0,
            handle,
            sni_hostname,
            #[cfg(target_os = "android")]
            socket_bypass_tx,
            service_tx: None,
            tls: Arc::new(config),
        }
    }

    fn read_cert_store() -> rustls::RootCertStore {
        let mut cert_store = rustls::RootCertStore::empty();

        let (num_certs_added, num_failures) = cert_store
            .add_pem_file(&mut BufReader::new(OLD_ROOT_CERT))
            .expect("Failed to add old root cert");
        if num_failures > 0 || num_certs_added != 1 {
            panic!("Failed to add old root cert");
        }

        let (num_certs_added, num_failures) = cert_store
            .add_pem_file(&mut BufReader::new(NEW_ROOT_CERT))
            .expect("Failed to add new root cert");
        if num_failures > 0 || num_certs_added != 1 {
            panic!("Failed to add new root cert");
        }

        cert_store
    }


    /// Set a channel to register sockets with the request service.
    pub(crate) fn set_service_tx(&mut self, service_tx: mpsc::Sender<RequestCommand>) {
        self.service_tx = Some(service_tx);
    }

    fn next_id(&mut self) -> usize {
        let next_id = self.next_socket_id;
        self.next_socket_id = self.next_socket_id.wrapping_add(1);
        next_id
    }

    #[cfg(not(target_os = "android"))]
    async fn open_socket(addr: SocketAddr) -> std::io::Result<TokioTcpStream> {
        TokioTcpStream::connect(addr).await
    }

    #[cfg(target_os = "android")]
    async fn open_socket(
        addr: SocketAddr,
        socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
    ) -> std::io::Result<TokioTcpStream> {
        use socket2::{Domain, Protocol, Socket, Type};
        let domain = match addr {
            SocketAddr::V4(_) => Domain::ipv4(),
            SocketAddr::V6(_) => Domain::ipv6(),
        };
        let socket = Socket::new(domain, Type::stream(), Some(Protocol::tcp()))?.into_tcp_stream();

        if let Some(mut tx) = socket_bypass_tx {
            let (done_tx, done_rx) = oneshot::channel();
            let _ = tx.send((socket.as_raw_fd(), done_tx)).await;
            if let Err(_) = done_rx.await {
                log::error!("Failed to bypass socket, connection might fail");
            }
        }

        TokioTcpStream::connect_std(socket, &addr).await
    }

    async fn resolve_address(hostname: &str) -> io::Result<SocketAddr> {
        match Self::parse_addr(&hostname) {
            Some(addr) => Ok(addr),
            None => {
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
                Ok(SocketAddr::new(addr, 443))
            }
        }
    }


    fn parse_addr(hostname: &str) -> Option<SocketAddr> {
        if let Ok(addr) = hostname.parse::<SocketAddr>() {
            return Some(addr);
        }
        let ip = hostname.parse::<IpAddr>().ok()?;
        Some(SocketAddr::new(ip, 443))
    }
}

impl fmt::Debug for HttpsConnectorWithSni {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpsConnectorWithSni").finish()
    }
}


impl Service<Uri> for HttpsConnectorWithSni {
    type Response = MaybeHttpsStream<TcpStream>;
    type Error = io::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let tls_connector: tokio_rustls::TlsConnector = self.tls.clone().into();
        let sni_hostname = self
            .sni_hostname
            .clone()
            .or_else(|| uri.host().map(str::to_owned))
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "invalid url, missing host")
            });
        let service_tx = self.service_tx.clone();

        let socket_id = self.next_id();
        let handle = self.handle.clone();
        #[cfg(target_os = "android")]
        let socket_bypass_tx = self.socket_bypass_tx.clone();

        let fut = async move {
            if uri.scheme() != Some(&Scheme::HTTPS) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid url, not https",
                ));
            }
            let host_addr = uri.host().ok_or(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid url, missing host",
            ))?;
            let hostname = sni_hostname?;
            let host = DNSNameRef::try_from_ascii_str(&hostname)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid hostname"))?;
            let addr = Self::resolve_address(host_addr).await?;

            let tokio_connection = Self::open_socket(
                addr,
                #[cfg(target_os = "android")]
                socket_bypass_tx,
            )
            .await?;

            let (socket_shutdown_tx, socket_shutdown_rx) = oneshot::channel();


            let (tcp_stream, socket_handle) =
                TcpStream::new(tokio_connection, socket_id, Some(socket_shutdown_tx));
            if let Some(mut service_tx) = service_tx {
                if service_tx
                    .send(RequestCommand::SocketOpened(socket_id, socket_handle))
                    .await
                    .is_err()
                {
                    log::error!("Failed to submit new socket to request service");
                }
                handle.spawn(async move {
                    let _ = socket_shutdown_rx.await;
                    if service_tx
                        .send(RequestCommand::SocketClosed(socket_id))
                        .await
                        .is_err()
                    {
                        log::error!("Failed to send socket closure command to request service");
                    }
                });
            }


            let tls_connection = tls_connector.connect(host, tcp_stream).await?;

            Ok(MaybeHttpsStream::Https(tls_connection))
        };


        Box::pin(fut)
    }
}


#[cfg(test)]
mod test {
    use super::HttpsConnectorWithSni;

    #[test]
    fn test_cert_loading() {
        let _certs = HttpsConnectorWithSni::read_cert_store();
    }
}
