use std::{
    ffi::CStr,
    future::Future,
    io::{self},
    pin::Pin,
    task::{ready, Poll},
    time::Duration,
};
use tokio::io::{AsyncRead, AsyncWrite};

use super::EphemeralPeerParameters;

fn connection_closed_err() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "TCP connection closed")
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct WgTcpConnectionFunctions {
    pub open_fn:
        unsafe extern "C" fn(tunnelHandle: i32, address: *const libc::c_char, timeout: u64) -> i32,
    pub close_fn: unsafe extern "C" fn(tunnelHandle: i32, socketHandle: i32) -> i32,
    pub recv_fn:
        unsafe extern "C" fn(tunnelHandle: i32, socketHandle: i32, data: *mut u8, len: i32) -> i32,
    pub send_fn: unsafe extern "C" fn(
        tunnelHandle: i32,
        socketHandle: i32,
        data: *const u8,
        len: i32,
    ) -> i32,
}

impl WgTcpConnectionFunctions {
    /// # Safety
    /// This function is safe to call so long as the function pointer is valid for its declared
    /// signature.
    pub unsafe fn open(&self, tunnel_handle: i32, address: *const u8, timeout: u64) -> i32 {
        // SAFETY: See above
        unsafe { (self.open_fn)(tunnel_handle, address.cast(), timeout) }
    }

    /// # Safety
    /// This function is safe to call so long as the function pointer is valid for its declared
    /// signature.
    pub unsafe fn close(&self, tunnel_handle: i32, socket_handle: i32) -> i32 {
        // SAFETY: See above
        unsafe { (self.close_fn)(tunnel_handle, socket_handle) }
    }

    /// # Safety
    /// This function is safe to call so long as the function pointer is valid for its declared
    /// signature.
    pub unsafe fn receive(&self, tunnel_handle: i32, socket_handle: i32, data: &mut [u8]) -> i32 {
        let ptr = data.as_mut_ptr();
        let len = data
            .len()
            .try_into()
            .expect("Cannot receive a buffer larger than 2GiB");
        // SAFETY: See notes for this function
        unsafe { (self.recv_fn)(tunnel_handle, socket_handle, ptr.cast(), len) }
    }

    /// # Safety
    /// This function is safe to call so long as the function pointer is valid for its declared
    /// signature.
    pub unsafe fn send(&self, tunnel_handle: i32, socket_handle: i32, data: &[u8]) -> i32 {
        let ptr = data.as_ptr();
        let len = data
            .len()
            .try_into()
            .expect("Cannot send a buffer larger than 2GiB");
        // SAFETY: See notes for this function
        unsafe { (self.send_fn)(tunnel_handle, socket_handle, ptr.cast(), len) }
    }
}

#[derive(Clone)]
pub struct IosTcpProvider {
    tunnel_handle: i32,
    timeout: Duration,
    funcs: WgTcpConnectionFunctions,
}

type InFlightIoTask = Option<Pin<Box<tokio::task::JoinHandle<io::Result<Vec<u8>>>>>>;

pub struct IosTcpConnection {
    tunnel_handle: i32,
    socket_handle: i32,
    funcs: WgTcpConnectionFunctions,
    in_flight_read: InFlightIoTask,
    in_flight_write: InFlightIoTask,
}

#[derive(Debug)]
pub enum WgTcpError {
    /// Failed to open the socket
    Open,
    /// Panicked during opening of the socket
    Panic,
}

impl IosTcpProvider {
    pub fn new(tunnel_handle: i32, params: EphemeralPeerParameters) -> Self {
        Self {
            tunnel_handle,
            timeout: Duration::from_secs(params.peer_exchange_timeout),
            funcs: params.funcs,
        }
    }

    pub async fn connect(&self, address: &'static CStr) -> Result<IosTcpConnection, WgTcpError> {
        let tunnel_handle = self.tunnel_handle;
        let timeout = self.timeout.as_secs();
        let funcs = self.funcs;
        // SAFETY:
        // The `open_fn` function pointer in `funcs` must be valid.
        let result = tokio::task::spawn_blocking(move || unsafe {
            funcs.open(tunnel_handle, address.as_ptr() as *const _, timeout)
        })
        .await
        .map_err(|_| WgTcpError::Panic)?;

        if result < 0 {
            return Err(WgTcpError::Open);
        }

        Ok(IosTcpConnection {
            tunnel_handle,
            socket_handle: result,
            funcs: self.funcs,
            in_flight_read: None,
            in_flight_write: None,
        })
    }
}

impl Drop for IosTcpConnection {
    fn drop(&mut self) {
        // Safety:
        // `funcs.close_fn` must be a valid function pointer.
        unsafe { self.funcs.close(self.tunnel_handle, self.socket_handle) };
    }
}

impl AsyncWrite for IosTcpConnection {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        // If task is already spawned, poll it
        if let Some(handle) = &mut self.in_flight_write {
            let result = match ready!(handle.as_mut().poll(cx)) {
                Ok(Ok(written)) => Ok(written.len()),
                Ok(Err(e)) => Err(e),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Write task panicked")),
            };
            // important to clear the in flight write here.
            self.in_flight_write = None;
            Poll::Ready(result)
        } else {
            // if no write task has been spawned, spawn one
            let tunnel_handle = self.tunnel_handle;
            let socket_handle = self.socket_handle;
            // The data has to be cloned, since it will be moved into another thread and it has to
            // outlive this function call.
            let data = buf.to_vec();
            let funcs = self.funcs;
            let task = tokio::task::spawn_blocking(move || {
                // Safety:
                // `funcs.send_fn` must be a valid function pointer.
                let result = unsafe { funcs.send(tunnel_handle, socket_handle, data.as_slice()) };
                if result < 0 {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Write error: {}", result),
                    ))
                } else {
                    Ok(data[..result as usize].to_vec())
                }
            });

            self.in_flight_write = Some(Box::pin(task));
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
}

impl AsyncRead for IosTcpConnection {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        // If task is already spawned, poll it
        if let Some(handle) = &mut self.in_flight_read {
            let result = match ready!(handle.as_mut().poll(cx)) {
                Ok(Ok(data)) => {
                    // We are assuming that the buffer has not been used for anything else between
                    // spawning the task and writing to it now, since we expect `buf.remaining()`
                    // to return the same value between those two points in time.
                    let len = data.len().min(buf.remaining());
                    buf.put_slice(&data[..len]);
                    Ok(())
                }
                Ok(Err(e)) => Err(e),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Read task panicked")),
            };
            // Clear the in-flight read, since the read task finished
            self.in_flight_read = None;
            Poll::Ready(result)
        } else {
            // If no read task has been spawned, spawn one
            let tunnel_handle = self.tunnel_handle;
            let socket_handle = self.socket_handle;
            let funcs = self.funcs;
            let mut buffer = vec![0u8; buf.remaining()];
            let task = tokio::task::spawn_blocking(move || {
                // Safety:
                // `funcs.receive_fn` must be a valid function pointer.
                let result =
                    unsafe { funcs.receive(tunnel_handle, socket_handle, buffer.as_mut_slice()) };
                match result {
                    size @ 1.. => {
                        buffer.truncate(size as usize);
                        Ok(buffer)
                    }

                    errval @ ..0 => Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Read error: {}", errval),
                    )),

                    0 => Err(connection_closed_err()),
                }
            });

            self.in_flight_read = Some(Box::pin(task));
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
