use std::{
    future::Future, hash::RandomState, io, net::SocketAddr, pin::pin, sync::Arc, time::Duration,
};

use http::{Request, Response, StatusCode, header};
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use papaya::{HashMapRef, LocalGuard};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc, oneshot},
    time::{sleep, timeout},
};
use uuid::Uuid;

use crate::domain_fronting::SESSION_HEADER_KEY;

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);
const READ_TIMEOUT: Duration = Duration::from_millis(50);

/// Factory trait for creating upstream connections.
///
/// This trait abstracts how upstream connections are created, allowing
/// injection of test doubles or alternative transports.
pub trait UpstreamConnector: Clone + Send + Sync + 'static {
    /// The stream type produced by this connector.
    type Stream: AsyncRead + AsyncWrite + Unpin + Send + 'static;

    /// Connect to the given address.
    fn connect(&self, addr: SocketAddr) -> impl Future<Output = io::Result<Self::Stream>> + Send;
}

/// Default connector using TCP streams.
#[derive(Clone, Default)]
pub struct TcpConnector;

impl UpstreamConnector for TcpConnector {
    type Stream = TcpStream;

    async fn connect(&self, addr: SocketAddr) -> io::Result<TcpStream> {
        TcpStream::connect(addr).await
    }
}

/// Manages domain fronting sessions, routing HTTP requests to upstream connections.
pub struct Sessions<C: UpstreamConnector = TcpConnector> {
    sessions: papaya::HashMap<Uuid, mpsc::Sender<SessionCommand>>,
    configuration: Configuration,
    connector: C,
}

impl<C: UpstreamConnector> std::fmt::Debug for Sessions<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sessions")
            .field("sessions", &self.sessions)
            .field("configuration", &self.configuration)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct Configuration {
    pub upstream: SocketAddr,
}

impl Sessions<TcpConnector> {
    /// Create a new session manager with the default TCP connector.
    pub fn new(upstream: SocketAddr) -> Arc<Self> {
        Self::with_connector(upstream, TcpConnector)
    }
}

impl<C: UpstreamConnector> Sessions<C> {
    /// Create a new session manager with a custom connector.
    ///
    /// This allows injecting test doubles or alternative transports.
    pub fn with_connector(upstream: SocketAddr, connector: C) -> Arc<Self> {
        let sessions = Sessions {
            configuration: Configuration { upstream },
            sessions: Default::default(),
            connector,
        };
        Arc::new(sessions)
    }

    pub async fn handle_request(
        self: Arc<Self>,
        request: Request<Incoming>,
    ) -> Response<Full<Bytes>> {
        let Some(session_id) = request
            .headers()
            .get(SESSION_HEADER_KEY)
            .and_then(|value| Uuid::try_parse_ascii(value.as_ref()).ok())
        else {
            return Self::handle_session_error();
        };

        let Ok(body) = request.collect().await.map(|b| b.to_bytes()) else {
            return Self::handle_session_error();
        };

        self.handle_request_inner(session_id, body).await
    }

    async fn handle_request_inner(
        self: Arc<Self>,
        session: Uuid,
        data: Bytes,
    ) -> Response<Full<Bytes>> {
        let cmd_tx = {
            let map = self.sessions.pin();
            match map.get(&session) {
                Some(tx) => tx.clone(),
                None => self.clone().handle_new_session(session, map),
            }
        };

        return self
            .clone()
            .handle_existing_session_request(&cmd_tx, data)
            .await;
    }

    async fn handle_existing_session_request(
        self: Arc<Self>,
        cmd_tx: &mpsc::Sender<SessionCommand>,
        data: Bytes,
    ) -> Response<Full<Bytes>> {
        let Ok(body) = SessionCommand::send(data, cmd_tx).await else {
            log::error!("Failed to send command to session");
            return Self::handle_session_error();
        };

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(Full::new(body))
            .unwrap()
    }

