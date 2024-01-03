use libc::c_void;
use std::{
    io, mem,
    net::SocketAddr,
    ptr,
    sync::{Arc, Weak},
};
use tokio::sync::{broadcast, mpsc, oneshot, watch};

use super::data::SwiftDataArray;
use std::pin::Pin;

#[derive(PartialEq)]
enum SocketState {
    Ready,
    NotReady,
}

pub struct ExcludedUdpSession {
    // Address of receiving host
    addr: SocketAddr,

    // Swift context
    swift_ctx: *const libc::c_void,

    // Channel for receiving data that should be forwarded to the swift side
    send_queue_rx: mpsc::Receiver<SwiftDataArray>,
    // Size of send queue
    send_queue_size: usize,

    // Receive updates from the Swift side about socket state.
    socket_state_rx: watch::Receiver<SocketState>,

    // Structure used byt swift to communicate with this struct asynchronously, a pointer to it
    // will be passed to the Swift side.
    swift_handle: Box<SwiftHandle>,
    // TODO: evaluate if `send_completion_tx` is absolutely necessary
    // Send completion channel. It's kept here so that it can be deallocated appropriately if a
    // send operation is not finished when the session is being dropped.
    // send_completion_tx: Option<Box<Pin<oneshot::Sender>>>,
}

// It's safe since the pointer in `swift_ctx` is valid from any thread as long as the lifecycle
// rules are upheld - ExcludeUdpSession is created with a valid pointer and it's deallocated.
unsafe impl Send for ExcludedUdpSession {}

impl ExcludedUdpSession {
    pub fn new(
        addr: SocketAddr,
        packet_tunnel_ptr: *const libc::c_void,
        send_queue_size: usize,
    ) -> (Self, ExcludedUdpHandle, ExcludedUdpReceiver) {
        let (send_queue_tx, send_queue_rx) = mpsc::channel(send_queue_size);
        let (socket_state_tx, socket_state_rx) = watch::channel(SocketState::NotReady);
        let (recv_data_tx, recv_data_rx) = mpsc::channel(1);
        let addr_string = format!("{}", addr.ip());

        let mut session = Self {
            addr,
            swift_ctx: std::ptr::null(),
            swift_handle: Box::new(SwiftHandle {
                socket_state_tx,
                recv_data_tx,
            }),
            send_queue_rx,
            send_queue_size,
            socket_state_rx,
        };

        let handle = ExcludedUdpHandle { tx: send_queue_tx };
        let reader = ExcludedUdpReceiver { recv_data_rx };


        let swift_ctx = unsafe {
            // SAFETY: addr_string mustn't be moved.
            let addr_ptr = addr_string.as_bytes().as_ptr();
            let addr_len = addr_string.as_bytes().len();
            let swift_handle_ptr = &*session.swift_handle as *const _ as *const _;

            swift_nw_excluded_udp_session_create(
                addr_ptr,
                addr_len,
                addr.port(),
                packet_tunnel_ptr,
                swift_handle_ptr,
            )
        };
        session.swift_ctx = swift_ctx;

        (session, handle, reader)
    }

    pub async fn run(&mut self) {
        self.wait_for_ready_state().await;
        self.handle_messages().await;
    }

    async fn wait_for_ready_state(&mut self) {
        if *self.socket_state_rx.borrow() == SocketState::Ready {
            return;
        }

        let _ = self
            .socket_state_rx
            .wait_for(|state| *state == SocketState::Ready)
            .await;
    }

    async fn handle_messages(&mut self) {
        loop {
            let Self {
                ref mut socket_state_rx,
                ref mut send_queue_rx,
                ..
            } = self;

            tokio::select! {
                biased;
                _socket_state_change = socket_state_rx.changed() => {
                    self.wait_for_ready_state().await;
                }

                outgoing_data = send_queue_rx.recv() => {
                    let Some(outgoing_data) = outgoing_data else {
                        log::trace!("UDP session finished");
                        return;
                    };
                    let _ = self.send_data(outgoing_data).await;
                }
            }
        }
    }

    async fn send_data(&mut self, data: SwiftDataArray) -> io::Result<()> {
        if data.len() == 0 {
            return Ok(());
        }

        let (completion_tx, completion_rx) = oneshot::channel::<io::Result<()>>();
        let completion_tx = Box::new(completion_tx);

        unsafe {
            swift_nw_excluded_udp_session_send(
                self.swift_ctx,
                data.into_raw(),
                Box::into_raw(completion_tx) as *mut _,
            );
        }
        if let Ok(Err(err)) = completion_rx.await {
            log::error!("failed to send data: {}", err);
        }

        Ok(())
    }
}

