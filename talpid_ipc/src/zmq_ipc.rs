extern crate zmq;
extern crate serde_json;

use super::{ErrorKind, Result, ResultExt, IpcServerId};

use serde;

use std::thread;

/// Starts the server end of an IPC channel. The returned `IpcServerId` is the unique identifier
/// allowing an `IpcClient` to connect to this server instance. Returns an error if unable to set
/// up the server.
///
/// Incoming messages are sent as `Ok` results to the `on_message` callback. IO errors will be sent
/// as `Err` results to the `on_message` callback.
///
/// This function is non-blocking and thus spawns a thread where it listens to messages.
pub fn start_new_server<T, F>(on_message: F) -> Result<IpcServerId>
    where T: serde::Deserialize + 'static,
          F: FnMut(Result<T>) + Send + 'static
{
    for port in 5000..5010 {
        let connection_string = format!("tcp://127.0.0.1:{}", port);
        if let Ok(socket) = start_zmq_server(&connection_string) {
            let _ = start_receive_loop(socket, on_message);
            debug!("Listening on {}", connection_string);
            return Ok(connection_string);
        }
    }
    bail!(ErrorKind::CouldNotStartServer)
}

fn start_zmq_server(connection_string: &str) -> zmq::Result<zmq::Socket> {
    let ctx = zmq::Context::new();

    let socket = ctx.socket(zmq::PULL)?;
    socket.bind(connection_string)?;

    Ok(socket)
}

fn start_receive_loop<T, F>(socket: zmq::Socket, mut on_message: F) -> thread::JoinHandle<()>
    where T: serde::Deserialize + 'static,
          F: FnMut(Result<T>) + Send + 'static
{
    thread::spawn(move || loop {
        let read_res = socket.recv_bytes(0)
            .chain_err(|| ErrorKind::ReadFailure)
            .and_then(|a| parse_message(&a));
        on_message(read_res);
    })
}

fn parse_message<T>(message: &[u8]) -> Result<T>
    where T: serde::Deserialize + 'static
{
    serde_json::from_slice(message).chain_err(|| ErrorKind::ParseFailure)
}


pub struct IpcClient<T>
    where T: serde::Serialize
{
    server_id: IpcServerId,
    socket: Option<zmq::Socket>,
    _phantom: ::std::marker::PhantomData<T>,
}

impl<T> IpcClient<T>
    where T: serde::Serialize
{
    pub fn new(server_id: IpcServerId) -> Self {
        IpcClient {
            server_id: server_id,
            socket: None,
            _phantom: ::std::marker::PhantomData,
        }
    }

    pub fn send(&mut self, message: &T) -> Result<()> {
        let bytes = Self::serialize(message)?;
        self.send_bytes(bytes.as_slice())
    }

    fn serialize(t: &T) -> Result<Vec<u8>> {
        serde_json::to_vec(t).chain_err(|| ErrorKind::ParseFailure)
    }

    fn send_bytes(&mut self, message: &[u8]) -> Result<()> {
        if self.socket.is_none() {
            self.connect().chain_err(|| ErrorKind::SendError)?;
        }

        let socket = self.socket.as_ref().unwrap();
        socket.send(message, 0).chain_err(|| ErrorKind::SendError)
    }

    fn connect(&mut self) -> Result<()> {
        debug!("Trying to establish connection to {}", self.server_id);
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::PUSH)
            .chain_err(|| "Could not create ZeroMQ PUSH socket".to_owned())?;
        socket.connect(&self.server_id)
            .chain_err(|| format!("Could not connect to {:?}", self.server_id))?;

        self.socket = Some(socket);
        Ok(())
    }
}
