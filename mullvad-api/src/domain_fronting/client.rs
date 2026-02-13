//! Domain fronting client implementation.

use std::{
    future::Future,
    io::{self, Read},
    net::SocketAddr,
    pin::{Pin, pin},
    sync::Arc,
    task::{Poll, Waker, ready},
};

use bytes::{Buf, BytesMut, buf::Reader};
use http::{header, status::StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, client::conn::http1::SendRequest};
use hyper_util::rt::TokioIo;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    sync::mpsc,
    task::JoinHandle,
};
use tokio_rustls::rustls::{self};
use uuid::Uuid;
use webpki_roots::TLS_SERVER_ROOTS;

use crate::tls_stream::TlsStream;

use super::{DomainFronting, Error};

/// Configuration for connecting to a domain fronting proxy.
///
/// Contains the resolved address and domain fronting configuration.
/// Created from [`DomainFronting::proxy_config()`].
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug, Clone)]
pub struct ProxyConfig {
    /// The resolved socket address of the CDN.
    pub addr: SocketAddr,
    /// Internal domain fronting configuration
    domain_fronting: DomainFronting,
}

impl ProxyConfig {
    /// Create a new ProxyConfig with the given address and domain fronting configuration.
    pub fn new(addr: SocketAddr, domain_fronting: DomainFronting) -> Self {
        Self {
            addr,
            domain_fronting,
        }
    }

    /// Connect to the proxy using a TCP connection with TLS.
    pub async fn connect(&self) -> Result<ProxyConnection, Error> {
        let connection = TcpStream::connect(self.addr)
            .await
            .map_err(Error::Connection)?;
        self.connect_with_socket(connection).await
    }

    /// Connect using an existing TCP socket with TLS applied.
    pub async fn connect_with_socket(
        &self,
        tcp_stream: TcpStream,
    ) -> Result<ProxyConnection, Error> {
        self.connect_with_stream(tcp_stream, true).await
    }

    /// Connect with any stream, optionally applying TLS.
    ///
    /// This allows using arbitrary transports like in-memory streams for testing.
    /// If `apply_tls` is true, wraps the stream in TLS before establishing the HTTP connection.
    pub async fn connect_with_stream<S>(
        &self,
        stream: S,
        apply_tls: bool,
    ) -> Result<ProxyConnection, Error>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        if apply_tls {
            let config = Arc::new(
                rustls::ClientConfig::builder()
                    .with_root_certificates(read_cert_store())
                    .with_no_client_auth(),
            );
            let tls = TlsStream::connect_https_with_client_config(
                stream,
                self.domain_fronting.front(),
                config,
            )
            .await
            .map_err(Error::Tls)?;
            ProxyConnection::from_stream(
                tls,
                self.domain_fronting.proxy_host().to_string(),
                self.domain_fronting.session_header_key().to_string(),
            )
            .await
        } else {
            ProxyConnection::from_stream(
                stream,
                self.domain_fronting.proxy_host().to_string(),
                self.domain_fronting.session_header_key().to_string(),
            )
            .await
        }
    }
}

type RequestFuture = Pin<Box<dyn Future<Output = Result<(), ()>> + Send>>;

pub struct ProxyConnection {
    bytes_received: usize,
    reader: Reader<BytesMut>,
    send_future: Option<RequestFuture>,
    request_tx: mpsc::Sender<Bytes>,
    response_rx: mpsc::Receiver<Bytes>,
    // call waker whenever the send_future resolves.
    read_waker: Option<Waker>,
    // call waker whenever the send_future resolves.
    write_waker: Option<Waker>,
    // Keeping the connection task
    connection_task: JoinHandle<()>,
}

