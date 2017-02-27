extern crate zmq;

use ipc::{IpcServer, OnMessage, ErrorKind, Result, ResultExt};
use std::thread;

/// The signature of functions that can be used to parse the incoming data
/// This is very very similar to `TryFrom` on purpose because I wanted `TryFrom`,
/// but I couldn't get it to work.
type MessageParser<MessageType> = Fn(Vec<u8>) -> Result<MessageType> + Send + 'static;

/// Implements the `IpcServer` trait using a ZeroMQ PULL socket
/// The IPC server can parse any kind of message, so when you create
/// the `ZmqIpcServer` you supply it with a `parser` that converts
/// the read bytes into the type you want.
pub struct ZmqIpcServer<T>
    where T: 'static
{
    parser: Box<MessageParser<T>>,
}

impl<T> IpcServer for ZmqIpcServer<T>
    where T: 'static
{
    type MessageType = T;

    fn start(self, port: u16, on_message: Box<OnMessage<T>>) -> Result<()> {
        let socket = Self::start_zmq_server(port).chain_err(|| ErrorKind::CouldNotStartServer)?;
        let _ = Self::start_receive_loop(socket, on_message, self.parser);
        Ok(())
    }
}

impl<T> ZmqIpcServer<T>
    where T: 'static
{
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

#[cfg(test)]
mod tests {
    use super::*;
    use ipc::{IpcServer, Result, ErrorKind};
    use std::result;
    use std::sync::mpsc::{self, Receiver};
    use std::time::Duration;
    extern crate zmq;

    const A_VALID_MESSAGE: u8 = 1;
    const AN_INVALID_MESSAGE: u8 = 2;

    #[test]
    fn gives_error_when_unable_to_start() {
        let port = 1340;

        let ipc_server1 = ZmqIpcServer { parser: Box::new(parse_to_test_enum) };
        let ipc_server2 = ZmqIpcServer { parser: Box::new(parse_to_test_enum) };

        ipc_server1.start(port, Box::new(|_| {}))
            .expect("Unable to start the first server. Results inconclusive");
        let start_res = ipc_server2.start(port, Box::new(|_| {}));

        assert!(start_res.is_err());
        let err = start_res.unwrap_err();
        assert_matches!(err.kind(), &ErrorKind::CouldNotStartServer);
        assert!(err.iter().count() > 1)
    }

    #[test]
    fn publishes_incoming_messages_to_channel() {
        let new_messages_rx = connect_and_send(1337, A_VALID_MESSAGE);

        let message = new_messages_rx.recv_timeout(Duration::from_millis(1000))
            .expect("Did not receive a message");
        assert_matches!(message, Ok(TestMessage::HELLO));
    }

    #[test]
    fn does_not_publish_unknown_messages() {
        let rx = connect_and_send(1338, AN_INVALID_MESSAGE);

        let message = rx.recv_timeout(Duration::from_millis(1000))
            .expect("Did not receive message");

        assert_matches!(message.unwrap_err().kind(), &ErrorKind::InvalidMessage(_));
    }

    fn connect_and_send(port: u16, message: u8) -> Receiver<Result<TestMessage>> {
        let (tx, rx) = mpsc::channel();

        let ipc_server = ZmqIpcServer { parser: Box::new(parse_to_test_enum) };
        ipc_server.start(port, Box::new(move |message| { let _ = tx.send(message); }))
            .expect("Could not start the server");

        let socket = connect_to_server(port).expect("Could not connect to the server");
        socket.send(&[message], 0).expect("Could not send message");

        rx
    }

    fn connect_to_server(port: u16) -> result::Result<zmq::Socket, zmq::Error> {
        let ctx = zmq::Context::new();

        let socket = ctx.socket(zmq::PUSH)?;
        let connection_string: String = format!("tcp://127.0.0.1:{}", port);
        try!(socket.connect(connection_string.as_str()));
        Ok(socket)
    }

    fn parse_to_test_enum(message_as_bytes: Vec<u8>) -> Result<TestMessage> {
        if message_as_bytes[0] == A_VALID_MESSAGE {
            Ok(TestMessage::HELLO)
        } else {
            Err(ErrorKind::InvalidMessage(message_as_bytes).into())
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum TestMessage {
        HELLO,
    }
}
