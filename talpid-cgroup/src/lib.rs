#![cfg(target_os = "linux")]
use anyhow::{Context as _, anyhow};
use nix::{libc::pid_t, unistd::Pid};
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
pub enum Error {
    /// The PID does not name a live process, so it cannot be moved between cgroups.
    ///
    /// Callers that are moving a process to a particular cgroup can usually treat this as
    /// success, since a process that no longer exists is not in any cgroup.
    #[error("No such process: {0}")]
    NoSuchProcess(Pid),

    /// Any other cgroup error.
    #[error("CGroup error")]
    Other(#[from] anyhow::Error),
}

/// Move `pids` into another cgroup by passing each of them to `add`.
///
/// PIDs that exit before they can be moved are ignored: a process that no longer exists is
/// already out of the source cgroup, which is the state we are asked to produce. Any other
/// failure is reported, but only after every remaining PID has been attempted — one failing PID
/// must not strand the rest of them in the source cgroup.
fn move_pids(pids: Vec<pid_t>, mut add: impl FnMut(Pid) -> Result<(), Error>) -> Result<(), Error> {
    let mut last_error = None;

    for pid in pids {
        let pid = Pid::from_raw(pid);
        match add(pid) {
            Ok(()) => (),
            Err(Error::NoSuchProcess(pid)) => {
                log::debug!("Process {pid} exited before it could be moved out of the cgroup");
            }
            Err(error) => {
                log::error!("Failed to move process {pid} out of the cgroup: {error:?}");
                last_error = Some(error);
            }
        }
    }

    match last_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

/// Find the path of the cgroup v1 net_cls controller mount if it exists.
///
/// Returns an error if `/proc/mounts` does not exist.
pub fn find_net_cls_mount() -> Result<Option<PathBuf>, Error> {
    let mounts =
        fs::read("/proc/mounts").with_context(|| anyhow!("Failed to stat `/proc/mounts`"))?;
    Ok(find_net_cls_mount_inner(&mounts))
}

fn find_net_cls_mount_inner(mounts: &[u8]) -> Option<PathBuf> {
    mounts
        .split(|byte| *byte == b'\n')
        .find_map(parse_mount_line)
}

fn parse_mount_line(line: &[u8]) -> Option<PathBuf> {
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
    use std::cell::RefCell;

    /// The error produced when a PID no longer names a live process.
    fn no_such_process(pid: pid_t) -> Error {
        Error::NoSuchProcess(Pid::from_raw(pid))
    }

    /// An error that is *not* a dead process, and so must be reported.
    fn write_failed() -> Error {
        Error::Other(anyhow!("Failed to add process to cgroup"))
    }

    /// Run [`move_pids`], recording which PIDs it attempted to move.
    fn move_recording(
        pids: Vec<pid_t>,
        add: impl Fn(pid_t) -> Result<(), Error>,
    ) -> (Result<(), Error>, Vec<pid_t>) {
        let attempted = RefCell::new(vec![]);
        let result = move_pids(pids, |pid| {
            attempted.borrow_mut().push(pid.as_raw());
            add(pid.as_raw())
        });
        (result, attempted.into_inner())
    }

    /// A PID that exits between being listed and being moved must not abort the operation.
    ///
    /// This is the regression test for `mullvad split-tunnel clear` failing with
    /// `FailedPrecondition` when the cgroup contained a PID that was no longer live.
    #[test]
    fn dead_pid_does_not_abort_move() {
        let (result, attempted) = move_recording(vec![1, 2, 3], |pid| {
            if pid == 2 {
                Err(no_such_process(pid))
            } else {
                Ok(())
            }
        });

        assert!(result.is_ok(), "move should have succeeded, got {result:?}");
        assert_eq!(
            attempted,
            vec![1, 2, 3],
            "every PID should be attempted, including those after the dead PID"
        );
    }

    /// A cgroup containing nothing but dead PIDs empties successfully.
    #[test]
    fn all_pids_dead_is_success() {
        let (result, attempted) = move_recording(vec![1, 2], |pid| Err(no_such_process(pid)));

        assert!(result.is_ok(), "move should have succeeded, got {result:?}");
        assert_eq!(attempted, vec![1, 2]);
    }

    /// A genuine failure is still reported, but only after the other PIDs have been moved.
    #[test]
    fn real_error_is_reported_but_does_not_abort_move() {
        let (result, attempted) = move_recording(vec![1, 2, 3], |pid| match pid {
            1 => Err(write_failed()),
            _ => Ok(()),
        });

        assert!(result.is_err(), "move should have reported the failure");
        assert_eq!(
            attempted,
            vec![1, 2, 3],
            "a failing PID should not strand the remaining PIDs in the cgroup"
        );
    }

    /// A dead PID must not mask a real error from elsewhere in the list.
    #[test]
    fn dead_pid_does_not_mask_real_error() {
        let (result, _) = move_recording(vec![1, 2], |pid| {
            if pid == 1 {
                Err(write_failed())
            } else {
                Err(no_such_process(pid))
            }
        });

        assert!(
            matches!(result, Err(Error::Other(_))),
            "the real error should be reported, got {result:?}"
        );
    }

    #[test]
    fn moving_an_empty_cgroup_is_success() {
        let (result, attempted) = move_recording(vec![], |_| Ok(()));

        assert!(result.is_ok(), "move should have succeeded, got {result:?}");
        assert!(attempted.is_empty());
    }

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
cgroup /sys/fs/cgroup/net_cls,net_prio cgroup2 rw,nosuid,nodev,noexec,relatime,net_cls,net_prio 0 0
cgroup /sys/fs/cgroup/net_cls,net_prio garbage rw,nosuid,nodev,noexec,relatime,net_cls,net_prio 0 0
cgroup /nope
"#;

        assert_eq!(find_net_cls_mount_inner(input), None)
    }
}
