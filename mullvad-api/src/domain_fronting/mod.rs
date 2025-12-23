//! A module for a POC of domain fronting. See IOS-1316.
//! This only compiles with the `domain-fronting` feature flag for the time being.

use std::{
    io::{self, BufRead, Error, Read},
    net::SocketAddr,
    pin::{Pin, pin},
    sync::Arc,
    task::{Poll, Waker, ready},
};

use bytes::{Buf, BufMut, BytesMut, buf::Reader};
use futures::channel::mpsc::SendError;
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
    bytes_received: usize,
    reader: Reader<BytesMut>,
    send_future: Option<Pin<Box<dyn Future<Output = Result<(), ()>> + Send>>>,
    ongoing_request: bool,
    request_tx: mpsc::Sender<Bytes>,
    response_rx: mpsc::Receiver<Bytes>,
    // call waker whenever the send_future resolves.
    read_waker: Option<Waker>,
    // call waker whenever the send_future resolves.
    write_waker: Option<Waker>,
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
            bytes_received: 0,
            reader: BytesMut::new().reader(),
            ongoing_request: false,
            request_tx,
            response_rx,
            send_future: None,
            read_waker: None,
            write_waker: None,
        })
    }

    fn update_write_waker(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) {
        let waker = cx.waker();
        let stored_waker = self.write_waker.get_or_insert_with(|| waker.clone());
        stored_waker.clone_from(waker);
    }

    fn update_read_waker(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) {
        let waker = cx.waker();
        let stored_waker = self.read_waker.get_or_insert_with(|| waker.clone());
        stored_waker.clone_from(waker);
    }
    fn resolve_write_waker(mut self: Pin<&mut Self>) {
        if let Some(waker) = self.write_waker.take() {
            waker.wake();
        }
    }
    fn resolve_read_waker(mut self: Pin<&mut Self>) {
        if let Some(waker) = self.read_waker.take() {
            waker.wake();
        }
    }

    fn fill_recv_buffer(mut self: Pin<&mut Self>, response: Bytes) {
        log::debug!("Received {} bytes", response.len());
        self.reader.get_mut().extend(response);
    }

    fn recv_buffer_empty(self: Pin<&Self>) -> bool {
        self.reader.get_ref().remaining() == 0
    }

    fn create_send_future(
        request_tx: mpsc::Sender<Bytes>,
        payload: Bytes,
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send>> {
        let send_future = async move {
            let result = request_tx.send(payload).await.map_err(|_| ());
            result
        };
        Box::pin(send_future)
    }

    // fn poll_send_future(self: Pin<&mut Self>) -> Poll<io::Result<
}

impl AsyncRead for ProxyConnection {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        log::trace!("call to poll_read");
        self.as_mut().update_read_waker(cx);
        match self.as_mut().response_rx.poll_recv(cx) {
            // indicate that the reader is shut down by reading 0 bytes.
            Poll::Ready(None) => {
                if self.as_ref().recv_buffer_empty() {
                    self.as_mut().resolve_write_waker();
                    self.as_mut().resolve_read_waker();
                    return Poll::Ready(Ok(()));
                }
            }
            Poll::Ready(Some(response)) => {
                self.as_mut().fill_recv_buffer(response);
            }
            Poll::Pending => (),
        };

        let buffer_empty = self.as_ref().recv_buffer_empty();
        if !buffer_empty {
            log::debug!("attempting to read");
            match self.reader.read(buf.initialize_unfilled()) {
                Ok(0) => (),
                Ok(n) => {
                    buf.advance(n);
                    self.bytes_received += n;
                    log::debug!("Received in total {} bytes", self.bytes_received);
                    return Poll::Ready(Ok(()));
                }
                Err(err) => {
                    return Poll::Ready(Err(err));
                }
            };
        }

        let request_tx = self.request_tx.clone();
        let send_future = self
            .send_future
            .get_or_insert_with(|| Self::create_send_future(request_tx, Bytes::new()));

        match pin!(send_future).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(_)) => {
                self.as_mut().resolve_write_waker();
                self.as_mut().resolve_read_waker();
                self.send_future = None;
                Poll::Pending
            }
            Poll::Ready(Err(_)) => {
                self.as_mut().resolve_write_waker();
                Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Actor shut down",
                )))
            }
        }
    }
}

impl AsyncWrite for ProxyConnection {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        log::debug!("call to poll_write");
        self.as_mut().update_write_waker(cx);
        if self.send_future.is_none() {
            let request_tx = self.request_tx.clone();
            let payload = Bytes::copy_from_slice(buf);
            self.send_future = Some(Self::create_send_future(request_tx, payload));
            self.as_mut().resolve_read_waker();
            self.as_mut().resolve_write_waker();
            return Poll::Ready(Ok(buf.len()));
        }

        if let Some(future) = &mut self.send_future {
            match ready!(pin!(future).poll(cx)) {
                Ok(_) => {
                    self.as_mut().resolve_write_waker();
                    self.as_mut().resolve_read_waker();
                    self.send_future = None;
                    return Poll::Pending;
                }
                Err(_) => {
                    self.send_future = None;
                    self.as_mut().resolve_read_waker();
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        "Actor shut down",
                    )));
                }
            }
        };

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
        log::debug!("Starting proxy actor with session {}", self.session_id);
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
            let payload = body.to_bytes();
            if payload.len() != 0 {
                if self.response_tx.send(payload).await.is_err() {
                    log::trace!("Response receiver down, shutting down actor");
                    return;
                }
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
