#[macro_use]
extern crate duct;
extern crate libc;
extern crate os_pipe;

mod common;

use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::Duration;

use common::DaemonInstance;

#[test]
fn rpc_info_file_permissions() {
    let rpc_file = rpc_file_path();

    if let Err(error) = fs::remove_file(&rpc_file) {
        if error.kind() != io::ErrorKind::NotFound {
            panic!("failed to remove existing RPC address file");
        }
    }

    assert!(!rpc_file.exists());

    let mut daemon = DaemonInstance::new();

    daemon.assert_log_contains(
        "Mullvad management interface listening on",
        Duration::from_secs(10),
    );
    assert!(rpc_file.exists());

    let uid = unsafe { libc::getuid() };
    let metadata = fs::metadata(&rpc_file).expect("failed to inspect RPC address file");
    assert_eq!(metadata.uid(), uid);
    assert_eq!(metadata.mode() & 0o022, 0);
}

#[cfg(target(unix))]
fn rpc_file_path() -> Path {
    Path::new("/tmp/.mullvad_rpc_address")
}

#[cfg(not(target(unix)))]
fn rpc_file_path() -> Path {
    ::std::env::temp_dir().join("/tmp/.mullvad_rpc_address")
}
