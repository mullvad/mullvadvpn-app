use {Result, ResultExt};
use serde;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use talpid_ipc::WsIpcClient;

pub fn call<T, O>(method: &str, args: &T) -> Result<O>
where
    T: serde::Serialize,
    O: for<'de> serde::Deserialize<'de>,
{
    call_internal(method, args).chain_err(|| "Unable to call backend over RPC")
}

pub fn call_internal<T, O>(method: &str, args: &T) -> Result<O>
where
    T: serde::Serialize,
    O: for<'de> serde::Deserialize<'de>,
{
    let address = read_rpc_address().chain_err(|| "Unable to read RPC address")?;
    info!("Using RPC address {}", address);
    let mut rpc_client = WsIpcClient::new(address).chain_err(|| "Unable to create RPC client")?;
    rpc_client
        .call(method, args)
        .chain_err(|| format!("Unable to call RPC method {}", method))
}


#[cfg(unix)]
lazy_static! {
    /// The path to the file where we read the RPC address
    static ref RPC_ADDRESS_FILE_PATH: PathBuf = Path::new("/tmp").join(".mullvad_rpc_address");
}

#[cfg(not(unix))]
lazy_static! {
    /// The path to the file where we read the RPC address
    static ref RPC_ADDRESS_FILE_PATH: PathBuf = ::std::env::temp_dir().join(".mullvad_rpc_address");
}

fn read_rpc_address() -> io::Result<String> {
    debug!(
        "Trying to read RPC address at {}",
        RPC_ADDRESS_FILE_PATH.to_string_lossy()
    );
    let mut address = String::new();
    let mut file = File::open(&*RPC_ADDRESS_FILE_PATH)?;
    file.read_to_string(&mut address)?;
    Ok(address)
}
