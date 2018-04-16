extern crate tokio_service;
extern crate tokio_tls;

use std::fmt;
use std::io;
use std::str;
use std::sync::Arc;

use futures::{Future, Poll};
use hyper::client::{Client, Connect, HttpConnector};
use hyper::{Body, Uri};
use hyper_tls::MaybeHttpsStream;
use jsonrpc_client_http::ClientCreator;
pub use native_tls::Error;
use native_tls::TlsConnector;
use tokio_core::reactor::Handle;

use self::tokio_service::Service;
use self::tokio_tls::TlsConnectorExt;

/// Number of threads in the thread pool doing DNS resolutions.
/// Since DNS is resolved via blocking syscall they must be run on separate threads.
static DNS_THREADS: usize = 2;

pub struct HttpsClientWithSni {
    sni_hostname: String,
}

impl HttpsClientWithSni {
    pub fn new(sni_hostname: String) -> Self {
        HttpsClientWithSni { sni_hostname }
    }
}

impl ClientCreator for HttpsClientWithSni {
    type Connect = HttpsConnectorWithSni<HttpConnector>;
    type Error = Error;

    fn create(&self, handle: &Handle) -> Result<Client<Self::Connect, Body>, Self::Error> {
        let mut connector = HttpsConnectorWithSni::new(DNS_THREADS, handle)?;
        connector.set_sni_hostname(Some(self.sni_hostname.clone()));
        let client = Client::configure().connector(connector).build(handle);
        Ok(client)
    }
}

/// A Connector for the `https` scheme.
#[derive(Clone)]
pub struct HttpsConnectorWithSni<T> {
    sni_hostname: Option<String>,
    http: T,
    tls: Arc<TlsConnector>,
}

impl HttpsConnectorWithSni<HttpConnector> {
    /// Construct a new HttpsConnectorWithSni.
    ///
    /// Takes number of DNS worker threads.
    ///
    /// This uses hyper's default `HttpConnector`, and default `TlsConnector`.
    /// If you wish to use something besides the defaults, use `From::from`.
    fn new(threads: usize, handle: &Handle) -> Result<Self, Error> {
        let mut http = HttpConnector::new(threads, handle);
        http.enforce_http(false);
        let tls = TlsConnector::builder()?.build()?;
        Ok(HttpsConnectorWithSni::from((http, tls)))
    }
}

impl<T> HttpsConnectorWithSni<T>
where
    T: Connect,
{
    /// Configure a hostname to use with SNI.
    ///
    /// Configures the TLS connection handshake to request a certificate for a given domain,
    /// instead of the domain obtained from the URI. Use `None` to use the domain from the URI.
    fn set_sni_hostname(&mut self, hostname: Option<String>) {
        self.sni_hostname = hostname;
    }
}

impl<T> From<(T, TlsConnector)> for HttpsConnectorWithSni<T> {
    fn from(args: (T, TlsConnector)) -> HttpsConnectorWithSni<T> {
        HttpsConnectorWithSni {
            sni_hostname: None,
            http: args.0,
            tls: Arc::new(args.1),
        }
    }
}

impl<T> fmt::Debug for HttpsConnectorWithSni<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HttpsConnectorWithSni").finish()
    }
}

impl<T: Connect> Service for HttpsConnectorWithSni<T> {
    type Request = Uri;
    type Response = MaybeHttpsStream<T::Output>;
    type Error = io::Error;
    type Future = HttpsConnecting<T::Output>;

    fn call(&self, uri: Uri) -> Self::Future {
        let is_https = uri.scheme() == Some("https");
        let maybe_host = self.sni_hostname
            .as_ref()
            .map(String::as_str)
            .or_else(|| uri.host())
            .map(str::to_owned);
        let host = match maybe_host {
            Some(host) => host,
            None => {
                return HttpsConnecting(Box::new(::futures::future::err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid url, missing host",
                ))));
            }
        };
        let connecting = self.http.connect(uri);
        let tls = self.tls.clone();

        let fut: BoxedFut<T::Output> = if is_https {
            let fut = connecting.and_then(move |tcp| {
                tls.connect_async(&host, tcp)
                    .map(|conn| MaybeHttpsStream::Https(conn))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            });
            Box::new(fut)
        } else {
            Box::new(connecting.map(|tcp| MaybeHttpsStream::Http(tcp)))
        };
        HttpsConnecting(fut)
    }
}

type BoxedFut<T> = Box<Future<Item = MaybeHttpsStream<T>, Error = io::Error>>;

/// A Future representing work to connect to a URL, and a TLS handshake.
pub struct HttpsConnecting<T>(BoxedFut<T>);


impl<T> Future for HttpsConnecting<T> {
    type Item = MaybeHttpsStream<T>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}

impl<T> fmt::Debug for HttpsConnecting<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("HttpsConnecting")
    }
}