    fn handle_new_session(
        self: Arc<Self>,
        new_session: Uuid,
        session_map: HashMapRef<
            '_,
            Uuid,
            mpsc::Sender<SessionCommand>,
            RandomState,
            LocalGuard<'_>,
        >,
    ) -> mpsc::Sender<SessionCommand> {
        let sessions = self.clone();
        let session_id = new_session;
        let (cmd_tx, cmd_rx) = mpsc::channel(1);
        session_map.insert(new_session, cmd_tx.clone());

        tokio::spawn(async move {
            let Ok(mut session) = Session::connect(cmd_rx, session_id, sessions).await else {
                return;
            };
            session.run().await;
        });

        cmd_tx
    }

    fn handle_session_error() -> Response<Full<Bytes>> {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Full::new(Bytes::new()))
            .unwrap()
    }

    pub fn remove_session(self: Arc<Self>, session: &Uuid) {
        log::debug!("Removing session {}", session);
        let _ = self.sessions.pin().remove(session);
    }
}

struct Session<C: UpstreamConnector> {
    connection: C::Stream,
    cmd_rx: mpsc::Receiver<SessionCommand>,
    session_id: Uuid,
    sessions: Arc<Sessions<C>>,
}

impl<C: UpstreamConnector> Session<C> {
    pub async fn connect(
        cmd_rx: mpsc::Receiver<SessionCommand>,
        session_id: Uuid,
        sessions: Arc<Sessions<C>>,
    ) -> io::Result<Self> {
        let connection = match sessions
            .connector
            .connect(sessions.configuration.upstream)
            .await
        {
            Ok(conn) => conn,
            Err(err) => {
                log::error!("Failed to connect to upstream server: {}", err);
                sessions.remove_session(&session_id);
                return Err(err);
            }
        };

        Ok(Self {
            connection,
            session_id,
            cmd_rx,
            sessions,
        })
    }

    pub async fn run(&mut self) {
        let Self {
            connection,
            cmd_rx,
            sessions: _,
            session_id,
        } = self;
        let mut deadline = pin!(sleep(CONNECTION_TIMEOUT));
        let mut read_buffer = vec![0u8; 1024 * 64];

        loop {
            let deadline_ref = deadline.as_mut();
            tokio::select! {
                maybe_cmd = cmd_rx.recv() => {
                    let Some(mut cmd) = maybe_cmd else {
                        return;
                    };

                    if let Some(tx_bytes) = cmd.take_payload() {
                        log::debug!("Received {} bytes for session {}", tx_bytes.len(), session_id);
                        if let Err(err) =  connection.write_all(&tx_bytes).await {
                            log::error!("Failed to send data to upstream: {err}");
                        }

                    }

                    let response_bytes = match timeout(READ_TIMEOUT, connection.read(&mut read_buffer)).await {
                        Ok(Ok(bytes_read)) => {
                            deadline.set(sleep(CONNECTION_TIMEOUT));
                            Bytes::copy_from_slice(&read_buffer[..bytes_read])
                        },
                        // drop everything on read error
                        Ok(Err(connection_error)) => {
                            log::error!("Failed to receive data from upstream {connection_error}");
                            return;
                        },
                        Err(_timeout) => Bytes::new(),
                    };
                    cmd.respond_with(response_bytes);
                },

                _ = deadline_ref => {
                    return;
                }
            }
        }
    }
}

impl<C: UpstreamConnector> Drop for Session<C> {
    fn drop(&mut self) {
        self.sessions.clone().remove_session(&self.session_id);
    }
}

#[derive(Debug)]
struct SessionCommand {
    tx_payload: Option<Bytes>,
    return_tx: oneshot::Sender<Bytes>,
}

impl SessionCommand {
    async fn send(payload: Bytes, cmd_tx: &mpsc::Sender<SessionCommand>) -> anyhow::Result<Bytes> {
        let (cmd, rx) = Self::new(payload);
        cmd_tx.send(cmd).await?;
        let payload = rx.await?;
        Ok(payload)
    }

    fn new(tx_payload: Bytes) -> (Self, oneshot::Receiver<Bytes>) {
        let (return_tx, rx) = oneshot::channel();
        (
            Self {
                tx_payload: Some(tx_payload),
                return_tx,
            },
            rx,
        )
    }
    fn take_payload(&mut self) -> Option<Bytes> {
        self.tx_payload.take()
    }

    fn respond_with(self, received_bytes: Bytes) {
        let _ = self.return_tx.send(received_bytes);
    }
}
