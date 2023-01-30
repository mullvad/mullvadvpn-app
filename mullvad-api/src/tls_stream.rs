//! Provides a TLS 1.3 stream with SNI and LE root cert only.
use std::{
    io::{self, ErrorKind},
    pin::Pin,
    sync::Arc,
    task::{self, Poll},
};

use hyper::client::connect::{Connected, Connection};
use once_cell::sync::Lazy;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_rustls::{
    rustls::{self, ClientConfig, ServerName},
    TlsConnector,
};

const LE_ROOT_CERT: &[u8] = include_bytes!("../le_root_cert.pem");

pub struct TlsStream<S: AsyncRead + AsyncWrite + Unpin> {
    stream: tokio_rustls::client::TlsStream<S>,
}

impl<S> TlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn connect_https(stream: S, domain: &str) -> io::Result<TlsStream<S>> {
        static TLS_CONFIG: Lazy<Arc<ClientConfig>> = Lazy::new(|| {
            let config = ClientConfig::builder()
                .with_safe_default_cipher_suites()
                .with_safe_default_kx_groups()
                .with_protocol_versions(&[&rustls::version::TLS13])
                .unwrap()
                .with_root_certificates(read_cert_store())
                .with_no_client_auth();
            Arc::new(config)
        });

        let connector = TlsConnector::from(TLS_CONFIG.clone());

        let host = match ServerName::try_from(domain) {
            Ok(n) => n,
            Err(_) => {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("invalid hostname \"{domain}\""),
                ));
            }
        };

        let stream = connector.connect(host, stream).await?;

        Ok(TlsStream { stream })
    }
}

fn read_cert_store() -> rustls::RootCertStore {
    let mut cert_store = rustls::RootCertStore::empty();

    let certs = rustls_pemfile::certs(&mut std::io::BufReader::new(LE_ROOT_CERT))
        .expect("Failed to parse pem file");
    let (num_certs_added, num_failures) = cert_store.add_parsable_certificates(&certs);
    if num_failures > 0 || num_certs_added != 1 {
        panic!("Failed to add root cert");
    }

    cert_store
}

impl<S> AsyncRead for TlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl<S> AsyncWrite for TlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

impl<S> Connection for TlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cert_loading() {
        let _certs = read_cert_store();
    }
}
