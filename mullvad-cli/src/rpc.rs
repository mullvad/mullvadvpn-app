use {Result, ResultExt};
use serde;

use std::fs::{File, Metadata};
use std::io::{BufRead, BufReader};
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;

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
    let (address, _shared_secret) = read_rpc_address().chain_err(|| "Unable to read RPC address")?;
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

fn read_rpc_address() -> Result<(String, String)> {
    let file = open_rpc_file()?;
    let mut buf_file = BufReader::new(file);
    let mut address = String::new();
    buf_file.read_line(&mut address)?;
    let mut shared_secret = String::new();
    buf_file.read_line(&mut shared_secret)?;
    Ok((address, shared_secret))
}

fn open_rpc_file() -> Result<File> {
    debug!(
        "Trying to read RPC address at {}",
        RPC_ADDRESS_FILE_PATH.to_string_lossy()
    );
    let file = File::open(&*RPC_ADDRESS_FILE_PATH)?;
    check_if_rpc_file_can_be_trusted(file.metadata()?).chain_err(|| "RPC file is not trusted")?;

    Ok(file)
}

#[cfg(unix)]
fn check_if_rpc_file_can_be_trusted(metadata: Metadata) -> Result<()> {
    use std::os::unix::fs::MetadataExt;

    let is_owned_by_root = metadata.uid() == 0;
    let is_read_only_by_non_owner = (metadata.mode() & 0o022) == 0;

    ensure!(is_owned_by_root, "RPC file is not owned by root");
    ensure!(
        is_read_only_by_non_owner,
        "RPC file is writable by non-root users"
    );

    Ok(())
}

#[cfg(windows)]
fn check_if_rpc_file_can_be_trusted(_metadata: Metadata) -> Result<()> {
    // TODO: Check permissions correctly
    Ok(())
}
