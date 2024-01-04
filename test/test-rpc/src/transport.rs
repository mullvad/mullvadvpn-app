use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{channel::mpsc, SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::Write,
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tarpc::{ClientMessage, Response};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::futures::Notified,
};
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};

use crate::{Error, ServiceRequest, ServiceResponse};

/// How long to wait for the RPC server to start
const CONNECT_TIMEOUT: Duration = Duration::from_secs(300);
const FRAME_TYPE_SIZE: usize = std::mem::size_of::<FrameType>();
const DAEMON_CHANNEL_BUF_SIZE: usize = 16 * 1024;

/// Unique payload that comes with the "handshake" frame
const MULLVAD_SIGNATURE: &[u8] = b"MULLV4D;";

pub enum Frame {
    Handshake,
    TestRunner(Bytes),
    DaemonRpc(Bytes),
}

#[repr(u8)]
enum FrameType {
    Handshake,
    TestRunner,
    DaemonRpc,
}

impl TryFrom<u8> for FrameType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            i if i == FrameType::Handshake as u8 => Ok(FrameType::Handshake),
            i if i == FrameType::TestRunner as u8 => Ok(FrameType::TestRunner),
            i if i == FrameType::DaemonRpc as u8 => Ok(FrameType::DaemonRpc),
            _ => Err(()),
        }
    }
}

pub type GrpcForwarder = tokio::io::DuplexStream;
pub type CompletionHandle = tokio::task::JoinHandle<()>;

#[derive(Debug, Clone)]
pub struct ConnectionHandle {
    handshake_fwd_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<()>>>,
    // True if the connection has received an initial "handshake" frame from the other end.
    is_connected: Arc<AtomicBool>,
    reset_notify: Arc<tokio::sync::Notify>,
}

impl ConnectionHandle {
    /// Returns a new "handshake forwarder" and connection handle.
    fn new() -> (mpsc::UnboundedSender<()>, Self) {
        let (handshake_fwd_tx, handshake_fwd_rx) = mpsc::unbounded();

        (
            handshake_fwd_tx,
            Self {
                handshake_fwd_rx: Arc::new(tokio::sync::Mutex::new(handshake_fwd_rx)),
                is_connected: Self::new_connected_state(false),
                reset_notify: Arc::new(tokio::sync::Notify::new()),
            },
        )
    }

    pub async fn wait_for_server(&mut self) -> Result<(), Error> {
        let mut handshake_fwd = self.handshake_fwd_rx.lock().await;

        log::info!("Waiting for server");

        match tokio::time::timeout(CONNECT_TIMEOUT, handshake_fwd.next()).await {
            Ok(_) => {
                log::info!("Server responded");
                Ok(())
            }
            _ => {
                log::error!("Connection timed out");
                Err(Error::TestRunnerTimeout)
            }
        }
    }

    /// Resets `Self::is_connected`.
    pub async fn reset_connected_state(&self) {
        let mut handshake_fwd = self.handshake_fwd_rx.lock().await;
        // empty stream
        while let Ok(Some(_)) = handshake_fwd.try_next() {}

        self.is_connected.store(false, Ordering::SeqCst);
        self.reset_notify.notify_waiters();
    }

    /// Returns a future that is notified when `reset_connected_state` is called.
    pub fn notified_reset(&self) -> Notified<'_> {
        self.reset_notify.notified()
    }

    fn connected_state(&self) -> Arc<AtomicBool> {
        self.is_connected.clone()
    }

    fn new_connected_state(initial: bool) -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(initial))
    }
}

type ServerTransports = (
    tarpc::transport::channel::UnboundedChannel<
        ClientMessage<ServiceRequest>,
        Response<ServiceResponse>,
    >,
    GrpcForwarder,
    CompletionHandle,
);

pub fn create_server_transports(
    serial_stream: impl AsyncRead + AsyncWrite + Unpin + Send + 'static,
) -> ServerTransports {
    let (runner_forwarder_1, runner_forwarder_2) = tarpc::transport::channel::unbounded();

    let (daemon_rx, mullvad_daemon_forwarder) = tokio::io::duplex(DAEMON_CHANNEL_BUF_SIZE);

    let (handshake_tx, handshake_rx) = mpsc::unbounded();

    let _ = handshake_tx.unbounded_send(());

    let completion_handle = tokio::spawn(async move {
        if let Err(error) = forward_messages(
            serial_stream,
            runner_forwarder_2,
            mullvad_daemon_forwarder,
            (handshake_tx, handshake_rx),
            None,
            // The server needs to be init to connected, or it will skip things it shouldn't
            ConnectionHandle::new_connected_state(true),
        )
        .await
        {
            log::error!(
                "forward_messages stopped due an error: {}",
                display_chain(error)
            );
        } else {
            log::debug!("forward_messages stopped");
        }
    });

    (runner_forwarder_1, daemon_rx, completion_handle)
}

