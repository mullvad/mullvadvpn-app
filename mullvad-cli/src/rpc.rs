use {Result, ResultExt};
use serde;

use std::fs::{File, Metadata};
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
    let mut file = File::open(&*RPC_ADDRESS_FILE_PATH)?;
    if is_rpc_file_trusted(file.metadata()?) {
        let mut address = String::new();
        file.read_to_string(&mut address)?;
        Ok(address)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "RPC file is not trusted",
        ))
    }
}

#[cfg(unix)]
fn is_rpc_file_trusted(metadata: Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    let is_owned_by_root = metadata.uid() == 0;
    let is_read_only_by_non_owner = (metadata.mode() & 0o022) == 0;

    is_owned_by_root && is_read_only_by_non_owner
}

#[cfg(windows)]
fn is_rpc_file_trusted(metadata: Metadata) -> bool {
    // TODO: Check permissions correctly
    true
}
