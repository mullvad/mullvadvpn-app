use libc::c_void;
use std::{
    io::{self, Result},
    sync::{Arc, atomic::{AtomicBool, self}},
    task::Poll,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc,
};

fn connection_closed_err() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "TCP connection closed")
}

extern "C" {
    /// Called when there is data to send on the TCP connection.
    /// The TCP connection must write data on the wire, then call the `handle_sent` function.
    pub fn swift_nw_tcp_connection_send(
        connection: *const libc::c_void,
        data: *const libc::c_void,
        data_len: usize,
        sender: *const libc::c_void,
    );

    /// Called when there is data to read on the TCP connection.
    /// The TCP connection must read data from the wire, then call the `handle_read` function.
    pub fn swift_nw_tcp_connection_read(
        connection: *const libc::c_void,
        sender: *const libc::c_void,
    );

    /// Called when the preshared post quantum key is ready.
    /// `raw_preshared_key` might be NULL if the key negotiation failed.
    pub fn swift_post_quantum_key_ready(
        raw_packet_tunnel: *const c_void,
        raw_preshared_key: *const u8,
        raw_ephemeral_private_key: *const u8,
    );
}

unsafe impl Send for IosTcpProvider {}

pub struct IosTcpProvider {
    write_tx: mpsc::UnboundedSender<usize>,
    write_rx: mpsc::UnboundedReceiver<usize>,
    read_tx: mpsc::UnboundedSender<Box<[u8]>>,
    read_rx: mpsc::UnboundedReceiver<Box<[u8]>>,
    tcp_connection: *const c_void,
    read_in_progress: bool,
    write_in_progress: bool,
    shutdown: Arc<AtomicBool>,
}

pub struct IosTcpShutdownHandle {
    shutdown: Arc<AtomicBool>,
}

impl IosTcpProvider {
    pub unsafe fn new(tcp_connection: *const c_void) -> (Self, IosTcpShutdownHandle) {
        let (tx, rx) = mpsc::unbounded_channel();
        let (recv_tx, recv_rx) = mpsc::unbounded_channel();
        let shutdown = Arc::new(AtomicBool::new(false));

        (
            Self {
                write_tx: tx,
                write_rx: rx,
                read_tx: recv_tx,
                read_rx: recv_rx,
                tcp_connection,
                read_in_progress: false,
                write_in_progress: false,
                shutdown: shutdown.clone(),
            },
            IosTcpShutdownHandle { shutdown },
        )
    }

    fn is_shutdown(&self) -> bool {
        self.shutdown.load(atomic::Ordering::SeqCst)
    }
}

impl IosTcpShutdownHandle {
    pub fn shutdown(&self) {
        self.shutdown.store(true, atomic::Ordering::SeqCst);
    }
}

impl AsyncWrite for IosTcpProvider {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize>> {
        let raw_sender = Box::into_raw(Box::new(self.write_tx.clone()));

        match self.write_rx.poll_recv(cx) {
            std::task::Poll::Ready(Some(bytes_sent)) => {
                self.write_in_progress = false;
                Poll::Ready(Ok(bytes_sent))
            }
            std::task::Poll::Ready(None) => {
                self.write_in_progress = false;
                Poll::Ready(Err(connection_closed_err()))
            }
            std::task::Poll::Pending => {
                if self.is_shutdown() {
                    return Poll::Ready(Err(connection_closed_err()));
                }
                if self.write_in_progress {
                    return std::task::Poll::Pending;
                }
                self.write_in_progress = true;
                unsafe {
                    swift_nw_tcp_connection_send(
                        self.tcp_connection,
                        buf.as_ptr() as _,
                        buf.len(),
                        raw_sender as _,
                    );
                }
                std::task::Poll::Pending
            }
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
}
impl AsyncRead for IosTcpProvider {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let raw_sender = Box::into_raw(Box::new(self.read_tx.clone()));
        if self.is_shutdown() {
            return Poll::Ready(Err(connection_closed_err()));
        }

        match self.read_rx.poll_recv(cx) {
            std::task::Poll::Ready(Some(data)) => {
                buf.put_slice(&data);
                self.read_in_progress = false;
                Poll::Ready(Ok(()))
            }
            std::task::Poll::Ready(None) => {
                self.read_in_progress = false;
                Poll::Ready(Err(connection_closed_err()))
            }
            std::task::Poll::Pending => {
                if self.read_in_progress {
                    return std::task::Poll::Pending;
                }
                self.read_in_progress = true;
                unsafe {
                    swift_nw_tcp_connection_read(self.tcp_connection, raw_sender as _);
                }

                std::task::Poll::Pending
            }
        }
    }
}
