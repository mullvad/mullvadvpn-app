use std::{io, net::SocketAddr, sync::Arc, time::Duration};

use bytes::BytesMut;
use http::{Request, Response, StatusCode, header};
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
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
        println!("NUMBER OF ENTRIES IN MAP: {}", self.sessions.pin().len());
        let session_id = request
            .headers()
            .get(SESSION_HEADER_KEY)
            .and_then(|value| Uuid::try_parse_ascii(value.as_ref()).ok());
        println!("Handling the request");
        let Some(conent_length_header) = request.headers().get(header::CONTENT_LENGTH) else {
            println!("Handling request with ID and no data");
            return self.handle_request_inner(session_id, None).await;
        };

        let Ok(content_length) = conent_length_header.to_str() else {
            println!("No content length");
            return Self::handle_session_error();
        };

        let Ok(content_length) = content_length.parse::<u64>() else {
            println!("Invalid content length: {content_length}");
            return Self::handle_session_error();
        };

        let body = match content_length {
            0 => None,
            _any_other_value => {
                let Ok(body) = request.collect().await.map(|b| b.to_bytes()) else {
                    println!("failed to read body");
                    return Self::handle_session_error();
                };
                Some(body)
            }
        };

        println!("Handling request with some body");
        return self.handle_request_inner(session_id, body).await;
    }

    async fn handle_request_inner(
        self: Arc<Self>,
        maybe_label: Option<Uuid>,
        data: Option<Bytes>,
    ) -> Response<Full<Bytes>> {
        println!("get here");
        let Some(label) = maybe_label else {
            println!("Creating a new session");
            return self.handle_new_session(data).await;
        };

        let cmd_tx = {
            let map = self.sessions.pin();
            let Some(cmd_tx) = map.get(&label) else {
                println!("no session? Failed to find session {label}");
                return Self::handle_session_error();
            };
            cmd_tx.clone()
        };

        return self
            .clone()
            .handle_existing_session_request(&cmd_tx, data)
            .await;
    }

    async fn handle_existing_session_request(
        self: Arc<Self>,
        cmd_tx: &mpsc::Sender<SessionCommand>,
        data: Option<Bytes>,
    ) -> Response<Full<Bytes>> {
        println!("Handling existing session");
        let Ok(return_payload) = SessionCommand::send(data, &cmd_tx).await else {
            println!("Failed send command");
            return Self::handle_session_error();
        };
        let body = return_payload
            .map(Bytes::from_owner)
            .unwrap_or(Bytes::new());

        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(Full::new(body))
            .unwrap();
    }

    async fn handle_new_session(self: Arc<Self>, data: Option<Bytes>) -> Response<Full<Bytes>> {
        let new_label = Uuid::new_v4();

        let sessions = self.clone();
        let session_id = new_label.clone();
        let (cmd_tx, cmd_rx) = mpsc::channel(1);
        self.sessions.pin().insert(new_label, cmd_tx.clone());
        println!("NUMBER OF ENTRIES IN MAP after insert: {}", self.sessions.pin().len());
        println!("Aded session {:?}", new_label);
        dbg!(&self);
        tokio::spawn(async move {
            let Ok(mut session) = Session::connect(cmd_rx, session_id, sessions).await else {
                return;
            };
            session.run().await;
        });

        println!("got here");
        let Ok(return_payload) = SessionCommand::send(data, &cmd_tx).await else {
            return Self::handle_session_error();
        };
        let body = return_payload
            .map(Bytes::from_owner)
            .unwrap_or(Bytes::new());

        return Response::builder()
            .status(StatusCode::CREATED)
            .header(SESSION_HEADER_KEY, new_label.hyphenated().to_string())
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(Full::new(body))
            .unwrap();
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
    upstream_rx_bytes: Option<BytesMut>,
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
            upstream_rx_bytes: None,
            sessions,
        })
    }

    pub async fn run(&mut self) {
        let Self {
            connection,
            upstream_rx_bytes,
            cmd_rx,
            sessions: _,
            session_id: _,
        } = self;
        let mut deadline = sleep(CONNECTION_TIMEOUT);
        let mut read_buffer = vec![0u8; 8192];
        println!("Starting session loop");

        loop {
            tokio::select! {
                maybe_cmd = cmd_rx.recv() => {
                    let Some(mut cmd) = maybe_cmd else {
                        return;
                    };

                    println!("Received msg");
                    if let Some(tx_bytes) = cmd.take_payload() {
                        if let Err(err) =  connection.write_all(&tx_bytes).await {
                            log::error!("Failed to send data to upstream: {err}");
                        }
                    }
                    // drop everything on read error
                    let response_bytes = match timeout(READ_TIMEOUT, connection.read(&mut read_buffer)).await {
                        Ok(Ok(bytes_read)) => {
                            Some(Bytes::copy_from_slice(&read_buffer[..bytes_read]))
                        },
                        Ok(Err(connection_error)) => {
                            log::error!("Failed to receive data from upstream {connection_error}");
                            return;
                        },
                        Err(timeout) => None,
                    };

                    cmd.respond_with(response_bytes);
                },

                _ = deadline => {
                    return;
                }
            }
            deadline = sleep(CONNECTION_TIMEOUT);
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
    return_tx: oneshot::Sender<Option<Bytes>>,
}

impl SessionCommand {
    async fn send(
        payload: Option<Bytes>,
        cmd_tx: &mpsc::Sender<SessionCommand>,
    ) -> anyhow::Result<Option<Bytes>> {
        let (cmd, rx) = Self::new(payload);
        cmd_tx.send(cmd).await?;
        let payload = rx.await?;
        Ok(payload)
    }

    fn new(tx_payload: Option<Bytes>) -> (Self, oneshot::Receiver<Option<Bytes>>) {
        let (return_tx, rx) = oneshot::channel();
        (
            Self {
                tx_payload,
                return_tx,
            },
            rx,
        )
    }
    fn take_payload(&mut self) -> Option<Bytes> {
        self.tx_payload.take()
    }

    fn respond_with(mut self, received_bytes: Option<Bytes>) {
        let _ = self.return_tx.send(received_bytes);
    }
}
