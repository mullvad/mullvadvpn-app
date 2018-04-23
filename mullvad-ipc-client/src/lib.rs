extern crate serde;
extern crate talpid_ipc;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use talpid_ipc::WsIpcClient;

pub struct DaemonRpcClient {
    address: String,
}

impl DaemonRpcClient {
    pub fn new() -> Result<Self, String> {
        let rpc_file = File::open(rpc_file_path())
            .map_err(|error| format!("failed to open RPC address file: {}", error))?;
        let reader = BufReader::new(rpc_file);
        let mut lines = reader.lines();
        let address = lines
            .next()
            .ok_or("RPC address file is empty".to_string())?
            .map_err(|error| format!("failed to read address from RPC address file: {}", error))?;

        Ok(DaemonRpcClient { address })
    }

    pub fn shutdown(&self) -> Result<(), String> {
        self.call("shutdown", &[] as &[u8; 0])
    }

    pub fn call<A, O>(&self, method: &str, args: &A) -> Result<O, String>
    where
        A: Serialize,
        O: for<'de> Deserialize<'de>,
    {
        let mut rpc_client = WsIpcClient::new(self.address.clone())
            .map_err(|error| format!("unable to create RPC client: {}", error))?;

        rpc_client
            .call(method, args)
            .map_err(|error| format!("RPC request failed: {}", error))
    }
}

#[cfg(unix)]
pub fn rpc_file_path() -> PathBuf {
    use std::path::Path;

    Path::new("/tmp/.mullvad_rpc_address").to_path_buf()
}

#[cfg(windows)]
pub fn rpc_file_path() -> PathBuf {
    let windows_directory = ::std::env::var_os("WINDIR").unwrap();
    PathBuf::from(windows_directory)
        .join("Temp")
        .join(".mullvad_rpc_address")
}
