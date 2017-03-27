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

error_chain! {
    errors {
        InitLoggingFailed {
            description("Failed to bootstrap logging system")
        }
    }
}

fn main() {

    if let Err(e) = init_logger() {
        log_error_chain_error(&e);
        ::std::process::exit(1);
    }

    let ipc_res = talpid_ipc::http_ipc::start(frontend_ipc_router::build_router);
    if let Err(e) = ipc_res {
        error!("Failed to start IPC server");
        log_error_chain_error(&e);
        ::std::process::exit(2);
    }

    let (tx, rx) = ::std::sync::mpsc::channel::<u8>();
    rx.recv();
}

fn init_logger() -> Result<()> {
    env_logger::init().chain_err(|| ErrorKind::InitLoggingFailed)
}

fn log_error_chain_error<E: ::error_chain::ChainedError>(e: &E) {
    println!("error: {}", e);
    for e in e.iter().skip(1) {
        println!("caused by: {}", e);
    }
    if let Some(backtrace) = e.backtrace() {
        println!("backtrace: {:?}", backtrace);
    }
}
