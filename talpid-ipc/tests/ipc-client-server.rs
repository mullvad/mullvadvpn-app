#[macro_use]
extern crate assert_matches;
extern crate env_logger;
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_ipc;
extern crate jsonrpc_core;
#[macro_use]
extern crate jsonrpc_macros;
extern crate talpid_ipc;
extern crate tokio_core;
extern crate uuid;

extern crate futures;

use futures::sync::oneshot;
use futures::Future;
use tokio_core::reactor::Core;

use jsonrpc_client_core::{Error as ClientError, Transport};
use jsonrpc_core::{Error, IoHandler};
use std::sync::{mpsc, Mutex};
use std::time::Duration;

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

// TODO fix this test on Windows
#[cfg(not(windows))]
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

// TODO fix this test on Windows
#[cfg(not(windows))]
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
    let server = talpid_ipc::IpcServer::start(io.into(), ipc_path).unwrap();
    (server, rx)
}

fn create_client(ipc_path: String) -> jsonrpc_client_core::ClientHandle {
    use std::thread;
    let (tx, rx) = oneshot::channel();

    thread::spawn(move || {
        let mut core = Core::new().expect("failed to spawn reactor");
        let (client, client_handle) =
            jsonrpc_client_ipc::IpcTransport::new(&ipc_path, &core.handle())
                .expect("failed to construct a transport")
                .into_client();
        tx.send(client_handle).unwrap();
        core.run(client).unwrap();
    });

    let handle = rx.wait().expect("Failed to construct a valid client");
    handle
}
