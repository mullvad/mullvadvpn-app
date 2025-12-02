//! A module for a POC of domain fronting. See IOS-1316.
//! This only compiles with the `domain-fronting` feature flag for the time being.

use std::{io::Error, net::SocketAddr, sync::Arc};

use http_body_util::Empty;
use hyper::{body::Bytes, upgrade::Upgraded};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use tokio_rustls::rustls::{self};
use webpki_roots::TLS_SERVER_ROOTS;

use crate::{DefaultDnsResolver, DnsResolver, tls_stream::TlsStream};

pub struct DomainFronting {
    /// Domain that will be used to connect to a CDN, used for SNI
    front: String,
    /// Host that will be reached via the CDN, i.e. this is the Host header value
    proxy_host: String,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug, Clone)]
pub struct ProxyConfig {
    pub addr: SocketAddr,
    front: String,
    proxy_host: String,
}

impl DomainFronting {
    pub fn new(front: String, proxy_host: String) -> Self {
        DomainFronting { front, proxy_host }
    }

    pub async fn proxy_config(&self) -> Result<ProxyConfig, Box<dyn std::error::Error>> {
        let dns_resolver = DefaultDnsResolver;

        let addrs = dns_resolver.resolve(self.front.clone()).await?;
        let addr = addrs
            .first()
            .ok_or_else(|| Error::other("Empty DNS response"))?;

        Ok(ProxyConfig {
            addr: *addr,
            front: self.front.clone(),
            proxy_host: self.proxy_host.clone(),
        })
    }
}

impl ProxyConfig {
    pub async fn connect_with_socket(
        &self,
        tcp_stream: TcpStream,
    ) -> anyhow::Result<TokioIo<Upgraded>> {
        let config = Arc::new(
            rustls::ClientConfig::builder()
                .with_root_certificates(read_cert_store())
                .with_no_client_auth(),
        );

        let front = self.front.clone();

        let io = TokioIo::new(
            TlsStream::connect_https_with_client_config(tcp_stream, &front, config).await?,
        );

        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.with_upgrades().await {
                log::trace!("Domain fronting connection failed: {:?}", err);
            }
        });

        let request = hyper::Request::connect(&format!("https://{}/", self.proxy_host))
            .body(Empty::<Bytes>::new())?;

        sender.ready().await?;
        let response = sender.send_request(request).await?;

        let upgraded_connection = hyper::upgrade::on(response).await?;
        Ok(TokioIo::new(upgraded_connection))
    }
}

fn read_cert_store() -> rustls::RootCertStore {
    let mut cert_store = rustls::RootCertStore::empty();

    cert_store.extend(TLS_SERVER_ROOTS.iter().cloned());
    cert_store
}
