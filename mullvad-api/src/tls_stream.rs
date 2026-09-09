//! Provides a TLS stream to the Mullvad API. See [`mullvad_tls_client::api`]
//! for the parameters it is locked down to.
use std::{
    io::{self, ErrorKind},
    pin::Pin,
    sync::Arc,
    task::{self, Poll},
};

use hyper_util::client::legacy::connect::{Connected, Connection};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_rustls::{
    TlsConnector,
    rustls::{ClientConfig, pki_types::ServerName},
};

pub struct TlsStream<S: AsyncRead + AsyncWrite + Unpin> {
    stream: tokio_rustls::client::TlsStream<S>,
}

impl<S> TlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn connect_https(stream: S, domain: &str) -> io::Result<TlsStream<S>> {
        Self::connect_https_with_client_config(stream, domain, mullvad_tls_client::api()).await
    }

    pub async fn connect_https_with_client_config(
        stream: S,
        domain: &str,
        client_config: &ClientConfig,
    ) -> io::Result<TlsStream<S>> {
        let connector = TlsConnector::from(Arc::new(client_config.clone()));

        let host = match ServerName::try_from(domain.to_owned()) {
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
