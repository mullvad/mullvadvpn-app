#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate assert_matches;

#[cfg(all(test, not(windows)))]
mod zmq_integration_tests {
    extern crate talpid_ipc;
    extern crate zmq;


    use self::talpid_ipc::Result;
    use std::sync::mpsc::{self, Receiver};
    use std::time::Duration;

    const A_VALID_MESSAGE: [u8; 1] = [1];

    #[test]
    fn can_connect_to_server_with_the_returned_id() {
        let connection_string = talpid_ipc::start_new_server(Box::new(|_| {}))
            .expect("Unable to start server");

        let connection_res = connect_to_server(&connection_string);
        assert!(connection_res.is_ok(),
                "Unable to connect to the server with the given connection string");
    }

    #[test]
    fn publishes_incoming_messages_to_channel() {
        let new_messages_rx = connect_and_send(&A_VALID_MESSAGE);

        let message = new_messages_rx.recv_timeout(Duration::from_millis(1000))
            .expect("Did not receive a message");
        assert_matches!(message, Ok(TestMessage::Hello));
    }

    fn connect_and_send(message: &[u8]) -> Receiver<Result<TestMessage>> {
        let (tx, rx) = mpsc::channel();

        let connection_string = talpid_ipc::start_new_server(Box::new(move |message| {
            let _ = tx.send(message.and_then(parse_to_test_enum));
        })).expect("Could not start the server");

        let socket = connect_to_server(&connection_string)
            .expect("Could not connect to the server");
        socket.send(message, 0).expect("Could not send message");

        rx
    }

    fn connect_to_server(connection_string: &str) -> zmq::Result<zmq::Socket> {
        let ctx = zmq::Context::new();

        let socket = ctx.socket(zmq::PUSH)?;
        socket.connect(connection_string)?;
        Ok(socket)
    }

    fn parse_to_test_enum(message_as_bytes: Vec<u8>) -> Result<TestMessage> {
        if message_as_bytes == A_VALID_MESSAGE {
            Ok(TestMessage::Hello)
        } else {
            Err(format!("Invalid message: {:?}", message_as_bytes).into())
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum TestMessage {
        Hello,
    }
}