impl ProxyConnection {
    /// Create a proxy connection from any AsyncRead + AsyncWrite stream.
    ///
    /// This performs the HTTP handshake over the provided stream without applying TLS.
    /// Use `ProxyConfig::connect_with_stream` if you need TLS support.
    pub async fn from_stream<S>(
        stream: S,
        proxy_host: String,
        session_header_key: String,
    ) -> Result<Self, Error>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let io = TokioIo::new(stream);
        let (sender, conn) = hyper::client::conn::http1::handshake(io).await?;
        let connection_task = tokio::spawn(async move {
            if let Err(err) = conn.await {
                log::error!("Domain fronting connection failed: {:?}", err);
            }
        });
        Self::initialize(sender, proxy_host, session_header_key, connection_task).await
    }

    async fn initialize(
        mut sender: SendRequest<Full<Bytes>>,
        proxy_host: String,
        session_header_key: String,
        connection_task: JoinHandle<()>,
    ) -> Result<Self, Error> {
        sender.ready().await?;
        let (response_tx, response_rx) = mpsc::channel(1);
        let (request_tx, request_rx) = mpsc::channel(1);
        let actor = ProxyActor::new(
            sender,
            proxy_host,
            session_header_key,
            request_rx,
            response_tx,
        );
        tokio::spawn(actor.run());

        Ok(Self {
            bytes_received: 0,
            reader: BytesMut::new().reader(),
            request_tx,
            response_rx,
            send_future: None,
            read_waker: None,
            write_waker: None,
            connection_task,
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
        self.reader.get_mut().extend(response);
    }

    fn recv_buffer_empty(self: Pin<&Self>) -> bool {
        self.reader.get_ref().remaining() == 0
    }

    fn create_send_future(
        request_tx: mpsc::Sender<Bytes>,
        payload: Bytes,
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send>> {
        let send_future = async move { request_tx.send(payload).await.map_err(|_| ()) };
        Box::pin(send_future)
    }
}

impl AsyncRead for ProxyConnection {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
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
            match self.reader.read(buf.initialize_unfilled()) {
                Ok(0) => (),
                Ok(n) => {
                    buf.advance(n);
                    self.bytes_received += n;
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

        match ready!(pin!(send_future).poll(cx)) {
            Ok(_) => {
                self.as_mut().resolve_write_waker();
                self.as_mut().resolve_read_waker();
                self.send_future = None;
                Poll::Pending
            }
            Err(_) => {
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
        self.as_mut().update_write_waker(cx);

        // If there's a pending send, wait for it to complete first
        if let Some(future) = &mut self.send_future {
            match ready!(pin!(future).poll(cx)) {
                Ok(_) => {
                    self.send_future = None;
                    // Fall through to accept new data
                }
                Err(_) => {
                    self.send_future = None;
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        "Actor shut down",
                    )));
                }
            }
        }

        // Accept the write by creating a new send future
        let request_tx = self.request_tx.clone();
        let payload = Bytes::copy_from_slice(buf);
        self.send_future = Some(Self::create_send_future(request_tx, payload));
        self.as_mut().resolve_read_waker();
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl Drop for ProxyConnection {
    fn drop(&mut self) {
        // Technically the conneciton task will be shut down once the last instance of the
        // associated `SendRequest` is destoryed, but this behavior is not documented anywhere, as
        // such, let's abort the task ourselves anyway.
        self.connection_task.abort();
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
    session_header_key: String,
    proxy_host: String,
    request_rx: mpsc::Receiver<Bytes>,
    response_tx: mpsc::Sender<Bytes>,
}

impl ProxyActor {
    fn new(
        sender: SendRequest<Full<Bytes>>,
        proxy_host: String,
        session_header_key: String,
        request_rx: mpsc::Receiver<Bytes>,
        response_tx: mpsc::Sender<Bytes>,
    ) -> Self {
        Self {
            sender,
            session_id: Uuid::new_v4(),
            session_header_key,
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
                    log::debug!("Failed to read whole body of response: {err}");
                    return;
                }
            };
            let payload = body.to_bytes();
            if !payload.is_empty() && self.response_tx.send(payload).await.is_err() {
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
            .header(&self.session_header_key, &format!("{}", self.session_id))
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::CONTENT_LENGTH, &format!("{}", content_length))
            .body(body)
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_fronting::server;
    use hyper_util::rt::TokioIo;
    use std::convert::Infallible;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt, duplex},
        net::TcpListener,
    };

    /// Spawn an echo TCP server for testing. Returns the address it's listening on.
    async fn spawn_echo_server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind echo server");
        let addr = listener.local_addr().expect("Failed to get local addr");

        tokio::spawn(async move {
            loop {
                let (mut socket, _) = match listener.accept().await {
                    Ok(conn) => conn,
                    Err(_) => break,
                };

                tokio::spawn(async move {
                    let mut buf = vec![0u8; 4096];
                    loop {
                        match socket.read(&mut buf).await {
                            Ok(0) => break, // EOF
                            Ok(n) => {
                                if socket.write_all(&buf[..n]).await.is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                });
            }
        });

        addr
    }

    const TEST_SESSION_HEADER: &str = "X-Test-Session";

    #[tokio::test]
    async fn test_client_server_bidirectional() {
        // Spawn echo server that will be the upstream target
        let echo_addr = spawn_echo_server().await;

        // Create in-memory transport between client and proxy server HTTP layers
        let (client_stream, server_stream) = duplex(8192);

        // Start proxy server with default TCP connector pointing to echo server
        let sessions = server::Sessions::new(echo_addr, TEST_SESSION_HEADER.to_string());
        let sessions_clone = sessions.clone();

        // Spawn HTTP server on server_stream
        tokio::spawn(async move {
            let io = TokioIo::new(server_stream);
            let service = hyper::service::service_fn(move |req| {
                let sessions = sessions_clone.clone();
                async move { Ok::<_, Infallible>(sessions.handle_request(req).await) }
            });

            let _ = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .await;
        });

        // Create client connection using the in-memory stream (no TLS)
        let proxy_config = ProxyConfig::new(
            echo_addr,
            DomainFronting::new(
                "example.com".to_string(),
                "api.example.com".to_string(),
                TEST_SESSION_HEADER.to_string(),
            ),
        );

        let mut client = proxy_config
            .connect_with_stream(client_stream, false)
            .await
            .expect("Failed to create client connection");

        // Test: write to client, should echo back
        let test_data = b"Hello from client";
        client
            .write_all(test_data)
            .await
            .expect("Failed to write to client");

        // Read the echo response
        let mut buffer = vec![0u8; 1024];
        let n = client
            .read(&mut buffer)
            .await
            .expect("Failed to read from client");

        assert_eq!(
            &buffer[..n],
            test_data,
            "Echo server should return the same data"
        );

        // Test multiple round trips
        let test_data2 = b"Second message";
        client
            .write_all(test_data2)
            .await
            .expect("Failed to write second message");

        let n = client
            .read(&mut buffer)
            .await
            .expect("Failed to read second response");

        assert_eq!(&buffer[..n], test_data2, "Second echo failed");
    }

    #[tokio::test]
    async fn test_multiple_sessions() {
        // Spawn echo server
        let echo_addr = spawn_echo_server().await;

        // Create two separate client-server pairs
        let (client_stream1, server_stream1) = duplex(8192);
        let (client_stream2, server_stream2) = duplex(8192);

        let sessions = server::Sessions::new(echo_addr, TEST_SESSION_HEADER.to_string());

        // Spawn server for first connection
        let sessions_clone1 = sessions.clone();
        tokio::spawn(async move {
            let io = TokioIo::new(server_stream1);
            let service = hyper::service::service_fn(move |req| {
                let sessions = sessions_clone1.clone();
                async move { Ok::<_, Infallible>(sessions.handle_request(req).await) }
            });
            let _ = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .await;
        });

        // Spawn server for second connection
        let sessions_clone2 = sessions.clone();
        tokio::spawn(async move {
            let io = TokioIo::new(server_stream2);
            let service = hyper::service::service_fn(move |req| {
                let sessions = sessions_clone2.clone();
                async move { Ok::<_, Infallible>(sessions.handle_request(req).await) }
            });
            let _ = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .await;
        });

        // Create two client connections
        let proxy_config = ProxyConfig::new(
            echo_addr,
            DomainFronting::new(
                "example.com".to_string(),
                "api.example.com".to_string(),
                TEST_SESSION_HEADER.to_string(),
            ),
        );

        let mut client1 = proxy_config
            .connect_with_stream(client_stream1, false)
            .await
            .expect("Failed to create client1");

        let mut client2 = proxy_config
            .connect_with_stream(client_stream2, false)
            .await
            .expect("Failed to create client2");

        // Write to both clients and verify they get independent echoes
        client1
            .write_all(b"from_client1")
            .await
            .expect("Client 1 write failed");
        client2
            .write_all(b"from_client2")
            .await
            .expect("Client 2 write failed");

        // Read responses
        let mut buf1 = vec![0u8; 1024];
        let mut buf2 = vec![0u8; 1024];

        let n1 = client1.read(&mut buf1).await.expect("Client 1 read failed");
        let n2 = client2.read(&mut buf2).await.expect("Client 2 read failed");

        assert_eq!(&buf1[..n1], b"from_client1", "Client 1 got wrong echo");
        assert_eq!(&buf2[..n2], b"from_client2", "Client 2 got wrong echo");
    }

    #[tokio::test]
    async fn test_connection_task_stopped_on_drop() {
        // Spawn echo server
        let echo_addr = spawn_echo_server().await;

        let (client_stream, server_stream) = duplex(8192);
        let sessions = server::Sessions::new(echo_addr, TEST_SESSION_HEADER.to_string());
        let sessions_clone = sessions.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(server_stream);
            let service = hyper::service::service_fn(move |req| {
                let sessions = sessions_clone.clone();
                async move { Ok::<_, Infallible>(sessions.handle_request(req).await) }
            });
            let _ = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .await;
        });

        let proxy_config = ProxyConfig::new(
            echo_addr,
            DomainFronting::new(
                "example.com".to_string(),
                "api.example.com".to_string(),
                TEST_SESSION_HEADER.to_string(),
            ),
        );

        let client = proxy_config
            .connect_with_stream(client_stream, false)
            .await
            .expect("Failed to create client connection");

        // Grab a handle to the connection task before dropping
        let connection_task = &client.connection_task;
        // The task should still be running
        assert!(
            !connection_task.is_finished(),
            "Connection task should be running before drop"
        );

        // Clone the abort handle so we can check task status after drop
        let task_handle = client.connection_task.abort_handle();

        // Drop the proxy connection
        drop(client);

        // Give the runtime a moment to process the abort
        tokio::task::yield_now().await;

        // The connection task should now be finished (aborted)
        assert!(
            task_handle.is_finished(),
            "Connection task should be stopped after ProxyConnection is dropped"
        );
    }

    #[tokio::test]
    async fn test_large_data_transfer() {
        // Spawn echo server
        let echo_addr = spawn_echo_server().await;

        let (client_stream, server_stream) = duplex(65536);
        let sessions = server::Sessions::new(echo_addr, TEST_SESSION_HEADER.to_string());
        let sessions_clone = sessions.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(server_stream);
            let service = hyper::service::service_fn(move |req| {
                let sessions = sessions_clone.clone();
                async move { Ok::<_, Infallible>(sessions.handle_request(req).await) }
            });
            let _ = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .await;
        });

        let proxy_config = ProxyConfig::new(
            echo_addr,
            DomainFronting::new(
                "example.com".to_string(),
                "api.example.com".to_string(),
                TEST_SESSION_HEADER.to_string(),
            ),
        );

        let mut client = proxy_config
            .connect_with_stream(client_stream, false)
            .await
            .expect("Failed to create client");

        // Send 100KB of data
        let large_data = vec![0x42u8; 100_000];
        client
            .write_all(&large_data)
            .await
            .expect("Failed to write large data");

        // Read the echo response
        let mut received = Vec::new();
        let mut buffer = vec![0u8; 4096];

        while received.len() < large_data.len() {
            match client.read(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(n) => received.extend_from_slice(&buffer[..n]),
                Err(e) => panic!("Read error: {}", e),
            }
        }

        assert_eq!(received.len(), large_data.len(), "Did not receive all data");
        assert_eq!(received, large_data, "Data corruption detected");
    }
}
