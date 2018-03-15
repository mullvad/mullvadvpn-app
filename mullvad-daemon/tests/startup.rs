#[macro_use]
extern crate duct;
#[cfg(unix)]
extern crate libc;
extern crate os_pipe;

mod common;

use std::fs;
use std::fs::Metadata;
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

    daemon.assert_log_contains(
        "Mullvad management interface listening on",
        Duration::from_secs(10),
    );
    assert!(rpc_file.exists());

    check_metadata(fs::metadata(&rpc_file).expect("failed to read RPC address file metadata"));
}

#[cfg(unix)]
mod platform_specific {
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
    pub fn check_metadata() {
        // TODO: Test when correctly implemented on Windows
    }
}
