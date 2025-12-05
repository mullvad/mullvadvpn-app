#[cfg(target_os = "linux")]
use nix::unistd::Pid;
#[cfg(target_os = "linux")]
use nix::unistd::{execvp, getgid, getpid, getuid, setgid, setuid};
#[cfg(target_os = "linux")]
use std::fmt::Write as _;
#[cfg(target_os = "linux")]
use std::{
    convert::Infallible,
    env,
    error::Error as StdError,
    ffi::{CString, NulError},
    fs,
    io::{self, BufWriter, Write},
    os::unix::ffi::OsStrExt,
    path::Path,
};
#[cfg(target_os = "linux")]
use talpid_cgroup::SPLIT_TUNNEL_CGROUP_NAME;
#[cfg(target_os = "linux")]
use talpid_cgroup::find_net_cls_mount;

#[cfg(target_os = "linux")]
const PROGRAM_NAME: &str = "mullvad-exclude";

#[cfg(target_os = "linux")]
#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("Invalid arguments")]
    InvalidArguments,

    #[error("Cannot set the cgroup")]
    AddProcToCGroup(#[source] io::Error),

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

fn main() {
    #[cfg(target_os = "linux")]
    // Drop the impossible case
    if let Err(error) = run().map(drop) {
        match error {
            Error::InvalidArguments => {
                let mut args = env::args();
                let program = args.next().unwrap_or_else(|| PROGRAM_NAME.to_string());
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
}

fn add_to_cgroups_v1_if_exists(pid: Pid) -> Result<(), Error> {
    let Some(net_cls_dir) = find_net_cls_mount().map_err(|_| Error::NoProcMounts)? else {
        return Ok(());
    };

    let procs_path = net_cls_dir
        .join(SPLIT_TUNNEL_CGROUP_NAME)
        .join("cgroup.procs");

    let procs_file = fs::OpenOptions::new()
        .write(true)
        .create(false)
        .truncate(false)
        .open(procs_path)
        .map_err(Error::AddProcToCGroup)?;

    BufWriter::new(procs_file)
        .write_all(pid.to_string().as_bytes())
        .map_err(Error::AddProcToCGroup)?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn run() -> Result<Infallible, Error> {
    use talpid_cgroup::{CGROUP2_DEFAULT_MOUNT_PATH, SPLIT_TUNNEL_CGROUP_NAME};

    let mut args_iter = env::args_os().skip(1);
    let program = args_iter.next().ok_or(Error::InvalidArguments)?;
    let program = CString::new(program.as_bytes()).map_err(Error::ArgumentNul)?;

    let args: Vec<CString> = env::args_os()
        .skip(1)
        .map(|arg| CString::new(arg.as_bytes()))
        .collect::<Result<Vec<CString>, NulError>>()
        .map_err(Error::ArgumentNul)?;

    let cgroup_path = Path::new(CGROUP2_DEFAULT_MOUNT_PATH).join(SPLIT_TUNNEL_CGROUP_NAME);

    // Ensure the cgroup2 exists.
    match fs::create_dir(&cgroup_path) {
        Ok(_) => (),

        // cgroup2 already exists, this is fine
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => (),

        Err(e) => {
            // Continue anyway. The next step will probably fail.
            eprintln!("Failed to create mullvad cgroup2 {e}");
        }
    }

    let procs_path = cgroup_path.join("cgroup.procs");
    // Add process PID to cgroup2
    let pid = getpid();
    let result = fs::OpenOptions::new()
        .write(true)
        .create(false)
        .truncate(false)
        .open(procs_path)
        .map(BufWriter::new)
        .and_then(|mut file| file.write_all(pid.to_string().as_bytes()))
        .map_err(Error::AddProcToCGroup);

    // Always add current PID to cgroup1 (deprecated solution). It does not hurt to be in both cgroup1 and cgroup2 at
    // the same time, the firewall will have to promise ttypeso behave appropriately.
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
