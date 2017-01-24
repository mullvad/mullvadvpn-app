use net::{RemoteAddr, ToRemoteAddrs};

use std::ffi::{OsString, OsStr};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Child, Stdio, ChildStdout, ChildStderr};

/// A module for monitoring child processes and get notified of events on them.
pub mod monitor;
use self::monitor::{MonitoredChild, ChildSpawner};

use clonablechild::{ClonableChild, ChildExt};

/// An OpenVPN process builder, providing control over the different arguments that the OpenVPN
/// binary accepts.
pub struct OpenVpnCommand {
    openvpn_bin: OsString,
    config: Option<PathBuf>,
    remotes: Vec<RemoteAddr>,
    pipe_output: bool,
}

impl OpenVpnCommand {
    /// Constructs a new `OpenVpnCommand` for launching OpenVPN processes from the binary at
    /// `openvpn_bin`.
    pub fn new<P: AsRef<OsStr>>(openvpn_bin: P) -> Self {
        OpenVpnCommand {
            openvpn_bin: OsString::from(openvpn_bin.as_ref()),
            config: None,
            remotes: vec![],
            pipe_output: true,
        }
    }

    /// Sets what configuration file will be given to OpenVPN
    pub fn config<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.config = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the addresses that OpenVPN will connect to. See OpenVPN documentation for how multiple
    /// remotes are handled.
    pub fn remotes<A: ToRemoteAddrs>(&mut self, remotes: A) -> io::Result<&mut Self> {
        self.remotes = remotes.to_remote_addrs()?.collect();
        Ok(self)
    }

    /// If piping the standard streams, stdout and stderr will be available to the parent process.
    /// This is the default behavior. If you want the equivalence of attaching the child streams to
    /// /dev/null, invoke this method with false.
    pub fn pipe_output(&mut self, pipe_output: bool) -> &mut Self {
        self.pipe_output = pipe_output;
        self
    }

    /// Executes the OpenVPN process as a child process, returning a handle to it.
    pub fn spawn(&self) -> io::Result<Child> {
        let mut command = self.create_command();
        command.args(&self.get_arguments());
        command.spawn()
    }

    fn create_command(&self) -> Command {
        let mut command = Command::new(&self.openvpn_bin);
        command.env_clear()
            .stdin(Stdio::null())
            .stdout(self.get_output_pipe_policy())
            .stderr(self.get_output_pipe_policy());
        command
    }

    fn get_output_pipe_policy(&self) -> Stdio {
        if self.pipe_output {
            Stdio::piped()
        } else {
            Stdio::null()
        }
    }

    /// Returns all arguments that the subprocess would be spawned with.
    pub fn get_arguments(&self) -> Vec<OsString> {
        let mut args = vec![];
        if let Some(ref config) = self.config {
            args.push(OsString::from("--config"));
            args.push(OsString::from(config.as_os_str()));
        }
        for remote in &self.remotes {
            args.push(OsString::from("--remote"));
            args.push(OsString::from(remote.address()));
            args.push(OsString::from(remote.port().to_string()));
        }
        args
    }
}

impl fmt::Display for OpenVpnCommand {
    /// Format the program and arguments of an `OpenVpnCommand` for display. Any non-utf8 data is
    /// lossily converted using the utf8 replacement character.
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.openvpn_bin.to_string_lossy())?;
        for arg in self.get_arguments().iter().map(|arg| arg.to_string_lossy()) {
            write_argument(fmt, &arg)?;
        }
        Ok(())
    }
}

fn write_argument(fmt: &mut fmt::Formatter, arg: &str) -> fmt::Result {
    fmt.write_str(" ")?;
    let quote = arg.contains(char::is_whitespace);
    if quote {
        fmt.write_str("\"")?;
    }
    fmt.write_str(arg)?;
    if quote {
        fmt.write_str("\"")?;
    }
    Ok(())
}


impl MonitoredChild for ClonableChild {
    fn wait(&self) -> io::Result<bool> {
        ClonableChild::wait(self).map(|exit_status| exit_status.success())
    }

    fn kill(&self) -> io::Result<()> {
        ClonableChild::kill(self)
    }

    fn stdout(&mut self) -> Option<ChildStdout> {
        self.stdout()
    }

    fn stderr(&mut self) -> Option<ChildStderr> {
        self.stderr()
    }
}

impl ChildSpawner<ClonableChild> for OpenVpnCommand {
    fn spawn(&mut self) -> io::Result<ClonableChild> {
        OpenVpnCommand::spawn(self).map(|child| child.into_clonable())
    }
}


#[cfg(test)]
mod tests {
    use super::OpenVpnCommand;
    use net::RemoteAddr;
    use std::ffi::OsString;

    #[test]
    fn no_arguments() {
        let testee_args = OpenVpnCommand::new("").get_arguments();
        assert_eq!(0, testee_args.len());
    }

    #[test]
    fn passes_one_remote() {
        let remote = RemoteAddr::new("example.com", 3333);

        let testee_args = OpenVpnCommand::new("").remotes(remote).unwrap().get_arguments();

        assert!(testee_args.contains(&OsString::from("example.com")));
        assert!(testee_args.contains(&OsString::from("3333")));
    }

    #[test]
    fn passes_two_remotes() {
        let remotes = vec![RemoteAddr::new("127.0.0.1", 998), RemoteAddr::new("fe80::1", 1337)];

        let testee_args = OpenVpnCommand::new("").remotes(&remotes[..]).unwrap().get_arguments();

        assert!(testee_args.contains(&OsString::from("127.0.0.1")));
        assert!(testee_args.contains(&OsString::from("998")));
        assert!(testee_args.contains(&OsString::from("fe80::1")));
        assert!(testee_args.contains(&OsString::from("1337")));
    }

    #[test]
    fn accepts_str() {
        assert!(OpenVpnCommand::new("").remotes("10.0.0.1:1377").is_ok());
    }

    #[test]
    fn accepts_slice_of_str() {
        let remotes = ["10.0.0.1:1337", "127.0.0.1:99"];

        let testee_args = OpenVpnCommand::new("").remotes(&remotes[..]).unwrap().get_arguments();

        assert!(testee_args.contains(&OsString::from("10.0.0.1")));
        assert!(testee_args.contains(&OsString::from("1337")));
        assert!(testee_args.contains(&OsString::from("127.0.0.1")));
        assert!(testee_args.contains(&OsString::from("99")));
    }
}
