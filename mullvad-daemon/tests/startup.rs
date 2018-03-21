#[macro_use]
extern crate duct;
extern crate os_pipe;
extern crate serde;
extern crate talpid_ipc;

mod common;

use std::fs::{self, Metadata};
use std::io;
use std::time::Duration;

use common::{rpc_file_path, DaemonInstance};

use platform_specific::*;

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

    daemon.assert_log_contains("Wrote RPC connection info to", Duration::from_secs(10));

    assert!(rpc_file.exists());
    check_metadata(fs::metadata(&rpc_file).expect("failed to read RPC address file metadata"));
}

#[cfg(unix)]
mod platform_specific {
    extern crate libc;

    use super::*;
    use std::os::unix::fs::MetadataExt;

    pub fn check_metadata(metadata: Metadata) {
        let process_uid = unsafe { libc::getuid() };
        assert_eq!(metadata.uid(), process_uid);
        assert_eq!(metadata.mode() & 0o022, 0);
    }
}

#[cfg(not(unix))]
mod platform_specific {
    use super::*;

    pub fn check_metadata(metadata: Metadata) {
        // TODO: Test when correctly implemented on Windows
    }
}
