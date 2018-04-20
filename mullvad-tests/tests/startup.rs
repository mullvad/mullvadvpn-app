#![cfg(target_os = "linux")]

extern crate mullvad_paths;
extern crate mullvad_tests;

use std::fs::{self, Metadata};
use std::io;
use std::time::Duration;

use mullvad_tests::DaemonRunner;

use platform_specific::*;

#[test]
fn rpc_info_file_permissions() {
    let rpc_file = mullvad_paths::get_rpc_address_path().unwrap();

    if let Err(error) = fs::remove_file(&rpc_file) {
        if error.kind() != io::ErrorKind::NotFound {
            panic!("failed to remove existing RPC address file");
        }
    }

    assert!(!rpc_file.exists());

    let mut daemon = DaemonRunner::spawn_with_real_rpc_address_file();

    daemon.assert_output("Wrote RPC connection info to", Duration::from_secs(10));

    assert!(rpc_file.exists());

    ensure_only_admin_can_write(
        fs::metadata(&rpc_file).expect("Failed to read RPC address file metadata"),
    );
}

#[cfg(unix)]
mod platform_specific {
    extern crate libc;

    use super::*;
    use std::os::unix::fs::MetadataExt;

    pub fn ensure_only_admin_can_write(metadata: Metadata) {
        let process_uid = unsafe { libc::getuid() };
        assert_eq!(metadata.uid(), process_uid);
        assert_eq!(metadata.mode() & 0o022, 0);
    }
}

#[cfg(not(unix))]
mod platform_specific {
    use super::*;

    pub fn ensure_only_admin_can_write(_metadata: Metadata) {
        // TODO: Test when correctly implemented on Windows
    }
}
