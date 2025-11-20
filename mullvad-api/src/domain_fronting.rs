//! A module for a POC of domain fronting. See IOS-1316.
//! This only compiles on macos for the time being.

use std::{io::Error, sync::Arc};

use tokio::net::TcpStream;
use tokio_rustls::rustls::{self};

use crate::{DefaultDnsResolver, DnsResolver, tls_stream::TlsStream};

pub struct DomainFronting {
    front: String,
}

impl DomainFronting {
    pub fn new(front: String) -> Self {
        DomainFronting { front }
    }

    pub async fn connect(&self) -> Result<TlsStream<TcpStream>, Box<dyn std::error::Error>> {
        let cert_store = read_cert_store();

        let config = Arc::new(
            rustls::ClientConfig::builder()
                .with_root_certificates(cert_store)
                .with_no_client_auth(),
        );

        let dns_resolver = DefaultDnsResolver;

        let addrs = dns_resolver.resolve(self.front.clone()).await?;
        let addr = addrs
            .first()
            .ok_or_else(|| Error::other("Empty DNS response"))?;
        log::debug!(
            "Resolved addresses {:?} for {:?}, will connect to {:?}",
            addrs.clone(),
            self.front,
            addr.ip()
        );
        let stream = TcpStream::connect((addr.ip(), 443)).await?;

        Ok(TlsStream::connect_https(stream, &self.front, config).await?)
    }
}

fn read_cert_store() -> rustls::RootCertStore {
    let mut cert_store = rustls::RootCertStore::empty();

    //FIXME: This does not build on iOS yet, it will be figured out later
    let root_certificates =
        rustls_native_certs::load_native_certs().expect("Could not load platform certs");
    for cert in root_certificates {
        cert_store.add(cert).unwrap();
    }

    cert_store
}
