#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate assert_matches;

#[cfg(all(test, not(windows)))]
mod zmq_integration_tests {
    extern crate talpid_ipc;
    extern crate zmq;

    use self::talpid_ipc::{Result, IpcClient};
    use std::sync::mpsc::{self, Receiver};
    use std::time::Duration;

    #[test]
    fn can_connect_and_send_and_receive_messages() {
        let (connection_string, new_messages_rx) = start_server();

        let mut ipc_client = IpcClient::new(connection_string);
        ipc_client.send(&[1, 3, 3, 7]).expect("Could not send message");

        let message = new_messages_rx.recv_timeout(Duration::from_millis(1000))
            .expect("Did not receive a message");

        assert_eq!(message.unwrap(),
                   &[1, 3, 3, 7],
                   "Read data does not match sent data");
    }

    fn start_server() -> (String, Receiver<Result<Vec<u8>>>) {
        let (tx, rx) = mpsc::channel();

        let connection_string =
            talpid_ipc::start_new_server(Box::new(move |message| { let _ = tx.send(message); }))
                .expect("Could not start the server");

        (connection_string, rx)
    }
}
