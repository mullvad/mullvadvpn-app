//! The real OS socket carrying a proxied connection's bytes to its original destination.

use bytes::{Bytes, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::{Notify, mpsc},
    task::JoinSet,
    time::timeout,
};
use tokio_util::task::AbortOnDropHandle;

use std::sync::Arc;

use super::{CONNECT_TIMEOUT, TCP_DATA_CHANNEL_CAPACITY, TcpConnectionId, UPSTREAM_READ_CHUNK};

/// Control-plane notifications from an upstream socket to the stack.
///
/// Payloads travel on the per-connection channels in [`UpstreamSocket`] instead, so that one
/// connection's backpressure cannot stall the stack's event handling.
pub enum UpstreamEvent {
    Connected,
    ConnectFailed,
    /// An established connection failed and cannot be used any more.
    Error,
}

pub struct UpstreamMsg {
    pub id: TcpConnectionId,
    pub event: UpstreamEvent,
}

/// The stack's handle on one upstream connection.
pub struct UpstreamSocket {
    /// Payloads to write upstream. Dropping this shuts down the upstream write half, which is how
    /// a client's FIN is forwarded.
    to_upstream_tx: Option<mpsc::Sender<Bytes>>,
    /// Payloads read from upstream. Disconnecting signals EOF.
    from_upstream_rx: mpsc::Receiver<Bytes>,
    _supervisor: AbortOnDropHandle<()>,
}

impl UpstreamSocket {
    /// Start connecting to `id.upstream`, reporting progress on `event_tx`.
    pub fn connect(
        id: TcpConnectionId,
        event_tx: mpsc::Sender<UpstreamMsg>,
        notify: Arc<Notify>,
    ) -> Self {
        let (to_upstream_tx, to_upstream_rx) = mpsc::channel::<Bytes>(TCP_DATA_CHANNEL_CAPACITY);
        let (from_upstream_tx, from_upstream_rx) = mpsc::channel::<Bytes>(TCP_DATA_CHANNEL_CAPACITY);

        let supervisor = tokio::spawn(async move {
            let stream = match timeout(CONNECT_TIMEOUT, TcpStream::connect(id.upstream)).await {
                Ok(Ok(stream)) => stream,
                Ok(Err(err)) => {
                    log::debug!("{id:?} - upstream connection failed: {err}");
                    return send_event(&event_tx, id, UpstreamEvent::ConnectFailed, &notify).await;
                }
                Err(_) => {
                    log::debug!("{id:?} - upstream connection timed out");
                    return send_event(&event_tx, id, UpstreamEvent::ConnectFailed, &notify).await;
                }
            };

            send_event(&event_tx, id, UpstreamEvent::Connected, &notify).await;

            let (reader, writer) = stream.into_split();
            let mut halves = JoinSet::new();
            halves.spawn(read_loop(
                reader,
                from_upstream_tx,
                id,
                event_tx.clone(),
                notify.clone(),
            ));
            halves.spawn(write_loop(writer, to_upstream_rx, id, event_tx, notify));

            // Dropping the set takes the other half down with the first one to finish.
            halves.join_next().await;
        });

        Self {
            to_upstream_tx: Some(to_upstream_tx),
            from_upstream_rx,
            _supervisor: AbortOnDropHandle::new(supervisor),
        }
    }

    /// Reserve capacity for one payload to be written upstream.
    ///
    /// Returning `None` means the upstream cannot accept more data, so the caller must leave those
    /// bytes where they are rather than dropping them.
    pub fn reserve_upstream(&self) -> Option<mpsc::Permit<'_, Bytes>> {
        self.to_upstream_tx.as_ref()?.try_reserve().ok()
    }

    pub fn poll_downstream_payload(&mut self) -> Result<Bytes, mpsc::error::TryRecvError> {
        self.from_upstream_rx.try_recv()
    }

    /// Shut down the upstream write half, forwarding the client's FIN.
    pub fn shutdown_upstream_writes(&mut self) {
        self.to_upstream_tx = None;
    }

    pub fn upstream_writes_shut_down(&self) -> bool {
        self.to_upstream_tx.is_none()
    }
}

async fn send_event(
    event_tx: &mpsc::Sender<UpstreamMsg>,
    id: TcpConnectionId,
    event: UpstreamEvent,
    notify: &Notify,
) {
    let _ = event_tx.send(UpstreamMsg { id, event }).await;
    notify.notify_one();
}

/// Forward everything the upstream sends until EOF, an error, or the stack drops the connection.
async fn read_loop(
    mut reader: OwnedReadHalf,
    payload_tx: mpsc::Sender<Bytes>,
    id: TcpConnectionId,
    event_tx: mpsc::Sender<UpstreamMsg>,
    notify: Arc<Notify>,
) {
    let mut read_buf = BytesMut::with_capacity(UPSTREAM_READ_CHUNK);

    loop {
        read_buf.reserve(UPSTREAM_READ_CHUNK);
        match reader.read_buf(&mut read_buf).await {
            // Dropping `payload_tx` is what tells the stack to FIN the client, once it has
            // forwarded everything still buffered.
            Ok(0) => return,
            Ok(_) => {
                // Awaiting here is the backpressure: a client that stops reading eventually stalls
                // this send, which stalls the read above and closes the upstream receive window.
                if payload_tx.send(read_buf.split().freeze()).await.is_err() {
                    return;
                }
                notify.notify_one();
            }
            Err(err) => {
                log::debug!("{id:?} - failed to read from upstream: {err}");
                return send_event(&event_tx, id, UpstreamEvent::Error, &notify).await;
            }
        }
    }
}

/// Write everything the client sends upstream, then half-close when the stack drops the sender.
async fn write_loop(
    mut writer: OwnedWriteHalf,
    mut payload_rx: mpsc::Receiver<Bytes>,
    id: TcpConnectionId,
    event_tx: mpsc::Sender<UpstreamMsg>,
    notify: Arc<Notify>,
) {
    while let Some(payload) = payload_rx.recv().await {
        if let Err(err) = writer.write_all(&payload).await {
            log::debug!("{id:?} - failed to write to upstream: {err}");
            return send_event(&event_tx, id, UpstreamEvent::Error, &notify).await;
        }
        // A freed slot may let the stack move more out of smoltcp's receive buffer.
        notify.notify_one();
    }

    let _ = writer.shutdown().await;
}

/// The far side of a [`UpstreamSocket::detached`], for tests to play the upstream with.
#[cfg(test)]
pub struct DetachedUpstream {
    pub to_upstream_rx: mpsc::Receiver<Bytes>,
    pub from_upstream_tx: mpsc::Sender<Bytes>,
}

#[cfg(test)]
impl UpstreamSocket {
    /// An upstream socket with no socket behind it, so a test can drive both directions by hand.
    pub fn detached() -> (Self, DetachedUpstream) {
        let (to_upstream_tx, to_upstream_rx) = mpsc::channel::<Bytes>(TCP_DATA_CHANNEL_CAPACITY);
        let (from_upstream_tx, from_upstream_rx) = mpsc::channel::<Bytes>(TCP_DATA_CHANNEL_CAPACITY);

        let socket = Self {
            to_upstream_tx: Some(to_upstream_tx),
            from_upstream_rx,
            _supervisor: AbortOnDropHandle::new(tokio::spawn(std::future::pending())),
        };

        (
            socket,
            DetachedUpstream {
                to_upstream_rx,
                from_upstream_tx,
            },
        )
    }
}
