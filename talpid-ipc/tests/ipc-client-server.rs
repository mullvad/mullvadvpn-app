#[macro_use]
extern crate assert_matches;
extern crate env_logger;
extern crate jsonrpc_core;
#[macro_use]
extern crate jsonrpc_macros;
extern crate talpid_ipc;

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
    let server_id = server.address().to_owned();
    let mut client = create_client(&server_id);

    let _result: () = client.call("foo", &[97]).unwrap();
    assert_eq!(Ok(97), rx.recv_timeout(Duration::from_millis(500)));

    let result: Result<(), _> = client.call("invalid_method", &[0]);
    assert_matches!(result, Err(_));
    server.close_handle().close();
}

// TODO fix this test on Windows
#[cfg(not(windows))]
#[test]
#[should_panic]
fn ipc_client_invalid_url() {
    create_client(&"INVALID ID".to_owned());
}

#[test]
fn ipc_client_bad_connection() {
    let mut client = create_client(&"ws://127.0.0.1:9876".to_owned());
    let result: Result<(), _> = client.call("invalid_method", &[0]);
    assert_matches!(result, Err(_));
}

fn create_server() -> (talpid_ipc::IpcServer, mpsc::Receiver<i64>) {
    let (tx, rx) = mpsc::channel();
    let rpc = ApiImpl { tx: Mutex::new(tx) };
    let mut io = IoHandler::new();
    io.extend_with(rpc.to_delegate());

    let server = talpid_ipc::IpcServer::start(io.into()).unwrap();
    (server, rx)
}

fn create_client(id: &talpid_ipc::IpcServerId) -> talpid_ipc::WsIpcClient {
    talpid_ipc::WsIpcClient::connect(id).unwrap()
}
