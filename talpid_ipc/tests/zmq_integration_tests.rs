#[cfg(all(test, not(windows)))]
mod zmq_integration_tests {
    extern crate serde;
    extern crate talpid_ipc;

    use self::talpid_ipc::{IpcClient, IpcServerId, Result};

    use std::sync::mpsc::{self, Receiver};
    use std::time::Duration;

    #[test]
    fn can_connect_and_send_and_receive_messages() {
        let (connection_string, new_messages_rx) = start_server::<String>();

        let mut ipc_client = IpcClient::new(connection_string);
        let msg = "Hello".to_owned();
        ipc_client.send(&msg).expect("Could not send message");

        let message = new_messages_rx
            .recv_timeout(Duration::from_millis(1000))
            .expect("Did not receive a message");

        assert_eq!(message.unwrap(), "Hello", "Got wrong message");
    }

    fn start_server<T>() -> (IpcServerId, Receiver<Result<T>>)
        where for<'de> T: serde::Deserialize<'de> + Send + 'static
    {
        let (tx, rx) = mpsc::channel();

        let callback = move |message: Result<T>| { let _ = tx.send(message); };
        let connection_string =
            talpid_ipc::start_new_server(callback).expect("Could not start the server");

        (connection_string, rx)
    }
}