pub fn create_client_transports(
    serial_stream: impl AsyncRead + AsyncWrite + Unpin + Send + 'static,
) -> Result<ClientTransports, Error> {
    let (runner_forwarder_1, runner_forwarder_2) = tarpc::transport::channel::unbounded();

    let (daemon_rx, mullvad_daemon_forwarder) = tokio::io::duplex(DAEMON_CHANNEL_BUF_SIZE);

    let (handshake_tx, handshake_rx) = mpsc::unbounded();

    let (handshake_fwd_tx, conn_handle) = ConnectionHandle::new();

    let _ = handshake_tx.unbounded_send(());

    let connected_state = conn_handle.connected_state();

    let completion_handle = tokio::spawn(async move {
        if let Err(error) = forward_messages(
            serial_stream,
            runner_forwarder_1,
            mullvad_daemon_forwarder,
            (handshake_tx, handshake_rx),
            Some(handshake_fwd_tx),
            connected_state,
        )
        .await
        {
            log::error!(
                "forward_messages stopped due an error: {}",
                display_chain(error)
            );
        } else {
            log::debug!("forward_messages stopped");
        }
    });

    Ok((
        runner_forwarder_2,
        daemon_rx,
        conn_handle,
        completion_handle,
    ))
}

type ClientTransports = (
    tarpc::transport::channel::UnboundedChannel<
        Response<ServiceResponse>,
        ClientMessage<ServiceRequest>,
    >,
    GrpcForwarder,
    ConnectionHandle,
    CompletionHandle,
);

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
enum ForwardError {
    #[error(display = "Failed to deserialize JSON data")]
    DeserializeFailed(#[error(source)] serde_json::Error),

    #[error(display = "Failed to serialize JSON data")]
    SerializeFailed(#[error(source)] serde_json::Error),

    #[error(display = "Serial connection error")]
    SerialConnection(#[error(source)] io::Error),

    #[error(display = "Test runner channel error")]
    TestRunnerChannel(#[error(source)] tarpc::transport::channel::ChannelError),

    #[error(display = "Daemon channel error")]
    DaemonChannel(#[error(source)] io::Error),

    #[error(display = "Handshake error")]
    HandshakeError(#[error(source)] io::Error),
}

async fn forward_messages<
    T: Serialize + Unpin + Send + 'static,
    S: DeserializeOwned + Unpin + Send + 'static,
>(
    serial_stream: impl AsyncRead + AsyncWrite + Unpin + Send + 'static,
    mut runner_forwarder: tarpc::transport::channel::UnboundedChannel<T, S>,
    mullvad_daemon_forwarder: GrpcForwarder,
    mut handshaker: (mpsc::UnboundedSender<()>, mpsc::UnboundedReceiver<()>),
    handshake_fwd: Option<mpsc::UnboundedSender<()>>,
    connected_state: Arc<AtomicBool>,
) -> Result<(), ForwardError> {
    let codec = MultiplexCodec::new(connected_state);
    let mut serial_stream = codec.framed(serial_stream);

    // Needs to be framed to allow empty messages.
    let mut mullvad_daemon_forwarder = LengthDelimitedCodec::new().framed(mullvad_daemon_forwarder);

    loop {
        match futures::future::select(
            futures::future::select(serial_stream.next(), handshaker.1.next()),
            futures::future::select(runner_forwarder.next(), mullvad_daemon_forwarder.next()),
        )
        .await
        {
            futures::future::Either::Left((futures::future::Either::Left((Some(frame), _)), _)) => {
                let frame = frame.map_err(ForwardError::SerialConnection)?;

                //
                // Deserialize frame and send it to one of the channels
                //

                match frame {
                    Frame::TestRunner(data) => {
                        let message = serde_json::from_slice(&data)
                            .map_err(ForwardError::DeserializeFailed)?;
                        runner_forwarder
                            .send(message)
                            .await
                            .map_err(ForwardError::TestRunnerChannel)?;
                    }
                    Frame::DaemonRpc(data) => {
                        mullvad_daemon_forwarder
                            .send(data)
                            .await
                            .map_err(ForwardError::DaemonChannel)?;
                    }
                    Frame::Handshake => {
                        log::trace!("shake: recv");
                        if let Some(shake_fwd) = handshake_fwd.as_ref() {
                            let _ = shake_fwd.unbounded_send(());
                        } else {
                            let _ = handshaker.0.unbounded_send(());
                        }
                    }
                }
            }
            futures::future::Either::Left((futures::future::Either::Right((Some(()), _)), _)) => {
                log::trace!("shake: send");

                // Ping the other end
                serial_stream
                    .send(Frame::Handshake)
                    .await
                    .map_err(ForwardError::HandshakeError)?;
            }
            futures::future::Either::Right((
                futures::future::Either::Left((Some(message), _)),
                _,
            )) => {
                let message = message.map_err(ForwardError::TestRunnerChannel)?;

                //
                // Serialize messages from tarpc channel into frames
                // and send them over the serial connection
                //

                let serialized =
                    serde_json::to_vec(&message).map_err(ForwardError::SerializeFailed)?;
                serial_stream
                    .send(Frame::TestRunner(serialized.into()))
                    .await
                    .map_err(ForwardError::SerialConnection)?;
            }
            futures::future::Either::Right((
                futures::future::Either::Right((Some(data), _)),
                _,
            )) => {
                let data = data.map_err(ForwardError::DaemonChannel)?;

                //
                // Forward whatever the heck this is
                //

                serial_stream
                    .send(Frame::DaemonRpc(data.into()))
                    .await
                    .map_err(ForwardError::SerialConnection)?;
            }
            futures::future::Either::Right((futures::future::Either::Right((None, _)), _)) => {
                //
                // Force management interface socket to close
                //
                let _ = serial_stream.send(Frame::DaemonRpc(Bytes::new())).await;

                break Ok(());
            }
            _ => {
                break Ok(());
            }
        }
    }
}

const MULTIPLEX_LEN_DELIMITED_HEADER_SIZE: usize = 4;

#[derive(Default, Debug, Clone)]
pub struct MultiplexCodec {
    len_delim_codec: LengthDelimitedCodec,
    has_connected: Arc<AtomicBool>,
}

impl MultiplexCodec {
    fn new(has_connected: Arc<AtomicBool>) -> Self {
        let mut codec_builder = LengthDelimitedCodec::builder();

        codec_builder.length_field_length(MULTIPLEX_LEN_DELIMITED_HEADER_SIZE);

        Self {
            has_connected,
            len_delim_codec: codec_builder.new_codec(),
        }
    }

    fn decode_frame(mut frame: BytesMut) -> Result<Frame, io::Error> {
        if frame.len() < FRAME_TYPE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "frame does not contain frame type",
            ));
        }

        let mut type_bytes = frame.split_to(FRAME_TYPE_SIZE);
        let frame_type = FrameType::try_from(type_bytes.get_u8())
            .map_err(|_err| io::Error::new(io::ErrorKind::InvalidInput, "invalid frame type"))?;

        match frame_type {
            FrameType::Handshake => Ok(Frame::Handshake),
            FrameType::TestRunner => Ok(Frame::TestRunner(frame.into())),
            FrameType::DaemonRpc => Ok(Frame::DaemonRpc(frame.into())),
        }
    }

    fn encode_frame(
        &mut self,
        frame_type: FrameType,
        bytes: Option<Bytes>,
        dst: &mut BytesMut,
    ) -> Result<(), io::Error> {
        let mut buffer = BytesMut::new();
        if let Some(bytes) = bytes {
            buffer.reserve(bytes.len() + FRAME_TYPE_SIZE);
            buffer.put_u8(frame_type as u8);
            // TODO: implement without copying
            buffer.put(&bytes[..]);
        } else {
            buffer.reserve(FRAME_TYPE_SIZE);
            buffer.put_u8(frame_type as u8);
        }
        self.len_delim_codec.encode(buffer.into(), dst)
    }

    fn decode_inner(&mut self, src: &mut BytesMut) -> Result<Option<Frame>, io::Error> {
        self.skip_noise(src);
        if !self.has_connected.load(Ordering::SeqCst) {
            return Ok(None);
        }
        let frame = self.len_delim_codec.decode(src)?;
        frame.map(Self::decode_frame).transpose()
    }

    fn skip_noise(&mut self, src: &mut BytesMut) {
        // The test runner likes to send ^@ once in while. Unclear why,
        // but it probably occurs (sometimes) when it reconnects to the
        // serial device. Ignoring these control characters is safe.
        while src.len() >= 2 {
            if src[0] == b'^' {
                log::debug!("ignoring control character");
                src.advance(2);
                continue;
            }

            // We use a magic constant to ignore any garbage sent before
            // our service starts. The reason is that OVMF sends stuff to
            // our serial device that we don't care about.
            if !self.has_connected.load(Ordering::SeqCst) {
                for (window_i, window) in src.windows(MULLVAD_SIGNATURE.len()).enumerate() {
                    if window == MULLVAD_SIGNATURE {
                        log::debug!("Found conn signature");

                        // Skip to where the first frame begins
                        src.advance(
                            window_i
                                .saturating_sub(FRAME_TYPE_SIZE)
                                .saturating_sub(MULTIPLEX_LEN_DELIMITED_HEADER_SIZE),
                        );

                        self.has_connected.store(true, Ordering::SeqCst);

                        break;
                    }
                }
            }

            break;
        }
    }
}

impl Decoder for MultiplexCodec {
    type Item = Frame;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decode_inner(src)
    }
}

impl Encoder<Frame> for MultiplexCodec {
    type Error = io::Error;

    fn encode(&mut self, frame: Frame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match frame {
            Frame::Handshake => self.encode_frame(
                FrameType::Handshake,
                Some(Bytes::from_static(MULLVAD_SIGNATURE)),
                dst,
            ),
            Frame::TestRunner(bytes) => self.encode_frame(FrameType::TestRunner, Some(bytes), dst),
            Frame::DaemonRpc(bytes) => self.encode_frame(FrameType::DaemonRpc, Some(bytes), dst),
        }
    }
}

fn display_chain(error: impl std::error::Error) -> String {
    let mut s = error.to_string();
    let mut error = &error as &dyn std::error::Error;
    while let Some(source) = error.source() {
        write!(&mut s, "\nCaused by: {}", source).unwrap();
        error = source;
    }
    s
}
