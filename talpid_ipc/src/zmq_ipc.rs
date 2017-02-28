extern crate zmq;

use super::{OnMessage, ErrorKind, Result, ResultExt};
use std::thread;

/// The signature of functions that can be used to parse the incoming data
/// This is very very similar to `TryFrom` on purpose because I wanted `TryFrom`,
/// but I couldn't get it to work.
type MessageParser<MessageType> = Fn(Vec<u8>) -> Result<MessageType> + Send + 'static;

type IpcServerId = String;

/// The server end of our Inter-Process Communcation implementation.
/// It can parse any kind of message, so when you create
/// it you supply it with a `parser` that converts
/// the read bytes into the type you want.
pub struct Server<T>
    where T: 'static
{
    parser: Box<MessageParser<T>>,
    port: u16,
}

impl<T> Server<T>
    where T: 'static
{
    pub fn new(port: u16, parser: Box<MessageParser<T>>) -> Self {
        Server {
            port: port,
            parser: parser,
        }
    }

    /// Starts listening to incoming IPC connections on the specified port.
    /// Messages are sent to the `on_message` callback. If anything went wrong
    /// when reading or parsing the message, the message will be an `Err`.
    /// NOTE that this does not apply to errors regarding whether the server
    /// could start or not, those are returned directly by this function.
    ///
    /// This function is non-blocking and thus spawns a thread where it
    /// listens to messages.
    pub fn start(self, on_message: Box<OnMessage<T>>) -> Result<IpcServerId> {
        let socket =
            Self::start_zmq_server(self.port).chain_err(|| ErrorKind::CouldNotStartServer)?;
        let _ = Self::start_receive_loop(socket, on_message, self.parser);
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
                          mut on_message: Box<OnMessage<T>>,
                          parser: Box<MessageParser<T>>)
                          -> thread::JoinHandle<()> {

        thread::spawn(move || loop {
            let read_res = Self::read(&socket, &parser);
            on_message(read_res)
        })
    }

    fn read(socket: &zmq::Socket, parser: &Box<MessageParser<T>>) -> Result<T> {
        let bytes = socket.recv_bytes(0).chain_err(|| ErrorKind::ReadFailure)?;
        parser(bytes)
    }
}
