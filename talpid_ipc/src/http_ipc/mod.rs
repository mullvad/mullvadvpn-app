extern crate jsonrpc_core;
extern crate jsonrpc_http_server;

use self::jsonrpc_http_server::{Server, ServerBuilder};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::result::Result as StdResult;

mod connection_info;

error_chain! {
    errors {
        FailedToWriteConnectionInfo {
            description("Unable to write IPC connection info")
        }
        UnableToStartServer {
            description("Failed to start the server")
        }
    }
}


pub struct ServerHandle {
    address: String,
    server: Server,
}

impl ServerHandle {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn stop(self) {
        self.server.close();
    }
}

pub fn start(build_router: fn() -> jsonrpc_core::IoHandler) -> Result<ServerHandle> {
    let server = start_server(build_router).chain_err(|| ErrorKind::UnableToStartServer)?;
    let write_result = connection_info::write(server.address()).chain_err(
        || {
            ErrorKind::FailedToWriteConnectionInfo
        },
    );
    if let Err(e) = write_result {
        error!("Could not write the connection info, killing the IPC server");
        server.stop();
        Err(e)
    } else {
        info!("Started Ipc server on: {}", server.address());
        Ok(server)
    }
}

fn start_server(build_router: fn() -> jsonrpc_core::IoHandler)
                -> StdResult<ServerHandle, jsonrpc_http_server::Error> {
    let mut last_error = None;
    for port in 5000..5010 {
        match start_server_on_port(port, build_router()) {
            Ok(server) => return Ok(server),
            Err(e) => last_error = Some(e),
        }
    }
    bail!(last_error.unwrap());
}

fn start_server_on_port(port: u16,
                        router: jsonrpc_core::IoHandler)
                        -> StdResult<ServerHandle, jsonrpc_http_server::Error> {
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let listen_addr = SocketAddr::new(ip, port);
    ServerBuilder::new(router)
        .allow_only_bind_host()
        .start_http(&listen_addr)
        .map(
            |server| {
                ServerHandle {
                    address: format!("http://{}", listen_addr),
                    server: server,
                }
            },
        )
}
