use libc::c_void;
use std::{
    io::{self, Result},
    sync::{Arc, Mutex, MutexGuard, Weak},
    task::{Poll, Waker},
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

    /// Called when the preshared post quantum key is ready,
    /// or when a Daita peer has been successfully requested.
    /// `raw_preshared_key` will be NULL if:
    /// - The post quantum key negotiation failed
    /// - A Daita peer has been requested without enabling post quantum keys.
    pub fn swift_ephemeral_peer_ready(
        raw_packet_tunnel: *const c_void,
        raw_preshared_key: *const u8,
        raw_ephemeral_private_key: *const u8,
        daita_enabled: bool,
    );
}

unsafe impl Send for IosTcpProvider {}

pub struct IosTcpProvider {
    write_tx: Arc<mpsc::UnboundedSender<usize>>,
    write_rx: mpsc::UnboundedReceiver<usize>,
    read_tx: Arc<mpsc::UnboundedSender<Box<[u8]>>>,
    read_rx: mpsc::UnboundedReceiver<Box<[u8]>>,
    tcp_connection: Arc<Mutex<ConnectionContext>>,
    read_in_progress: bool,
    write_in_progress: bool,
}

pub struct IosTcpShutdownHandle {
    context: Arc<Mutex<ConnectionContext>>,
}

pub struct ConnectionContext {
    waker: Option<Waker>,
    tcp_connection: Option<*const c_void>,
}

unsafe impl Send for ConnectionContext {}

impl IosTcpProvider {
    /// # Safety
    /// `connection` must be pointing to a valid instance of a `NWTCPConnection`, created by the
    /// `PacketTunnelProvider`
    pub unsafe fn new(connection: Arc<Mutex<ConnectionContext>>) -> (Self, IosTcpShutdownHandle) {
        let (tx, rx) = mpsc::unbounded_channel();
        let (recv_tx, recv_rx) = mpsc::unbounded_channel();

        (
            Self {
                write_tx: Arc::new(tx),
                write_rx: rx,
                read_tx: Arc::new(recv_tx),
                read_rx: recv_rx,
                tcp_connection: connection.clone(),
                read_in_progress: false,
                write_in_progress: false,
            },
            IosTcpShutdownHandle {
                context: connection,
            },
        )
    }

    fn maybe_set_waker(new_waker: Waker, connection: &mut MutexGuard<'_, ConnectionContext>) {
        connection.waker = Some(new_waker);
    }
}

impl IosTcpShutdownHandle {
    pub fn shutdown(self) {
        let Ok(mut context) = self.context.lock() else {
            return;
        };

        context.tcp_connection = None;
        if let Some(waker) = context.waker.take() {
            waker.wake();
        }
        std::mem::drop(context);
    }
}

impl AsyncWrite for IosTcpProvider {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize>> {
        let connection_lock = self.tcp_connection.clone();
        let Ok(mut connection) = connection_lock.lock() else {
            return Poll::Ready(Err(connection_closed_err()));
        };
        let Some(tcp_ptr) = connection.tcp_connection else {
            return Poll::Ready(Err(connection_closed_err()));
        };
        Self::maybe_set_waker(cx.waker().clone(), &mut connection);

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
                if !self.write_in_progress {
                    let raw_sender = Weak::into_raw(Arc::downgrade(&self.write_tx));
                    unsafe {
                        swift_nw_tcp_connection_send(
                            tcp_ptr,
                            buf.as_ptr() as _,
                            buf.len(),
                            raw_sender as _,
                        );
                    }
                    self.write_in_progress = true;
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
        let connection_lock = self.tcp_connection.clone();
        let Ok(mut connection) = connection_lock.lock() else {
            return Poll::Ready(Err(connection_closed_err()));
        };
        let Some(tcp_ptr) = connection.tcp_connection else {
            return Poll::Ready(Err(connection_closed_err()));
        };
        Self::maybe_set_waker(cx.waker().clone(), &mut connection);

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
                if !self.read_in_progress {
                    let raw_sender = Weak::into_raw(Arc::downgrade(&self.read_tx));
                    unsafe {
                        swift_nw_tcp_connection_read(tcp_ptr, raw_sender as _);
                    }
                    self.read_in_progress = true;
                }
                Poll::Pending
            }
        }
    }
}

impl ConnectionContext {
    pub fn new(tcp_connection: *const c_void) -> Self {
        Self {
            tcp_connection: Some(tcp_connection),
            waker: None,
        }
    }

    pub fn shutdown(&mut self) {
        self.tcp_connection = None;
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}
