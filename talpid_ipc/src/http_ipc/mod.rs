extern crate jsonrpc_core;
extern crate jsonrpc_http_server;

use self::jsonrpc_http_server::{ServerBuilder, Server};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

mod connection_info;

pub struct ServerHandle {
    pub address: String,
    server: Server,
}

impl ServerHandle {
    pub fn stop(self) {
        self.server.close();
    }
}

pub fn start(build_router: fn() -> jsonrpc_core::IoHandler) -> Result<ServerHandle> {
    let server = start_server(build_router)?;

    let write_res = connection_info::write(&server.address)
        .chain_err(|| ErrorKind::FailedToWriteConnectionInfo);
    if let Err(e) = write_res {
        debug!("Could not write the connection info, killing the IPC server");
        server.stop();

        return Err(e);
    }

    info!("Started Ipc server on: {:?}", server.address);
    Ok(server)
}

fn start_server(build_router: fn() -> jsonrpc_core::IoHandler) -> Result<ServerHandle> {

    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    for port in 5000..5010 {
        let server_config = ServerBuilder::new(build_router()).allow_only_bind_host();

        let socket_addr = SocketAddr::new(ip, port);
        let start_res = attempt_to_start(server_config, &socket_addr);

        match start_res {
            Ok(server) => {
                return Ok(ServerHandle {
                    address: format!("http://{}", socket_addr),
                    server: server,
                });
            }
            Err(Error(ErrorKind::UnableToStartServer, _)) => (),
            Err(e) => return Err(e),
        }
    }
    bail!(ErrorKind::UnableToStartServer)
}

fn attempt_to_start(server_config: ServerBuilder, address: &SocketAddr) -> Result<Server> {
    server_config.start_http(&address)
        .chain_err(|| ErrorKind::UnableToStartServer)
}

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
