#[cfg(all(test, not(windows)))]
mod http_integration_tests {
    extern crate serde;
    extern crate talpid_ipc;

    use self::talpid_ipc::{Result, IpcServerId};
    use self::talpid_ipc::http_ipc;

    use std::sync::mpsc::{self, Receiver};
    use std::time::Duration;

    #[test]
    fn can_connect_and_send_and_receive_messages() {
        let (connection_string, server_messages) = start_server::<String>();

        let mut ipc_client = http_ipc::IpcClient::new(connection_string);
        let msg = "Hello".to_owned();
        let response: String = ipc_client.send(&msg).expect("Could not send message");

        let message = server_messages.recv_timeout(Duration::from_millis(1000))
            .expect("Did not receive a message");

        assert_eq!(message.unwrap(), "Hello", "Got wrong message");
        assert_eq!(response, "RESPONSE");
    }

    fn start_server<T>() -> (IpcServerId, Receiver<Result<T>>)
        where T: serde::Deserialize + Send + 'static
    {
        let (tx, rx) = mpsc::channel();

        let connection_string = http_ipc::start_new_server(move |message: Result<T>| {
                let _ = tx.send(message);
                "RESPONSE"
            })
            .expect("Could not start the server");

        (connection_string, rx)
    }
}
