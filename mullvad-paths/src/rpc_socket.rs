use std::{env, path::PathBuf};

pub fn get_rpc_socket_path() -> PathBuf {
    match env::var_os("MULLVAD_RPC_SOCKET_PATH") {
        Some(path) => PathBuf::from(path),
        None => get_default_rpc_socket_path(),
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn get_default_rpc_socket_path() -> PathBuf {
    PathBuf::from("/var/run/mullvad-vpn")
}

#[cfg(windows)]
pub fn get_default_rpc_socket_path() -> PathBuf {
    PathBuf::from("//./pipe/Mullvad VPN")
}

#[cfg(target_os = "android")]
pub fn get_default_rpc_socket_path() -> PathBuf {
    PathBuf::from(format!("{}/rpc-socket", crate::APP_PATH))
}
