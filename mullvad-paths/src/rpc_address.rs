use std::env;
use std::path::PathBuf;

pub fn get_rpc_socket_path() -> PathBuf {
    match env::var_os("MULLVAD_RPC_SOCKET_PATH") {
        Some(path) => PathBuf::from(path),
        #[cfg(unix)]
        None => PathBuf::from("/var/run/mullvad-vpn"),
        #[cfg(windows)]
        None => PathBuf::from("//./pipe/Mullvad VPN"),
    }
}
