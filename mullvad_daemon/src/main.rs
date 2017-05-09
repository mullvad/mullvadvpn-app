#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate talpid_ipc;

mod frontend_ipc_router;

error_chain!{}

quick_main!(run);

fn run() -> Result<()> {
    init_logger()?;
    let _server = start_ipc()?;
    main_loop()
}

fn init_logger() -> Result<()> {
    env_logger::init().chain_err(|| "Failed to bootstrap logging system")
}

fn start_ipc() -> Result<talpid_ipc::IpcServer> {
    talpid_ipc::IpcServer::start(frontend_ipc_router::build_router().into(), 0)
        .chain_err(|| "Failed to start IPC server")
}

fn main_loop() -> Result<()> {
    let (_tx, rx) = ::std::sync::mpsc::channel::<u8>();
    let _ = rx.recv();
    Ok(())
}
