use std::ffi::{OsString, OsStr};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Child, Stdio};

/// An OpenVPN process builder, providing control over the different arguments that the OpenVPN
/// binary accepts.
pub struct OpenVpnBuilder {
    openvpn_bin: OsString,
    config: Option<PathBuf>,
}

impl OpenVpnBuilder {
    /// Constructs a new `OpenVpnBuilder` for launching OpenVPN processes from the binary at
    /// `openvpn_bin`.
    pub fn new<P: AsRef<OsStr>>(openvpn_bin: P) -> Self {
        OpenVpnBuilder {
            openvpn_bin: OsString::from(openvpn_bin.as_ref()),
            config: None,
        }
    }

    /// Sets what configuration file will be given to OpenVPN
    pub fn config<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.config = Some(path.as_ref().to_path_buf());
        self
    }

    /// Executes the OpenVPN process as a child process, returning a handle to it.
    pub fn spawn(&mut self) -> io::Result<Child> {
        let mut command = self.create_command();
        self.apply_settings(&mut command);
        command.spawn()
    }

    fn create_command(&mut self) -> Command {
        let mut command = Command::new(&self.openvpn_bin);
        command.env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command
    }

    fn apply_settings(&self, command: &mut Command) {
        if let Some(ref config) = self.config {
            command.arg("--config").arg(config);
        }
    }
}
