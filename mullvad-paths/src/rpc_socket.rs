use std::{env, path::PathBuf};

pub fn get_rpc_socket_path() -> PathBuf {
    match env::var_os("MULLVAD_RPC_SOCKET_PATH") {
        Some(path) => PathBuf::from(path),
        None => get_default_rpc_socket_path(),
    }
}

pub fn get_default_rpc_socket_path() -> PathBuf {
    #[cfg(all(unix, not(target_os = "android")))]
    {
        PathBuf::from("/var/run/mullvad-vpn")
    }
    #[cfg(windows)]
    {
        PathBuf::from("//./pipe/Mullvad VPN")
    }
    #[cfg(target_os = "android")]
    {
        PathBuf::from(format!("{}/rpc-socket", crate::APP_PATH))
    }
}
