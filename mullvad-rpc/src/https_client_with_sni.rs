use http::uri::Scheme;
use hyper::{client::HttpConnector, service::Service, Uri};
use hyper_rustls::MaybeHttpsStream;
use std::{
    fmt,
    future::Future,
    io::{self, BufReader},
    pin::Pin,
    str,
    sync::Arc,
    task::{Context, Poll},
};
use tokio_rustls::rustls;
use webpki::DNSNameRef;

// Old LetsEncrypt root certificate
const OLD_ROOT_CERT: &[u8] = include_bytes!("../old_le_root_cert.pem");

/// A Connector for the `https` scheme.
#[derive(Clone)]
pub struct HttpsConnectorWithSni {
    sni_hostname: Option<String>,
    http: HttpConnector,
    tls: Arc<rustls::ClientConfig>,
}

impl HttpsConnectorWithSni {
    /// Construct a new HttpsConnectorWithSni.
    ///
    /// Takes number of DNS worker threads.
    ///
    /// This uses hyper's default `HttpConnector`, and default `TlsConnector`.
    /// If you wish to use something besides the defaults, use `From::from`.
    pub fn new() -> Self {
        let mut http = HttpConnector::new();
        http.enforce_http(false);

        let mut config = rustls::ClientConfig::new();
        config.enable_sni = true;
        config.root_store = Self::read_cert_store();

        HttpsConnectorWithSni::from((http, config))
    }

    fn read_cert_store() -> rustls::RootCertStore {
        let mut cert_store = rustls::RootCertStore::empty();

        let (num_certs_added, num_failures) = cert_store
            .add_pem_file(&mut BufReader::new(OLD_ROOT_CERT))
            .expect("Failed to add old root cert");
        if num_failures > 0 || num_certs_added != 1 {
            panic!("Failed to add old root cert");
        }

        cert_store
    }


    /// Configure a hostname to use with SNI.
    ///
    /// Configures the TLS connection handshake to request a certificate for a given domain,
    /// instead of the domain obtained from the URI. Use `None` to use the domain from the URI.
    pub fn set_sni_hostname(&mut self, hostname: Option<String>) {
        self.sni_hostname = hostname;
    }
}

impl From<(HttpConnector, rustls::ClientConfig)> for HttpsConnectorWithSni {
    fn from(args: (HttpConnector, rustls::ClientConfig)) -> HttpsConnectorWithSni {
        HttpsConnectorWithSni {
            sni_hostname: None,
            http: args.0,
            tls: Arc::new(args.1),
        }
    }
}

impl fmt::Debug for HttpsConnectorWithSni {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpsConnectorWithSni").finish()
    }
}

impl Service<Uri> for HttpsConnectorWithSni {
    type Response = MaybeHttpsStream<tokio::net::TcpStream>;
    type Error = io::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let tls_connector: tokio_rustls::TlsConnector = self.tls.clone().into();
        let mut http = self.http.clone();
        let sni_hostname = self
            .sni_hostname
            .clone()
            .or_else(|| uri.host().map(str::to_owned))
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "invalid url, missing host")
            });


        let fut = async move {
            if uri.scheme() != Some(&Scheme::HTTPS) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid url, not https",
                ));
            }
            let hostname = sni_hostname?;
            let host = DNSNameRef::try_from_ascii_str(&hostname)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid hostname"))?;
            let connection = http
                .call(uri)
                .await
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
            let tls_connection = tls_connector.connect(host, connection).await?;

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
