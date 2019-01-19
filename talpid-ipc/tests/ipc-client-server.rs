// TODO fix these tests on Windows
#![cfg(not(windows))]

use assert_matches::assert_matches;
use futures::{sync::oneshot, Future};

use jsonrpc_client_core::{Error as ClientError, Transport};
use jsonrpc_core::{Error, IoHandler};
use jsonrpc_macros::build_rpc_trait;
use std::{
    sync::{mpsc, Mutex},
    time::Duration,
};

build_rpc_trait! {
    pub trait TestApi {
        #[rpc(name = "foo")]
        fn foo(&self, i64) -> Result<(), Error>;
    }
}

struct ApiImpl {
    tx: Mutex<mpsc::Sender<i64>>,
}

impl TestApi for ApiImpl {
    fn foo(&self, i: i64) -> Result<(), Error> {
        self.tx.lock().unwrap().send(i).unwrap();
        Ok(())
    }
}

#[test]
fn can_call_rpcs_on_server() {
    env_logger::init();

    let (server, rx) = create_server();
    let server_path = server.path().to_owned();
    let client = create_client(server_path);

    let _result: () = client.call_method("foo", &[97]).wait().unwrap();
    assert_eq!(Ok(97), rx.recv_timeout(Duration::from_millis(500)));

    let result: Result<(), ClientError> = client.call_method("invalid_method", &[0]).wait();
    assert_matches!(result, Err(_));
    server.close_handle().close();
}

#[test]
#[should_panic]
fn ipc_client_invalid_url() {
    let _client = create_client("INVALID ID".to_owned());
}

fn create_server() -> (talpid_ipc::IpcServer, mpsc::Receiver<i64>) {
    let (tx, rx) = mpsc::channel();
    let rpc = ApiImpl { tx: Mutex::new(tx) };
    let mut io = IoHandler::new();
    io.extend_with(rpc.to_delegate());

    let uuid = uuid::Uuid::new_v4().to_string();
    let ipc_path = if cfg!(windows) {
        format!(r"\\.\pipe\ipc-test-{}", uuid)
    } else {
        format!("/tmp/ipc-test-{}", uuid)
    };
    let server = talpid_ipc::IpcServer::start(io.into(), &ipc_path).unwrap();
    (server, rx)
}

fn create_client(ipc_path: String) -> jsonrpc_client_core::ClientHandle {
    use std::thread;
    let (tx, rx) = oneshot::channel();

    thread::spawn(move || {
        let (client, client_handle) =
            jsonrpc_client_ipc::IpcTransport::new(&ipc_path, &tokio::reactor::Handle::current())
                .expect("failed to construct a transport")
                .into_client();
        tx.send(client_handle).unwrap();
        client.wait().unwrap();
    });

    rx.wait().expect("Failed to construct a valid client")
}
