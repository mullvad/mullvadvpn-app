use std::{
    collections::VecDeque,
    mem::size_of,
    pin::Pin,
    task::{ready, Context, Poll},
    time::Duration,
};

use nix::{
    fcntl,
    sys::socket::{socket, AddressFamily, SockFlag, SockType},
};
use std::{
    fs::File,
    io::{self, Read, Write},
    os::fd::{AsRawFd, RawFd},
};

use super::data::{rt_msghdr_short, MessageType, RouteMessage};

use tokio::io::{unix::AsyncFd, AsyncWrite, AsyncWriteExt};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to open routing socket")]
    OpenSocket(#[source] io::Error),
    #[error("Failed to write to routing socket")]
    Write(#[source] io::Error),
    #[error("Failed to read from routing socket")]
    Read(#[source] io::Error),
    #[error("Received a message that's too small")]
    MessageTooSmall(usize),
    #[error("Failed to receive response to route message")]
    ResponseTimeout,
}

type Result<T> = std::result::Result<T, Error>;

const RESPONSE_TIMEOUT: Duration = Duration::from_secs(10);

/// Wraps a `PF_ROUTE` socket, keeps track of sent message IDs, and facilitates sending and
/// receiving [route socket messages](#RouteMessage)
pub struct RoutingSocket {
    socket: RoutingSocketInner,
    seq: i32,
    // buffers up messages received whilst waiting on a response
    // TODO: might we want to limit the max size of this?
    buf: VecDeque<Vec<u8>>,
    own_pid: i32,
}

impl RoutingSocket {
    pub fn new() -> Result<Self> {
        Ok(Self {
            socket: RoutingSocketInner::new().map_err(Error::OpenSocket)?,
            seq: 1,
            buf: Default::default(),
            own_pid: std::process::id().try_into().unwrap(),
        })
    }

    pub async fn recv_msg(&mut self, mut buf: &mut [u8]) -> Result<usize> {
        if let Some(buffered_msg) = self.buf.pop_front() {
            let bytes_written = buf.write(&buffered_msg).map_err(Error::Write)?;
            return Ok(bytes_written);
        }
        self.read_next_msg(buf).await
    }

    async fn read_next_msg(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.socket.read(buf).await.map_err(Error::Read)
    }

    pub async fn send_route_message(
        &mut self,
        message: &RouteMessage,
        message_type: MessageType,
    ) -> Result<Vec<u8>> {
        let (msg, seq) = self.next_route_msg(message, message_type);
        match self.socket.write(&msg).await {
            Ok(_) => tokio::time::timeout(RESPONSE_TIMEOUT, self.wait_for_response(seq))
                .await
                .map_err(|_| Error::ResponseTimeout)?,
            Err(err) => Err(Error::Write(err)),
        }
    }

    pub async fn wait_for_response(&mut self, response_num: i32) -> Result<Vec<u8>> {
        loop {
            let mut buffer = vec![0u8; 2048];
            // do not truncate the buffer - trailing empty bytes won't be written but will be
            // assumed in the data format.
            let bytes_read = self.read_next_msg(&mut buffer).await?;

            {
                let header = rt_msghdr_short::from_bytes(buffer.as_slice())
                    .ok_or(Error::MessageTooSmall(bytes_read))?;

                if header.rtm_pid == self.own_pid && response_num == header.rtm_seq {
                    return Ok(buffer);
                }
            }

            self.buf.push_back(buffer);
        }
    }

    fn next_route_msg(&mut self, message: &RouteMessage, msg_type: MessageType) -> (Vec<u8>, i32) {
        let seq = self.seq;
        self.seq = seq.wrapping_add(1);

        let (header, payload) = message.payload(msg_type, seq, self.own_pid);
        let mut msg_buffer = vec![0u8; header.rtm_msglen.into()];

        // SAFETY: `msg_buffer` is guaranteed to be at least as large as `rt_msghdr`.
        unsafe {
            std::ptr::copy_nonoverlapping(
                &header as *const _ as *const u8,
                msg_buffer.as_mut_ptr(),
                size_of::<super::data::rt_msghdr>(),
            );
        }
        let mut sockaddr_buf = &mut msg_buffer[std::mem::size_of::<super::data::rt_msghdr>()..];
        for socket_addr in payload {
            sockaddr_buf
                .write_all(socket_addr.as_slice())
                .expect("faled to write socket address into message buffer");
        }
        (msg_buffer, header.rtm_seq)
    }
}

struct RoutingSocketInner {
    // storing the file handle in a std::file::File automagically provides sane io::{Write,
    // Read} and Drop implementations.
    socket: AsyncFd<File>,
}

impl RoutingSocketInner {
    fn new() -> io::Result<Self> {
        let fd = socket(AddressFamily::Route, SockType::Raw, SockFlag::empty(), None)?;
        let _ = fcntl::fcntl(
            fd.as_raw_fd(),
            fcntl::FcntlArg::F_SETFL(fcntl::OFlag::O_NONBLOCK),
        )?;
        let socket = File::from(fd);
        Ok(Self {
            socket: AsyncFd::new(socket)?,
        })
    }

    async fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let mut guard = self.socket.readable().await?;
            match guard.try_io(|sock| sock.get_ref().read(out)) {
                Ok(result) => return result,
                Err(_err) => continue,
            }
        }
    }
}

impl AsRawFd for RoutingSocketInner {
    fn as_raw_fd(&self) -> RawFd {
        self.socket.as_raw_fd()
    }
}

impl AsyncWrite for RoutingSocketInner {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            let mut guard = ready!(self.socket.poll_write_ready(cx))?;

            match guard.try_io(|inner| inner.get_ref().write(buf)) {
                Ok(result) => return Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        // tcp flush is a no-op
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        // no need for a shutdown on the routing socket
        Poll::Ready(Ok(()))
    }
}
