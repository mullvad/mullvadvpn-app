#[macro_use]
extern crate error_chain;
extern crate serde;
extern crate talpid_ipc;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use talpid_ipc::WsIpcClient;

error_chain! {
    errors {
        EmptyRpcFile(file_path: String) {
            description("RPC connection file is empty")
            display("RPC connection file \"{}\" is empty", file_path)
        }

        ReadRpcFileError(file_path: String) {
            description("Failed to read RPC connection information")
            display("Failed to read RPC connection information from {}", file_path)
        }

        RpcCallError(method: String) {
            description("Failed to call RPC method")
            display("Failed to call RPC method \"{}\"", method)
        }

        StartRpcClient(address: String) {
            description("Failed to start RPC client")
            display("Failed to start RPC client to {}", address)
        }

        UnknownRpcFilePath {
            description("Failed to determine RPC connection information file path")
        }
    }
}

pub struct DaemonRpcClient {
    address: String,
}

impl DaemonRpcClient {
    pub fn new() -> Result<Self> {
        let address = Self::read_rpc_file()?;

        Ok(DaemonRpcClient { address })
    }

    fn read_rpc_file() -> Result<String> {
        let file_path = rpc_file_path()?;
        let file_path_string = || file_path.display().to_string();
        let rpc_file =
            File::open(&file_path).chain_err(|| ErrorKind::ReadRpcFileError(file_path_string()))?;
        let reader = BufReader::new(rpc_file);
        let mut lines = reader.lines();

        lines
            .next()
            .ok_or_else(|| ErrorKind::EmptyRpcFile(file_path_string()))?
            .chain_err(|| ErrorKind::ReadRpcFileError(file_path_string()))
    }

    pub fn shutdown(&self) -> Result<()> {
        self.call("shutdown", &[] as &[u8; 0])
    }

    pub fn call<A, O>(&self, method: &str, args: &A) -> Result<O>
    where
        A: Serialize,
        O: for<'de> Deserialize<'de>,
    {
        let mut rpc_client = WsIpcClient::new(self.address.clone())
            .chain_err(|| ErrorKind::StartRpcClient(self.address.clone()))?;

        rpc_client
            .call(method, args)
            .chain_err(|| ErrorKind::RpcCallError(method.to_owned()))
    }
}

#[cfg(unix)]
pub fn rpc_file_path() -> Result<PathBuf> {
    use std::path::Path;

    Ok(Path::new("/tmp/.mullvad_rpc_address").to_path_buf())
}

#[cfg(windows)]
pub fn rpc_file_path() -> Result<PathBuf> {
    let windows_directory =
        ::std::env::var_os("WINDIR").ok_or_else(|| ErrorKind::UnknownRpcFilePath)?;

    Ok(PathBuf::from(windows_directory)
        .join("Temp")
        .join(".mullvad_rpc_address"))
}
