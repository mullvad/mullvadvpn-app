extern crate openvpn_ffi;

use super::monitor::ChildMonitor;

use duct;

use net::{RemoteAddr, ToRemoteAddrs};

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::{Arc, Mutex};

use talpid_ipc;

error_chain!{
    errors {
        /// Error while communicating with the OpenVPN plugin.
        PluginCommunicationError {
            description("Error while communicating with the OpenVPN plugin")
        }
        /// Error while trying to spawn OpenVPN process.
        ChildSpawnError {
            description("Error while trying to spawn OpenVPN process")
        }
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

    /// Build a runnable expression from the current state of the command.
    pub fn build(&self) -> duct::Expression {
        debug!("Building expression: {}", &self);
        duct::cmd(&self.openvpn_bin, self.get_arguments())
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
            args.extend(plugin_args.iter().map(OsString::from));
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


/// Possible events from OpenVPN
pub enum OpenVpnEvent {
    /// An event from the plugin loaded into OpenVPN.
    PluginEvent(talpid_ipc::Result<(openvpn_ffi::OpenVpnPluginEvent, openvpn_ffi::OpenVpnEnv)>),
    /// The OpenVPN process exited. Containing the result of waiting for the process.
    Shutdown(io::Result<process::ExitStatus>),
}

/// A struct able to start and monitor OpenVPN processes.
pub struct OpenVpnMonitor {
    child: ChildMonitor,
}

impl OpenVpnMonitor {
    /// Spawns a new OpenVPN process and monitors it for exit and events.
    pub fn start<P, L>(mut cmd: OpenVpnCommand, plugin_path: P, listener: L) -> Result<Self>
        where P: AsRef<Path>,
              L: FnMut(OpenVpnEvent) + Send + 'static
    {
        let shared_listener = Arc::new(Mutex::new(listener));
        let server_id = Self::start_plugin_listener(shared_listener.clone())?;
        cmd.plugin(plugin_path, vec![server_id]);
        let child = Self::start_child_monitor(&cmd, shared_listener)?;
        Ok(OpenVpnMonitor { child })
    }

    fn start_plugin_listener<L>(shared_listener: Arc<Mutex<L>>) -> Result<String>
        where L: FnMut(OpenVpnEvent) + Send + 'static
    {
        talpid_ipc::start_new_server(
            move |msg| {
                let mut listener = shared_listener.lock().unwrap();
                (listener.deref_mut())(OpenVpnEvent::PluginEvent(msg));
            },
        )
                .chain_err(|| ErrorKind::PluginCommunicationError)
    }

    fn start_child_monitor<L>(cmd: &OpenVpnCommand,
                              shared_listener: Arc<Mutex<L>>)
                              -> Result<ChildMonitor>
        where L: FnMut(OpenVpnEvent) + Send + 'static
    {
        let on_exit = move |result: io::Result<&process::Output>| {
            let status = result.map(|out: &process::Output| out.status.clone());
            let mut listener = shared_listener.lock().unwrap();
            (listener.deref_mut())(OpenVpnEvent::Shutdown(status));
        };
        ChildMonitor::start(&cmd.build(), on_exit).chain_err(|| ErrorKind::ChildSpawnError)
    }

    /// Send a kill signal to the OpenVPN process.
    pub fn kill(&self) -> io::Result<()> {
        self.child.kill()
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
        let remotes = vec![
            RemoteAddr::new("127.0.0.1", 998),
            RemoteAddr::new("fe80::1", 1337),
        ];

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
