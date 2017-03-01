extern crate talpid_ipc;
extern crate zmq;
extern crate regex;

#[macro_use]
extern crate assert_matches;

use regex::Regex;
use std::result;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;
use talpid_ipc::{ErrorKind, Result};

const A_VALID_MESSAGE: u8 = 1;
const AN_INVALID_MESSAGE: u8 = 2;

#[test]
fn returns_connection_string_when_started() {
    let connection_string = talpid_ipc::Server::new()
        .start(Box::new(|_| {}))
        .expect("Unable to start server");

    let re = Regex::new(r"^tcp://localhost:\d+$").unwrap();
    assert!(re.is_match(&connection_string),
            format!("'{}' does not match 'tcp://localhost:port'",
                    connection_string));
}

#[test]
fn publishes_incoming_messages_to_channel() {
    let new_messages_rx = connect_and_send(A_VALID_MESSAGE);

    let message = new_messages_rx.recv_timeout(Duration::from_millis(1000))
        .expect("Did not receive a message");
    assert_matches!(message, Ok(TestMessage::HELLO));
}

#[test]
fn does_not_publish_unknown_messages() {
    let rx = connect_and_send(AN_INVALID_MESSAGE);

    let message = rx.recv_timeout(Duration::from_millis(1000))
        .expect("Did not receive message");

    assert_matches!(message.unwrap_err().kind(), &ErrorKind::InvalidMessage(_));
}

fn connect_and_send(message: u8) -> Receiver<Result<TestMessage>> {
    let (tx, rx) = mpsc::channel();

    let ipc_server = talpid_ipc::Server::new();
    let connection_string = ipc_server.start(Box::new(move |message| {
            let _ = tx.send(message.and_then(parse_to_test_enum));
        })).expect("Could not start the server");

    let socket = connect_to_server(&connection_string).expect("Could not connect to the server");
    socket.send(&[message], 0).expect("Could not send message");

    rx
}

fn connect_to_server(connection_string: &str) -> result::Result<zmq::Socket, zmq::Error> {
    let ctx = zmq::Context::new();

    let socket = ctx.socket(zmq::PUSH)?;
    try!(socket.connect(connection_string));
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