impl Drop for ExcludedUdpSession {
    fn drop(&mut self) {
        unsafe { swift_nw_excluded_udp_session_destroy(self.swift_ctx) }
    }
}

#[derive(Clone)]
pub struct ExcludedUdpHandle {
    tx: mpsc::Sender<SwiftDataArray>,
}
impl ExcludedUdpHandle {
    pub(crate) async fn send(&self, data: SwiftDataArray) {
        let _ = self.tx.send(data).await;
    }
}

pub struct ExcludedUdpReceiver {
    recv_data_rx: mpsc::Receiver<io::Result<SwiftDataArray>>,
}


impl ExcludedUdpReceiver {
    pub async fn recv(&mut self) -> io::Result<SwiftDataArray> {
        self.recv_data_rx
            .recv()
            .await
            .ok_or(io::Error::from_raw_os_error(-1))?
    }
}

struct SwiftHandle {
    socket_state_tx: watch::Sender<SocketState>,
    recv_data_tx: mpsc::Sender<io::Result<SwiftDataArray>>,
}

impl SwiftHandle {
    fn set_state(&self, state: SocketState) {
        let _ = self.socket_state_tx.send(state);
    }

    fn recv_data(&self, result: io::Result<SwiftDataArray>) {
        let _ = self.recv_data_tx.blocking_send(result);
    }
}

extern "C" {
    fn swift_nw_excluded_udp_session_create(
        addr_ptr: *const u8,
        addr_len: usize,
        port: u16,
        packetTunnelPtr: *const libc::c_void,
        rustContext: *const libc::c_void,
    ) -> *mut libc::c_void;

    fn swift_nw_excluded_udp_session_send(
        session: *const libc::c_void,
        data: *const libc::c_void,
        completion: *mut libc::c_void,
    );

    fn swift_nw_excluded_udp_session_destroy(session: *const libc::c_void);
}

#[no_mangle]
pub extern "C" fn excluded_udp_session_send_complete(chan_ptr: *mut libc::c_void, status: i32) {
    // SAFETY: this function must be called with a valid pointer to the completion channel
    let tx: Box<oneshot::Sender<io::Result<()>>> = unsafe { Box::from_raw(chan_ptr as *mut _) };

    let result = if status == 0 {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(status))
    };

    let _ = tx.send(result);
}

/// Informs the UDP session actor that the current connection is ready to be used.
/// The context pointer is expected to be a valid pointer to a `Box<SwiftHandle>`.
#[no_mangle]
pub extern "C" fn excluded_udp_session_ready(ctx: *mut libc::c_void) {
    unsafe {
        with_swift_handle(ctx, |handle| {
            handle.set_state(SocketState::Ready);
        })
    };
}

/// Informs the UDP session actor that the current connection is not ready to be used.
/// The context pointer is expected to be a valid pointer to a `Box<SwiftHandle>`.
#[no_mangle]
pub extern "C" fn excluded_udp_session_not_ready(ctx: *mut libc::c_void) {
    unsafe {
        with_swift_handle(ctx, |handle| {
            handle.set_state(SocketState::NotReady);
        })
    };
}

/// To be executed when the UDP session receives data.
///
/// The context pointer is expected to be a valid pointer to a `Box<SwiftHandle>`. This function
/// takes ownership over the `data` object, it shouldn't be used from the Swift side again.
#[no_mangle]
pub extern "C" fn excluded_udp_session_recv(ctx: *mut libc::c_void, data: *mut libc::c_void) {
    let data: SwiftDataArray = unsafe { SwiftDataArray::from_raw(data) };
    unsafe {
        with_swift_handle(ctx, |handle| {
            handle.recv_data(Ok(data));
        })
    };
}

#[no_mangle]
pub extern "C" fn excluded_udp_session_recv_err(ctx: *mut libc::c_void, error_code: i32) {
    // SAFETY: expecting `ctx` pointer to be a valid refernce to a `Box<SwiftHandle>`
    unsafe {
        with_swift_handle(ctx, |handle| {
            handle.recv_data(Err(io::Error::from_raw_os_error(error_code)));
        })
    };
}


unsafe fn with_swift_handle(ptr: *mut libc::c_void, mut f: impl FnOnce(&mut SwiftHandle)) {
    let mut handle: Box<SwiftHandle> = unsafe { Box::from_raw(ptr as *mut _) };
    f(&mut handle);
    mem::forget(handle);
}
