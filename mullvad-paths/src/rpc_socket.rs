use std::path::PathBuf;

#[cfg(not(target_os = "android"))]
pub fn get_rpc_socket_path() -> PathBuf {
    match std::env::var_os("MULLVAD_RPC_SOCKET_PATH") {
        Some(path) => PathBuf::from(path),
        None => get_default_rpc_socket_path(),
    }
}

/// Return the path to the RPC socket using `data_dir` as the base directory.
#[cfg(target_os = "android")]
pub fn get_rpc_socket_path(data_dir: PathBuf) -> PathBuf {
    data_dir.join("rpc-socket")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn get_default_rpc_socket_path() -> PathBuf {
    PathBuf::from("/var/run/mullvad-vpn")
}

#[cfg(windows)]
pub fn get_default_rpc_socket_path() -> PathBuf {
    PathBuf::from("//./pipe/Mullvad VPN")
}
