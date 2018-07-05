extern crate tokio_openssl;
extern crate tokio_service;

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::str;
use std::sync::Arc;

use futures::{Future, Poll};
use hyper::client::{Client, Connect, HttpConnector};
use hyper::{Body, Uri};
pub use hyper_openssl::openssl::error::ErrorStack;
use hyper_openssl::openssl::ssl::{SslConnector, SslMethod};
use jsonrpc_client_http::ClientCreator;
use tokio_core::reactor::Handle;

use self::tokio_openssl::{SslConnectorExt, SslStream};
use self::tokio_service::Service;

pub struct HttpsClientWithSni {
    sni_hostname: String,
    ca_path: Box<Path>,
}

impl HttpsClientWithSni {
    pub fn new<P: Into<PathBuf>>(sni_hostname: String, ca_path: P) -> Self {
        HttpsClientWithSni {
            sni_hostname,
            ca_path: ca_path.into().into_boxed_path(),
        }
    }
}

impl ClientCreator for HttpsClientWithSni {
    type Connect = HttpsConnectorWithSni<HttpConnector>;
    type Error = ErrorStack;

    fn create(&self, handle: &Handle) -> Result<Client<Self::Connect, Body>, Self::Error> {
        let mut connector = HttpsConnectorWithSni::new(&self.ca_path, handle)?;
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
    tls: Arc<SslConnector>,
}

impl HttpsConnectorWithSni<HttpConnector> {
    /// Construct a new HttpsConnectorWithSni.
    ///
    /// Takes number of DNS worker threads.
    ///
    /// This uses hyper's default `HttpConnector`, and default `TlsConnector`.
    /// If you wish to use something besides the defaults, use `From::from`.
    pub fn new<P: AsRef<Path>>(ca_path: P, handle: &Handle) -> Result<Self, ErrorStack> {
        let mut http = HttpConnector::new(::DNS_THREADS, handle);
        http.enforce_http(false);
        let mut ssl_builder = SslConnector::builder(SslMethod::tls())?;
        ssl_builder.set_ca_file(ca_path)?;
        let ssl = ssl_builder.build();

        Ok(HttpsConnectorWithSni::from((http, ssl)))
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
    pub fn set_sni_hostname(&mut self, hostname: Option<String>) {
        self.sni_hostname = hostname;
    }
}

impl<T> From<(T, SslConnector)> for HttpsConnectorWithSni<T> {
    fn from(args: (T, SslConnector)) -> HttpsConnectorWithSni<T> {
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
    type Response = SslStream<T::Output>;
    type Error = io::Error;
    type Future = HttpsConnecting<T::Output>;

    fn call(&self, uri: Uri) -> Self::Future {
        if uri.scheme() != Some("https") {
            return HttpsConnecting(Box::new(::futures::future::err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid url, not https",
            ))));
        }
        let maybe_host = self
            .sni_hostname
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

        let fut = connecting.and_then(move |tcp| {
            tls.connect_async(&host, tcp)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        });
        HttpsConnecting(Box::new(fut))
    }
}

type BoxedFut<T> = Box<Future<Item = SslStream<T>, Error = io::Error>>;

/// A Future representing work to connect to a URL, and a TLS handshake.
pub struct HttpsConnecting<T>(BoxedFut<T>);


impl<T> Future for HttpsConnecting<T> {
    type Item = SslStream<T>;
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
