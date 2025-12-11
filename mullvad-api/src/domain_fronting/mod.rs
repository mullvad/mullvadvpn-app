//! A module for a POC of domain fronting. See IOS-1316.
//! This only compiles with the `domain-fronting` feature flag for the time being.

use std::{
    io::{self, Error},
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Poll, ready},
};

use bytes::BufMut;
use http::{
    header, status::StatusCode, Request, Response
};
use http_body_util::{BodyExt, Empty, Full};
use hyper::{
    body::{Bytes, Incoming},
    client::conn::http1::SendRequest,
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
    pub async fn connect(&self) -> anyhow::Result<ProxyConnection> {
        let connection = TcpStream::connect(self.addr).await?;
        self.connect_with_socket(connection).await
    }

    pub async fn connect_with_socket(
        &self,
        tcp_stream: TcpStream,
    ) -> anyhow::Result<ProxyConnection> {
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
            if let Err(err) = conn.await {
                log::trace!("Domain fronting connection failed: {:?}", err);
            }
        });

        ProxyConnection::initialize(sender, self.proxy_host.clone()).await
    }
}

pub struct ProxyConnection {
    sender: SendRequest<Full<Bytes>>,
    proxy_host: String,
    session_id: Uuid,
    request: Option<Pin<Box<dyn Future<Output = io::Result<Vec<u8>>> + Send>>>,
    last_response: Option<Vec<u8>>,
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
        panic!("{:?}", response);

        let session_header = response
            .headers()
            .get(SESSION_HEADER_KEY)
            .ok_or(anyhow::anyhow!(
                "Proxy server didn't include session ID inresponse"
            ))?;
        let session_id = Uuid::try_parse_ascii(session_header.as_bytes())?;

        Ok(Self {
            sender,
            proxy_host,
            session_id,
            last_response: None,
            request: None,
        })
    }

    fn initial_request(proxy_host: &str) -> Request<Full<Bytes>> {
       let req=  hyper::Request::get(&format!("https://{}/hey", proxy_host))
            .header(header::USER_AGENT, "curl/8.14.1")
            .header(header::ACCEPT, "*/*")
            .body(Full::<Bytes>::new(Bytes::new()))
            .unwrap();
        dbg!(&req);
        req
    }

    fn create_request(
        &mut self,
        buffer: Option<&[u8]>,
    ) -> Pin<Box<dyn Future<Output = io::Result<Vec<u8>>> + 'static + Send>> {
        let bytes = buffer
            .as_ref()
            .map(|buffer| Bytes::copy_from_slice(*buffer))
            .unwrap_or(Bytes::new());
        let content_length = bytes.len();
        let body = Full::new(bytes);

        let mut request = hyper::Request::post(&format!("https://{}/", self.proxy_host));
        if buffer.is_some() {
            request = request
                .header(header::CONTENT_TYPE, "application/octet-stream")
                .header(header::CONTENT_LENGTH, &format!("{}", content_length));
        }
        let request = request.body(body).unwrap();

        let request_future = self.sender.send_request(request);

        Box::pin(async move {
            let response = request_future.await.map_err(io::Error::other)?;
            if response.status() != StatusCode::OK && response.status() != StatusCode::CREATED {
                return Err(io::Error::other(format!(
                    "Unexpected response status code: {}",
                    response.status()
                )));
            };
            let body = response.collect().await.map_err(io::Error::other)?;

            Ok(body.to_bytes().to_vec())
        })
    }

    fn drain_buffer(
        &mut self,
        buf: &mut tokio::io::ReadBuf<'_>,
        last_response: Vec<u8>,
    ) -> std::task::Poll<io::Result<()>> {
        let read_slice = match buf.remaining_mut() {
            buffer_length if buffer_length < last_response.len() => &last_response[..],
            buffer_length => {
                self.last_response = Some(last_response[buffer_length..].to_vec());
                &last_response[..buffer_length]
            }
        };
        buf.put_slice(read_slice);
        return Poll::Ready(Ok(()));
    }

    fn fill_recv_buffer(&mut self, response: Vec<u8>) {
        if !response.is_empty() {
            self.last_response.get_or_insert(vec![]).extend(response);
        }
    }
}

impl AsyncRead for ProxyConnection {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // Consume previously received bytes

        if let Some(last_response) = self.last_response.take() {
            return self.drain_buffer(buf, last_response);
        };

        // self.request.get_or_insert_with would be cool, but mutable borrows prevent that.
        let request = match self.request.as_mut() {
            None => {
                self.request = Some(self.create_request(None));
                self.request.as_mut().unwrap()
            }
            Some(request) => request,
        };

        // Empty response implies that no data was received from upstream.
        let response = ready!(request.as_mut().poll(cx))?;
        if response.is_empty() {
            return Poll::Pending;
        }

        self.drain_buffer(buf, response)
    }
}

impl AsyncWrite for ProxyConnection {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let request = match self.request.as_mut() {
            Some(request) => request,
            None => {
                ready!(self.sender.poll_ready(cx)).map_err(io::Error::other)?;
                let mut pin = self.get_mut();
                let request = pin.create_request(Some(buf));
                pin.request = Some(request);
                return Poll::Ready(Ok(buf.len()));
            }
        };

        let response = ready!(request.as_mut().poll(cx))?;
        self.fill_recv_buffer(response);

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
