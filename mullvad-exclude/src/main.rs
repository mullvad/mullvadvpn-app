#![cfg(target_os = "linux")]
use nix::unistd::{Pid, execvp, getgid, getpid, getuid, setgid, setuid};
use std::{
    convert::Infallible,
    env,
    error::Error as StdError,
    ffi::{CString, NulError},
    fmt::Write as _,
    os::unix::ffi::OsStrExt,
};
use talpid_cgroup::{SPLIT_TUNNEL_CGROUP_NAME, find_net_cls_mount, v1::CGroup1, v2::CGroup2};

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("Invalid arguments")]
    InvalidArguments,

    #[error("Cannot assing process to cgroup")]
    AddProcToCGroup(#[from] talpid_cgroup::Error),

    #[error("Failed to drop root user privileges for the process")]
    DropRootUid(#[source] nix::Error),

    #[error("Failed to drop root group privileges for the process")]
    DropRootGid(#[source] nix::Error),

    #[error("Failed to launch the process")]
    Exec(#[source] nix::Error),

    #[error("An argument contains interior nul bytes")]
    ArgumentNul(#[source] NulError),

    #[error("Failed to stat /proc/mounts")]
    NoProcMounts,
}

/// Launch a program in a cgroup where traffic will be excluded from the VPN tunnel.
///
/// Note: Set the `TALPID_EXCLUSSION_CGROUP` env variable to control where the root cgroup is
/// mounted. See (README.md)[../../README.md#Environment-variables-used-by-the-service] for
/// details.
fn main() {
    let Err(error) = run();

    match error {
        Error::InvalidArguments => {
            let mut args = env::args();
            let program = args
                .next()
                .unwrap_or_else(|| env!("CARGO_PKG_NAME").to_string());
            eprintln!("Usage: {program} COMMAND [ARGS]");
            std::process::exit(1);
        }
        e => {
            let mut s = format!("Error: {e}");
            let mut source = e.source();
            while let Some(error) = source {
                write!(&mut s, "\nCaused by: {error}").expect("formatting failed");
                source = error.source();
            }
            eprintln!("{s}");

            std::process::exit(1);
        }
    }
}

fn add_to_cgroups_v1_if_exists(pid: Pid) -> Result<(), Error> {
    let Some(net_cls_dir) = find_net_cls_mount().map_err(|_| Error::NoProcMounts)? else {
        return Ok(());
    };

    let cgroup_path = net_cls_dir.join(SPLIT_TUNNEL_CGROUP_NAME);

    CGroup1::open(cgroup_path)
        .and_then(|cgroup| cgroup.add_pid(pid))
        .map_err(Error::from)
}

fn run() -> Result<Infallible, Error> {
    let mut args_iter = env::args_os().skip(1);
    let program = args_iter.next().ok_or(Error::InvalidArguments)?;
    let program = CString::new(program.as_bytes()).map_err(Error::ArgumentNul)?;

    let args: Vec<CString> = env::args_os()
        .skip(1)
        .map(|arg| CString::new(arg.as_bytes()))
        .collect::<Result<Vec<CString>, NulError>>()
        .map_err(Error::ArgumentNul)?;

    let pid = getpid();

    let result = CGroup2::open_root()
        .and_then(|root_cgroup2| root_cgroup2.create_or_open_child(SPLIT_TUNNEL_CGROUP_NAME))
        .and_then(|exclusion_cgroup2| exclusion_cgroup2.add_pid(pid));

    // Always add current PID to cgroup1 (deprecated solution). It does not hurt to be in both cgroup1 and cgroup2 at
    // the same time, the firewall will have to promise to behave appropriately.
    if let Err(add_err) = add_to_cgroups_v1_if_exists(pid)
        && result.is_err()
    {
        eprintln!("Failed to add process to v1 cgroup: {add_err}");
    }

    result?;

    // Drop root privileges
    let real_uid = getuid();
    setuid(real_uid).map_err(Error::DropRootUid)?;
    let real_gid = getgid();
    setgid(real_gid).map_err(Error::DropRootGid)?;

    // Launch the process
    execvp(&program, &args).map_err(Error::Exec)
}
