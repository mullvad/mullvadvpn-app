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
use http::{Request, Response, header, status::StatusCode};
use http_body_util::{BodyExt, Empty, Full};
use hyper::{
    body::{Bytes, Incoming},
    client::conn::http1::SendRequest,
};
use hyper_util::rt::TokioIo;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    sync::mpsc,
};
use tokio_rustls::rustls::{self};
use uuid::Uuid;
use webpki_roots::TLS_SERVER_ROOTS;

use crate::{DefaultDnsResolver, DnsResolver, tls_stream::TlsStream};

pub mod server;

const SESSION_HEADER_KEY_CLIENT: &str = "X-Mullvad-Session";
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
    recv_buffer: Option<Bytes>,
    ongoing_request: bool,
    request_tx: mpsc::Sender<Bytes>,
    response_tx: mpsc::Receiver<Bytes>,
}

impl ProxyConnection {
    async fn initialize(
        mut sender: SendRequest<Full<Bytes>>,
        proxy_host: String,
    ) -> anyhow::Result<Self> {
        sender.ready().await?;
        let (response_tx, response_rx) = mpsc::channel(1);
        let (request_tx, request_rx) = mpsc::channel(1);
        let actor = ProxyActor::new(sender, proxy_host, request_rx, response_tx);
        tokio::spawn(actor.run());

        Ok(Self {
            sender,
            proxy_host,
            recv_buffer: None,
            ongoing_request: false,
            request_tx,
            response_rx,
        })
    }

    fn drain_buffer(
        &mut self,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        let Some(received_bytes) = self.recv_buffer.take() else {
            return Poll::Pending;
        };
        received_bytes.copy_to_slice(buf);
        let read_slice = match buf.remaining_mut() {
            buffer_length if buffer_length > received_bytes.len() => &received_bytes[..],
            buffer_length => {
                self.recv_buffer = Some(received_bytes[buffer_length..].to_vec());
                &received_bytes[..buffer_length]
            }
        };
        buf.put_slice(read_slice);
        return Poll::Ready(Ok(()));
    }

    fn fill_recv_buffer(&mut self, response: Vec<u8>) {
        if !response.is_empty() {
            self.recv_buffer.get_or_insert(vec![]).extend(response);
        }
    }
}

impl AsyncRead for ProxyConnection {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // self.request.get_or_insert_with would be cool, but mutable borrows prevent that.
        let request = match self.request.as_mut() {
            None => self.create_request(None),
            Some(request) => request,
        };

        let response = ready!(request.as_mut().poll(cx))?;
        self.request = None;
        self.fill_recv_buffer(response);

        self.drain_buffer(buf)
    }
}

impl AsyncWrite for ProxyConnection {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let (request, sent_request) = match self.request.as_mut() {
            Some(request) => (request, false),
            None => {
                self.create_request(Some(buf));
                cx.waker().wake_by_ref();
                println!("from none ACtually sent request traffic");
                return Poll::Ready(Ok(buf.len()));
            }
        };

        let response = ready!(request.as_mut().poll(cx))?;
        self.request = None;
        self.fill_recv_buffer(response);

        self.create_request(Some(buf));
        cx.waker().wake_by_ref();
        println!("from end ACtually sent request traffic");
        return Poll::Ready(Ok(buf.len()));
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

struct Payload {
    payload: Bytes,
    // permit: OwnedPermit
}

struct ProxyActor {
    sender: SendRequest<Full<Bytes>>,
    session_id: Uuid,
    proxy_host: String,
    request_rx: mpsc::Receiver<Bytes>,
    response_tx: mpsc::Sender<Bytes>,
}

impl ProxyActor {
    fn new(
        sender: SendRequest<Full<Bytes>>,
        proxy_host: String,
        request_rx: mpsc::Receiver<Bytes>,
        response_tx: mpsc::Sender<Bytes>,
    ) -> Self {
        Self {
            sender,
            session_id: Uuid::new_v4(),
            proxy_host,
            request_rx,
            response_tx,
        }
    }
    async fn run(mut self) {
        loop {
            let Some(msg) = self.request_rx.recv().await else {
                log::trace!("Shutting down proxy - rx channel has no writers");
                return;
            };

            let request = self.create_request(msg);
            if let Err(err) = self.sender.ready().await {
                log::trace!(
                    "Dropping proxy actor due to error when waiting for connection to be ready: {err}"
                );
                return;
            };
            let response = match self.sender.send_request(request).await {
                Ok(response) => response,
                Err(err) => {
                    log::trace!(
                        "Dropping proxy actor due to error when waiting for connection to be ready: {err}"
                    );
                    return;
                }
            };

            if response.status() != StatusCode::OK {
                log::debug!("Unexpected status code from proxy: {}", response.status());
                return;
            }

            let body = match response.collect().await {
                Ok(body) => body,
                Err(err) => {
                    log::debug!("Failed to read whole body of reqsponse: {err}");
                    return;
                }
            };
            if self.response_tx.send(body.to_bytes()).await.is_err() {
                log::trace!("Response receiver down, shutting down actor");
                return;
            }
        }
    }

    fn create_request(&mut self, buffer: Bytes) -> http::Request<Full<Bytes>> {
        let content_length = buffer.len();
        let body = Full::new(buffer);

        hyper::Request::post(&format!("https://{}/", self.proxy_host))
            .header(header::HOST, self.proxy_host.clone())
            .header(header::ACCEPT, "*/*")
            .header(SESSION_HEADER_KEY, &format!("{}", self.session_id))
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::CONTENT_LENGTH, &format!("{}", content_length))
            .body(body)
            .unwrap()
    }
}
