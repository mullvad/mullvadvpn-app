

use {Result, ResultExt};
use serde;
use serde_json;
use std::fs::File;
use std::io::Read;
use talpid_ipc::WsIpcClient;

pub fn call<T>(method: &str, args: &T) -> Result<serde_json::Value>
    where T: serde::Serialize
{
    let address = read_rpc_address()?;
    info!("Using RPC address {}", address);
    let mut rpc_client = WsIpcClient::new(address)
        .chain_err(|| "Unable to create RPC client")?;
    rpc_client.call(method, args).chain_err(|| "Unable to call RPC method")
}

fn read_rpc_address() -> Result<String> {
    for path in &["./.mullvad_rpc_address", "../.mullvad_rpc_address"] {
        debug!("Trying to read RPC address at {}", path);
        let mut address = String::new();
        if let Ok(_) = File::open(path).and_then(|mut file| file.read_to_string(&mut address)) {
            return Ok(address);
        }
    }
    bail!("Unable to read RPC address");
}
