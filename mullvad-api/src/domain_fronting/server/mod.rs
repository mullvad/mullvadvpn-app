use std::{hash::RandomState, io, net::SocketAddr, pin::pin, sync::Arc, time::Duration};

use bytes::BytesMut;
use http::{Request, Response, StatusCode, header};
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use papaya::{HashMapRef, LocalGuard};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc, oneshot},
    time::{Timeout, sleep, timeout},
};
use uuid::Uuid;

use crate::domain_fronting::SESSION_HEADER_KEY;

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);
const READ_TIMEOUT: Duration = Duration::from_millis(50);

#[derive(Debug)]
pub struct Sessions {
    sessions: papaya::HashMap<Uuid, mpsc::Sender<SessionCommand>>,
    configuration: Configuration,
}

#[derive(Debug)]
pub struct Configuration {
    pub upstream: SocketAddr,
}

impl Sessions {
    pub fn new(upstream: SocketAddr) -> Arc<Self> {
        let sessions = Sessions {
            configuration: Configuration { upstream },
            sessions: Default::default(),
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
            log::trace!("Failed to read session header");
            return Self::handle_session_error();
        };

        let Ok(body) = request.collect().await.map(|b| b.to_bytes()) else {
            log::trace!("failed to read body");
            return Self::handle_session_error();
        };

        return self.handle_request_inner(session_id, body).await;
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
        let Ok(body) = SessionCommand::send(data, &cmd_tx).await else {
            println!("Failed send command");
            return Self::handle_session_error();
        };

        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(Full::new(body))
            .unwrap();
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
        let session_id = new_session.clone();
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
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Full::new(Bytes::new()))
            .unwrap();
    }

    pub fn remove_session(self: Arc<Self>, session: &Uuid) {
        println!("Removing session {session}");
        let _ = self.sessions.pin().remove(session);
    }
}

struct Session {
    connection: TcpStream,
    cmd_rx: mpsc::Receiver<SessionCommand>,
    session_id: Uuid,
    sessions: Arc<Sessions>,
}

impl Session {
    pub async fn connect(
        cmd_rx: mpsc::Receiver<SessionCommand>,
        session_id: Uuid,
        sessions: Arc<Sessions>,
    ) -> io::Result<Self> {
        let connection = match TcpStream::connect(sessions.configuration.upstream).await {
            Ok(conn) => conn,
            Err(err) => {
                log::error!("Failed to connect to upstream server: {err}");
                println!("Failed to connect to upstream server: {err}");
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
        let mut read_buffer = vec![0u8; 8192];
        log::trace!("Starting session loop");

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

                    } else {
                        log::debug!("Received no payload for session {}", session_id);
                    }
                    // drop everything on read error
                    let response_bytes = match timeout(READ_TIMEOUT, connection.read(&mut read_buffer)).await {
                        Ok(Ok(bytes_read)) => {
                            deadline.set(sleep(CONNECTION_TIMEOUT));
                            Bytes::copy_from_slice(&read_buffer[..bytes_read])
                        },
                        Ok(Err(connection_error)) => {
                            log::error!("Failed to receive data from upstream {connection_error}");
                            return;
                        },
                        Err(timeout) => Bytes::new(),
                    };


                    log::debug!("Responding with {} bytes for session {}", response_bytes.len(), session_id);
                    cmd.respond_with(response_bytes);
                },

                _ = deadline_ref => {
                    return;
                }
            }
        }
    }
}

impl Drop for Session {
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

    fn respond_with(mut self, received_bytes: Bytes) {
        let _ = self.return_tx.send(received_bytes);
    }
}
