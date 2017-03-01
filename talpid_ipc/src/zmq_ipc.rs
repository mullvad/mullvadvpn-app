extern crate zmq;
extern crate rand;

use self::rand::distributions::{IndependentSample, Range};
use super::{OnMessage, ErrorKind, Result, ResultExt};
use std::thread;

type IpcServerId = String;

/// The server end of our Inter-Process Communcation implementation.
pub struct Server;
impl Server {
    pub fn new() -> Self {
        Server {}
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

        let port_range = Range::new(1024, 65535); // Docs doesn't say if this is inclusive or exclusive...
        let mut rng = rand::thread_rng();
        for _ in 0..10 {

            let port = port_range.ind_sample(&mut rng);
            if let Ok(socket) = Self::attempt_to_start_on_port(port) {
                let _ = Self::start_receive_loop(socket, on_message);
                return Ok(format!("tcp://localhost:{}", port));
            }
        }

        return Err(ErrorKind::CouldNotStartServer.into());
    }

    fn attempt_to_start_on_port(port: u16) -> Result<zmq::Socket> {
        let socket = Self::start_zmq_server(port).chain_err(|| ErrorKind::CouldNotStartServer)?;
        Ok(socket)
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
