extern crate libc;

mod common;

use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::Duration;

use common::DaemonInstance;

#[test]
fn rpc_info_file_permissions() {
    let rpc_file = Path::new("/tmp/.mullvad_rpc_address");

    let _ = fs::remove_file(&rpc_file);
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
