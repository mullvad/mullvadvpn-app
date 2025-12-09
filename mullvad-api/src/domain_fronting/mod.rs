//! A module for a POC of domain fronting. See IOS-1316.
//! This only compiles with the `domain-fronting` feature flag for the time being.

use std::{
    io::{self, Error},
    net::SocketAddr,
    sync::Arc,
    task::{Poll, ready},
};

use http::{
    Request, Response,
    header::{self, UPGRADE},
    status::StatusCode,
};
use http_body_util::{Empty, Full};
use hyper::{
    body::{Bytes, Incoming},
    client::conn::http1::SendRequest,
    upgrade::Upgraded,
};
use hyper_util::rt::TokioIo;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_rustls::rustls::{self};
use uuid::Uuid;
use webpki_roots::TLS_SERVER_ROOTS;

use crate::{DefaultDnsResolver, DnsResolver, tls_stream::TlsStream};

pub mod server;

const SESSION_HEADER_KEY: &str = "X-Mullvad-Session";

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
            .ok_or_else(|| io::Error::other("Empty DNS response"))?;

        Ok(ProxyConfig {
            addr: SocketAddr::new(addr.ip(), 443),
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
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::CONTENT_LENGTH, "0")
            .body(Full::<Bytes>::new(Bytes::new()))?;

        sender.ready().await?;
        let response = sender.send_request(request).await?;

        unimplemented!()
    }
}

struct ProxyConnection {
    sender: SendRequest<Full<Bytes>>,
    proxy_host: String,
    session_id: Uuid,
    request: Option<Box<dyn Future<Output = io::Result<Vec<u8>>>>>,
    last_response: Option<Vec<u8>>,
    only_reading: bool,
}

impl ProxyConnection {
    async fn initialize(
        mut sender: SendRequest<Full<Bytes>>,
        proxy_host: String,
    ) -> anyhow::Result<Self> {
        sender.ready().await?;
        let response = sender
            .send_request(Self::initial_request(&proxy_host))
            .await?;
        let session_header = response
            .headers()
            .get(SESSION_HEADER_KEY)
            .ok_or(anyhow::bail!(
                "Proxy server didn't include session ID inresponse"
            ))?;
        let session_id = Uuid::try_parse_ascii(session_header.as_bytes())?;

        Ok(Self {
            sender,
            proxy_host,
            session_id,
            last_response: None,
            only_reading: false,
            request: None,
        })
    }

    fn initial_request(proxy_host: &str) -> Request<Full<Bytes>> {
        hyper::Request::connect(&format!("https://{}/", proxy_host))
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::CONTENT_LENGTH, "0")
            .body(Full::<Bytes>::new(Bytes::new()))
            .unwrap()
    }

    fn create_request(
        &mut self,
        buffer: Option<&[u8]>,
    ) -> Box<dyn Future<Output = io::Result<Vec<u8>>>> {
        let bytes = buffer
            .as_ref()
            .map(|buffer| Bytes::copy_from_slice(*buffer))
            .unwrap_or(Bytes::new());
        let body = Full::new(bytes);

        let mut request = hyper::Request::connect(&format!("https://{}/", self.proxy_host));
        if buffer.is_some() {
            request = request
                .header(header::CONTENT_TYPE, "application/octet-stream")
                .header(header::CONTENT_LENGTH, &format!("{}", bytes.len()));
        }
        let request = request.body(body).unwrap();

        let request_future = self.sender.send(request);

        let future = async move {
            let response = request_future.await?;
            if response.status() != StatusCode::OK && response.status() != StatusCode::CREATED {
                return Err(io::Error::other(format!(
                    "Unexpected response status code: {}",
                    response.status()
                )));
            };
            let body = response.collect().map_err(io::Error::other)?;

            Ok(body.to_bytes().to_vec())
        };

        Box::new(request_future)
    }
}

impl AsyncRead for ProxyConnection {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if self.request.is_none() {
            self.only_reading = true;
            self.request = Some(self.create_request(None));
        }
        let response = ready!(self.request.poll_ready(cx));
        self.only_reading = false;

        todo!()
    }
}

impl AsyncWrite for ProxyConnection {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        // if there's a request in flight, then don't bother writing anything.
        if self.request.is_some() && self.only_reading {
            return Poll::Pending;
        }

        let requst = match self.request.as_mut() {
            Some(request) => request,
            None => {
                ready!(self.sender.poll_ready(cx))?;
                self.request = Some(self.get_mut().create_future(buf));
                return Poll::Ready(Ok(buf.len()));
            }
        };

        let response = ready!(self.request.poll_ready(cx))?;

        if !response.empty() {
            self.last_response = Some(response);
        }
        return Poll::Pending;
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}

fn read_cert_store() -> rustls::RootCertStore {
    let mut cert_store = rustls::RootCertStore::empty();

    cert_store.extend(TLS_SERVER_ROOTS.iter().cloned());
    cert_store
}
