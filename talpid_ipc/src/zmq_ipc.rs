extern crate zmq;

use super::{OnMessage, ErrorKind, Result, ResultExt};
use std::thread;

type IpcServerId = String;

/// The server end of our Inter-Process Communcation implementation.
pub struct Server {
    port: u16,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Server { port: port }
    }

    /// Starts listening to incoming IPC connections on the specified port.
    /// Messages are sent to the `on_message` callback. If anything went wrong
    /// when reading the message, the message will be an `Err`.
    /// NOTE that this does not apply to errors regarding whether the server
    /// could start or not, those are returned directly by this function.
    ///
    /// This function is non-blocking and thus spawns a thread where it
    /// listens to messages.
    pub fn start(self, on_message: Box<OnMessage<Vec<u8>>>) -> Result<IpcServerId> {
        let socket =
            Self::start_zmq_server(self.port).chain_err(|| ErrorKind::CouldNotStartServer)?;
        let _ = Self::start_receive_loop(socket, on_message);
        Ok(format!("localhost:{}", self.port))
    }

    fn start_zmq_server(port: u16) -> zmq::Result<zmq::Socket> {
        let ctx = zmq::Context::new();

        let socket = ctx.socket(zmq::PULL)?;
        let connection_string = format!("tcp://127.0.0.1:{}", port);
        socket.bind(&connection_string)?;

        Ok(socket)
    }

    fn start_receive_loop(socket: zmq::Socket,
                          mut on_message: Box<OnMessage<Vec<u8>>>)
                          -> thread::JoinHandle<()> {

        thread::spawn(move || loop {
            let read_res = socket.recv_bytes(0).chain_err(|| ErrorKind::ReadFailure);
            on_message(read_res)
        })
    }
}
