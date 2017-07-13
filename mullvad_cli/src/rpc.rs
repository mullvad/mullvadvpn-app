

use {Result, ResultExt};
use serde;
use std::fs::File;
use std::io::Read;
use talpid_ipc::WsIpcClient;

pub fn call<T, O>(method: &str, args: &T) -> Result<O>
    where T: serde::Serialize,
          O: for<'de> serde::Deserialize<'de>
{
    call_internal(method, args).chain_err(|| "Unable to call backend over RPC")
}

pub fn call_internal<T, O>(method: &str, args: &T) -> Result<O>
    where T: serde::Serialize,
          O: for<'de> serde::Deserialize<'de>
{
    let address = read_rpc_address()?;
    info!("Using RPC address {}", address);
    let mut rpc_client = WsIpcClient::new(address)
        .chain_err(|| "Unable to create RPC client")?;
    rpc_client.call(method, args).chain_err(|| format!("Unable to call RPC method {}", method))
}

fn read_rpc_address() -> Result<String> {
    let path = ::std::env::temp_dir().join(".mullvad_rpc_address");

    debug!("Trying to read RPC address at {}", path.to_string_lossy());
    let mut address = String::new();
    if let Ok(_) = File::open(path).and_then(|mut file| file.read_to_string(&mut address)) {
        return Ok(address);
    }
    bail!("Unable to read RPC address");
}
