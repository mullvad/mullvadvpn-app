extern crate openvpn_ffi;

use super::monitor::{ChildSpawner, ChildMonitor};

use clonablechild::{ClonableChild, ChildExt};

use net::{RemoteAddr, ToRemoteAddrs};

use std::collections::HashMap;
use std::ffi::{OsString, OsStr};
use std::fmt;
use std::io;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::process::{Command, Child, Stdio, ChildStdout, ChildStderr};
use std::sync::{Arc, Mutex};

use talpid_ipc;

error_chain!{
    errors {
        /// Error while communicating with the OpenVPN plugin
        PluginCommunicationError
    }
    links {
        ChildMonitorError(::process::monitor::Error, ::process::monitor::ErrorKind)
        #[doc="Something went wrong in the underlying ChildMonitor"];
    }
}

/// An OpenVPN process builder, providing control over the different arguments that the OpenVPN
/// binary accepts.
#[derive(Clone)]
pub struct OpenVpnCommand {
    openvpn_bin: OsString,
    config: Option<PathBuf>,
    remotes: Vec<RemoteAddr>,
    plugin: Option<(PathBuf, Vec<String>)>,
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
            plugin: None,
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

    /// Sets a plugin and its arguments that OpenVPN will be started with.
    pub fn plugin<P: AsRef<Path>>(&mut self, path: P, args: Vec<String>) -> &mut Self {
        self.plugin = Some((path.as_ref().to_path_buf(), args));
        self
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
        let args = self.get_arguments();
        command.args(&args);
        debug!("Spawning: {}", &self);
        command.spawn()
    }

    fn create_command(&self) -> Command {
        let mut command = Command::new(&self.openvpn_bin);
        command.stdin(Stdio::null())
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
        if let Some((ref path, ref plugin_args)) = self.plugin {
            args.push(OsString::from("--plugin"));
            args.push(OsString::from(path));
            args.extend(plugin_args.iter().map(|arg| OsString::from(arg)));
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


impl ChildSpawner for OpenVpnCommand {
    type Child = ClonableChild;

    fn spawn(&mut self) -> io::Result<ClonableChild> {
        OpenVpnCommand::spawn(self).map(|child| child.into_clonable())
    }
}


/// Possible events from OpenVPN
pub enum OpenVpnEvent {
    /// An event from the plugin loaded into OpenVPN.
    PluginEvent(Result<(openvpn_ffi::OpenVpnPluginEvent, HashMap<String, String>)>),
    /// The OpenVPN process exited. The bool indicates if the process exited cleanly.
    Shutdown(bool),
}

/// A struct able to start and monitor OpenVPN processes.
pub struct OpenVpnMonitor {
    command: OpenVpnCommand,
    plugin_path: PathBuf,
    monitor: ChildMonitor<OpenVpnCommand>,
}

impl OpenVpnMonitor {
    /// Creates a new `OpenVpnMonitor` based on the given command
    pub fn new<P: AsRef<Path>>(command: OpenVpnCommand, plugin_path: P) -> Self {
        OpenVpnMonitor {
            command: command.clone(),
            plugin_path: plugin_path.as_ref().to_path_buf(),
            monitor: ChildMonitor::new(command),
        }
    }

    /// Starts OpenVPN and begins to monitor it.
    pub fn start<L>(&mut self, listener: L) -> Result<(Option<ChildStdout>, Option<ChildStderr>)>
        where L: FnMut(OpenVpnEvent) + Send + 'static
    {
        let shared_listener = Arc::new(Mutex::new(listener));
        self.start_plugin_listener(shared_listener.clone())?;
        self.start_child_monitor(shared_listener)
    }

    fn start_plugin_listener<L>(&mut self, shared_listener: Arc<Mutex<L>>) -> Result<()>
        where L: FnMut(OpenVpnEvent) + Send + 'static
    {
        let server_id = talpid_ipc::start_new_server(move |msg| {
                let chained_msg = msg.chain_err(|| ErrorKind::PluginCommunicationError);
                let mut listener = shared_listener.lock().unwrap();
                (listener.deref_mut())(OpenVpnEvent::PluginEvent(chained_msg));
            }).chain_err(|| ErrorKind::PluginCommunicationError)?;
        self.command.plugin(&self.plugin_path, vec![server_id]);
        Ok(())
    }

    fn start_child_monitor<L>(&mut self,
                              shared_listener: Arc<Mutex<L>>)
                              -> Result<(Option<ChildStdout>, Option<ChildStderr>)>
        where L: FnMut(OpenVpnEvent) + Send + 'static
    {
        let callback = move |clean_exit| {
            let mut listener = shared_listener.lock().unwrap();
            (listener.deref_mut())(OpenVpnEvent::Shutdown(clean_exit));
        };

        self.monitor = ChildMonitor::new(self.command.clone());
        Ok(self.monitor.start(callback)?)
    }

    /// Forwards a stop call to the underlying `ChildMonitor`.
    pub fn stop(&self) -> Result<()> {
        Ok(self.monitor.stop()?)
    }
}


#[cfg(test)]
mod openvpn_command_tests {
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

    #[test]
    fn passes_plugin_path() {
        let path = "./a/path";
        let testee_args = OpenVpnCommand::new("").plugin(path, vec![]).get_arguments();
        assert!(testee_args.contains(&OsString::from("./a/path")));
    }

    #[test]
    fn passes_plugin_args() {
        let args = vec![String::from("123"), String::from("cde")];
        let testee_args = OpenVpnCommand::new("").plugin("", args).get_arguments();
        assert!(testee_args.contains(&OsString::from("123")));
        assert!(testee_args.contains(&OsString::from("cde")));
    }
}


#[cfg(test)]
mod openvpn_monitor_tests {
    use super::*;

    #[test]
    fn stop_without_start() {
        let command = OpenVpnCommand::new("");
        let testee = OpenVpnMonitor::new(command, "");

        use super::super::monitor::ErrorKind::InvalidState as MInvalidState;
        assert_matches!(testee.stop(), Err(Error(ErrorKind::ChildMonitorError(MInvalidState), _)));
    }
}
