#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate error_chain;

extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate talpid_ipc;

extern crate jsonrpc_core;
extern crate jsonrpc_pubsub;
#[macro_use]
extern crate jsonrpc_macros;
extern crate jsonrpc_ws_server;

pub mod ipc_api;
pub mod mock_ipc;

error_chain!{}

quick_main!(run);

fn run() -> Result<()> {
    init_logger()?;

    let server = start_ipc()?;
    info!("Mullvad daemon listening on {}", server.address());
    main_loop(server)
}

fn init_logger() -> Result<()> {
    env_logger::init().chain_err(|| "Failed to bootstrap logging system")
}

fn start_ipc() -> Result<mock_ipc::IpcServer> {
    mock_ipc::IpcServer::start().chain_err(|| "Failed to start IPC server")
}

fn main_loop(server: mock_ipc::IpcServer) -> Result<()> {
    server.wait().chain_err(|| "Error while waiting for server to process")
}
