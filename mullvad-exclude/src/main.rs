#[cfg(target_os = "linux")]
use nix::unistd::{execvp, getgid, getpid, getuid, setgid, setuid};
#[cfg(target_os = "linux")]
use std::{
    env,
    error::Error as StdError,
    ffi::{CStr, CString},
    fs, io,
};

#[cfg(target_os = "linux")]
const CGROUP_PROCS_PATH: &str = "/sys/fs/cgroup/net_cls/mullvad-exclusions/cgroup.procs";

#[cfg(target_os = "linux")]
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
enum Error {
    #[error(display = "Invalid arguments")]
    InvalidArguments,

    #[error(display = "Cannot set the cgroup")]
    AddProcToCGroup(#[error(source)] io::Error),

    #[error(display = "Cannot set the uid")]
    SetUid(#[error(source)] nix::Error),

    #[error(display = "Cannot set the gid")]
    SetGid(#[error(source)] nix::Error),

    #[error(display = "Failed to launch the process")]
    Exec(#[error(source)] nix::Error),
}

fn main() {
    #[cfg(target_os = "linux")]
    match run() {
        Err(Error::InvalidArguments) => {
            let mut args = env::args();
            eprintln!("Usage: {} <command>", args.next().unwrap());
            std::process::exit(1);
        }
        Err(e) => {
            let mut s = format!("{}", e);
            let mut source = e.source();
            while let Some(error) = source {
                s.push_str(&format!("\nCaused by: {}", error));
                source = error.source();
            }
            eprintln!("{}", s);

            std::process::exit(1);
        }
        _ => unreachable!("execv returned unexpectedly"),
    }
}

#[cfg(target_os = "linux")]
fn run() -> Result<void::Void, Error> {
    if env::args().len() < 2 {
        return Err(Error::InvalidArguments);
    }

    // Set the cgroup of this process
    fs::write(CGROUP_PROCS_PATH, getpid().to_string().as_bytes())
        .map_err(Error::AddProcToCGroup)?;

    // Drop root privileges
    let real_uid = getuid();
    setuid(real_uid).map_err(Error::SetUid)?;
    let real_gid = getgid();
    setgid(real_gid).map_err(Error::SetGid)?;

    // Launch the process
    let args: Vec<CString> = env::args().map(|arg| CString::new(arg).unwrap()).collect();
    let args: Vec<&CStr> = args.iter().map(|arg| &**arg).collect();
    execvp(args[1], &args[1..]).map_err(Error::Exec)
}
