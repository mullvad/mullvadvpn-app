#![cfg(target_os = "linux")]
use anyhow::{Context as _, anyhow};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

pub mod v1;
pub mod v2;

pub const SPLIT_TUNNEL_CGROUP_NAME: &str = "mullvad-exclusions";

/// The path where linux normally mounts the cgroup2 filesystem.
pub const CGROUP2_DEFAULT_MOUNT_PATH: &str = "/sys/fs/cgroup";

/// The path where linux normally mounts the net_cls cgroup v1 filesystem.
pub const DEFAULT_NET_CLS_DIR: &str = "/sys/fs/cgroup/net_cls";

/// Errors related to cgroups.
#[derive(thiserror::Error, Debug)]
#[error("CGroup error")]
pub struct Error(#[from] anyhow::Error);

/// Return whether systemd is the init system and manages cgroups v2 on this system
pub fn is_systemd_managed() -> bool {
    let systemd_is_init_system = fs::read("/proc/1/comm")
        .map(|comm| comm == b"systemd\n")
        .unwrap_or(false);

    // TODO: This is not perfect at detecting whether systemd manages cgroup2
    systemd_is_init_system && find_cgroup2_mount().is_ok()
}

/// Find the path of the cgroup v1 net_cls controller mount if it exists.
///
/// Returns an error if `/proc/mounts` does not exist.
pub fn find_net_cls_mount() -> Result<Option<PathBuf>, Error> {
    let mounts =
        fs::read("/proc/mounts").with_context(|| anyhow!("Failed to stat `/proc/mounts`"))?;
    Ok(find_net_cls_mount_inner(&mounts))
}

/// Find the path of the cgroup v2 mount.
///
/// Returns an error if `/proc/mounts` does not exist.
pub fn find_cgroup2_mount() -> Result<Option<PathBuf>, Error> {
    let mounts =
        fs::read("/proc/mounts").with_context(|| anyhow!("Failed to stat `/proc/mounts`"))?;
    Ok(find_cgroup2_mount_inner(&mounts))
}

fn find_cgroup2_mount_inner(mounts: &[u8]) -> Option<PathBuf> {
    mounts.split(|byte| *byte == b'\n').find_map(|line| {
        // There can only be one cgroup2 hierarchy, which looks like this:
        // cgroup2 /sys/fs/cgroup cgroup2 ...
        let mut parts = line.split(|byte| *byte == b' ');
        let _device_type = parts.next()?;
        let mount_path = parts.next()?;
        let filesystem_type = parts.next()?;

        if filesystem_type != b"cgroup2" {
            return None;
        }

        Some(PathBuf::from(OsStr::from_bytes(mount_path)))
    })
}

fn find_net_cls_mount_inner(mounts: &[u8]) -> Option<PathBuf> {
    mounts
        .split(|byte| *byte == b'\n')
        .find_map(parse_net_cls_mount_line)
}

fn parse_net_cls_mount_line(line: &[u8]) -> Option<PathBuf> {
    // Each line contains multiple values separated by space.
    // `cgroup /sys/fs/cgroup/net_cls,net_prio cgroup
    // rw,nosuid,nodev,noexec,relatime,net_cls,net_prio 0 0`  Value meanings:
    // 1. device type
    // 2. mount path
    // 3. filesystem type
    // 4. mount options
    // 5./6. legacy dummy values
    let mut parts = line.split(|byte| *byte == b' ');
    let _device_type = parts.next()?;
    let mount_path = parts.next()?;
    let filesystem_type = parts.next()?;
    let mount_options = parts.next()?;
    // The expected device type and fs type is "cgroup";
    if filesystem_type != b"cgroup" {
        return None;
    }

    if !mount_options
        .split(|byte| *byte == b',')
        .any(|key| key == b"net_cls")
    {
        return None;
    }

    Some(PathBuf::from(OsStr::from_bytes(mount_path)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_net_cls_path() {
        let input =
            br#"cgroup /sys/fs/cgroup/memory cgroup rw,nosuid,nodev,noexec,relatime,memory 0 0
cgroup /sys/fs/cgroup/net_cls,net_prio cgroup rw,nosuid,nodev,noexec,relatime,net_cls,net_prio 0 0
"#;

        assert_eq!(
            find_net_cls_mount_inner(input),
            Some(PathBuf::from("/sys/fs/cgroup/net_cls,net_prio"))
        )
    }

    #[test]
    fn test_fail_to_find_net_cls_path() {
        let input =
            br#"cgroup /sys/fs/cgroup/memory cgroup rw,nosuid,nodev,noexec,relatime,memory 0 0
cgroup /sys/fs/cgroup/net_cls,net_prio cgroup rw,nosuid,nodev,noexec,relatime,,net_prio 0 0
cgroup /sys/fs/cgroup/net_cls,net_prio garbage rw,nosuid,nodev,noexec,relatime,net_cls,net_prio 0 0
cgroup /nope
"#;

        assert_eq!(find_net_cls_mount_inner(input), None)
    }

    #[test]
    fn test_find_cgroup2_path() {
        let input =
            br#"cgroup2 /sys/fs/cgroup cgroup2 rw,seclabel,nosuid,nodev,noexec,relatime,nsdelegate,memory_recursiveprot 0 0
"#;

        assert_eq!(
            find_cgroup2_mount_inner(input),
            Some(PathBuf::from("/sys/fs/cgroup"))
        )
    }

    #[test]
    fn test_fail_to_find_cgroup2_path() {
        let input =
            br#"cgroup /sys/fs/cgroup/memory cgroup rw,nosuid,nodev,noexec,relatime,memory 0 0
cgroup /sys/fs/cgroup/net_cls,net_prio cgroup rw,nosuid,nodev,noexec,relatime,,net_prio 0 0
cgroup /sys/fs/cgroup/net_cls,net_prio garbage rw,nosuid,nodev,noexec,relatime,net_cls,net_prio 0 0
cgroup /nope
"#;

        assert_eq!(find_cgroup2_mount_inner(input), None)
    }
}
