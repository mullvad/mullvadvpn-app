extern crate talpid_ipc;
extern crate zmq;

#[macro_use]
extern crate assert_matches;

use std::result;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;
use talpid_ipc::{ErrorKind, Result};

const A_VALID_MESSAGE: u8 = 1;
const AN_INVALID_MESSAGE: u8 = 2;

#[test]
fn returns_connection_string_when_started() {
    let port = 1341;
    let connection_string = talpid_ipc::Server::new(port)
        .start(Box::new(|_| {}))
        .expect("Unable to start server");

    assert!(connection_string.contains("localhost"),
            format!("'{}' did not contain 'localhost'", connection_string));
    assert!(connection_string.contains(&port.to_string()),
            format!("'{}' did not contain the port", connection_string));
}

#[test]
fn gives_error_when_unable_to_start() {
    let port = 1340;

    let ipc_server1 = talpid_ipc::Server::new(port);
    let ipc_server2 = talpid_ipc::Server::new(port);

    ipc_server1.start(Box::new(|_| {}))
        .expect("Unable to start the first server. Results inconclusive");
    let start_res = ipc_server2.start(Box::new(|_| {}));

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

    let ipc_server = talpid_ipc::Server::new(port);
    ipc_server.start(Box::new(move |message| {
        let _ = tx.send(message.and_then(parse_to_test_enum));
    })).expect("Could not start the server");

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
