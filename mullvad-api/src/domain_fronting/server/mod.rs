use std::{io, net::SocketAddr, sync::Arc, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc, oneshot},
    time::{Timeout, sleep},
};
use uuid::Uuid;

const TIMEOUT: Duration = Duration::from_secs(30);

struct Sessions {
    sessions: papaya::HashMap<Uuid, mpsc::Sender<SessionCommand>>,
    configuration: Configuration,
}

pub struct Configuration {
    pub upstream_address: SocketAddr,
}

impl Sessions {
    pub async fn handle_request(self: Arc<Self>, label: Uuid, data: Option<Vec<u8>>) {
        let map = self.sessions.pin();
        if let Some(session) = map.get(label) {
            let (cmd, rx) = SessionCommand::new(data);
            session.send()
        } else {
        }
    }

    fn remove_session(&self, session: &Uuid) {
        let _ = self.sessions.pin().remove(session);
    }
}

struct Session {
    connection: TcpStream,
    upstream_rx_bytes: Option<Vec<u8>>,
    cmd_rx: mpsc::Receiver<SessionCommand>,
    session_id: Uuid,
    sessions: Arc<Sessions>,
}

impl Session {
    pub async fn connect(
        addr: SocketAddr,
        session_id: Uuid,
        sessions: Arc<Sessions>,
    ) -> io::Result<(Self, mpsc::Sender<SessionCommand>)> {
        let connection = TcpStream::connect(addr).await?;
        let (tx, cmd_rx) = mpsc::channel(1);

        Ok((
            Self {
                connection,
                session_id,
                cmd_rx,
                upstream_rx_bytes: None,
                sessions,
            },
            tx,
        ))
    }

    pub async fn run(&mut self) {
        let Self {
            connection,
            upstream_rx_bytes,
            cmd_rx,
            sessions,
            session_id,
        } = self;
        let mut deadline = sleep(TIMEOUT);
        let mut read_buffer = vec![0u8; 8192];
        loop {
            tokio::select! {
                maybe_cmd = cmd_rx.recv() => {
                    let Some(mut cmd) = maybe_cmd else {
                        return;
                    };
                    if let Some(tx_bytes) = cmd.take_payload() {
                        if let Err(err) =  connection.write_all(&tx_bytes).await {
                            log::error!("Failed to send data to upstream: {err}");
                        }
                    }
                    cmd.respond_with(upstream_rx_bytes.take());
                },

                read_result = connection.read(&mut read_buffer) => {
                    match read_result {
                        Ok(bytes_received) => {
                            upstream_rx_bytes
                                .get_or_insert(vec![])
                                .extend(&read_buffer[..bytes_received]);
                        },
                        Err(err) => {
                            log::error!("Failed to receive data from upstream: {err}");
                            return;
                        },
                    }


                },

                _ = deadline => {


                }
            }
            deadline = sleep(TIMEOUT);
        }

        // *sessions.pin().remove()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.sessions.remove_session(&self.session_id);
    }
}

struct SessionCommand {
    tx_payload: Option<Vec<u8>>,
    return_tx: oneshot::Sender<Option<Vec<u8>>>,
}

impl SessionCommand {
    fn new(tx_payload: Option<Vec<u8>>) -> (Self, oneshot::Receiver<Option<Vec<u8>>>) {
        let (return_tx, rx) = oneshot::channel();
        (
            Self {
                tx_payload,
                return_tx,
            },
            rx,
        )
    }
    fn take_payload(&mut self) -> Option<Vec<u8>> {
        self.tx_payload.take()
    }

    fn respond_with(mut self, received_bytes: Option<Vec<u8>>) {
        let _ = self.return_tx.send(received_bytes);
    }
}
