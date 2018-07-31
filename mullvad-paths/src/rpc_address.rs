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

fn get_default_rpc_address_dir() -> Result<PathBuf> {
    #[cfg(unix)]
    {
        Ok(PathBuf::from("/tmp"))
    }
    #[cfg(windows)]
    {
        ::get_allusersprofile_dir().map(|dir| dir.join(::PRODUCT_NAME))
    }
}
