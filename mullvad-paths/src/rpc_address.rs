use Result;

use std::env;
use std::path::PathBuf;

const RPC_ADDRESS_FILENAME: &str = ".mullvad_rpc_address";

pub fn get_rpc_address_path() -> Result<PathBuf> {
    match env::var_os("MULLVAD_RPC_ADDRESS_PATH") {
        Some(path) => Ok(PathBuf::from(path)),
        None => get_default_rpc_address_dir().map(|dir| dir.join(RPC_ADDRESS_FILENAME)),
    }
}

#[cfg(unix)]
fn get_default_rpc_address_dir() -> Result<PathBuf> {
    Ok(PathBuf::from("/tmp"))
}

#[cfg(windows)]
fn get_default_rpc_address_dir() -> Result<PathBuf> {
    let program_data_dir =
        env::var_os("ALLUSERSPROFILE").ok_or_else(|| ::ErrorKind::NoProgramDataDir)?;
    Ok(Path::new(program_data_dir).join(::PRODUCT_NAME))
}
