use duct;
extern crate os_pipe;

use super::stoppable_process::StoppableProcess;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use self::os_pipe::{pipe, PipeWriter};
use atty;
use shell_escape;
use std::io;
use talpid_types::net;

static BASE_ARGUMENTS: &[&[&str]] = &[
    &["--client"],
    &["--nobind"],
    #[cfg(not(windows))]
    &["--dev", "tun"],
    #[cfg(windows)]
    &["--dev-type", "tun"],
    &["--ping", "3"],
    &["--ping-exit", "15"],
    &["--connect-retry", "0", "0"],
    &["--connect-retry-max", "1"],
    &["--comp-lzo"],
    &["--remote-cert-tls", "server"],
    &["--rcvbuf", "1048576"],
    &["--sndbuf", "1048576"],
    &["--fast-io"],
    &["--cipher", "AES-256-CBC"],
    &["--verb", "3"],
];

static ALLOWED_TLS_CIPHERS: &[&str] = &[
    "TLS-DHE-RSA-WITH-AES-256-GCM-SHA384",
    "TLS-DHE-RSA-WITH-AES-256-CBC-SHA",
];

/// An OpenVPN process builder, providing control over the different arguments that the OpenVPN
/// binary accepts.
#[derive(Clone)]
pub struct OpenVpnCommand {
    openvpn_bin: OsString,
    config: Option<PathBuf>,
    remote: Option<net::Endpoint>,
    user_pass_path: Option<PathBuf>,
    ca: Option<PathBuf>,
    crl: Option<PathBuf>,
    iproute_bin: Option<OsString>,
    plugin: Option<(PathBuf, Vec<String>)>,
    log: Option<PathBuf>,
    tunnel_options: net::OpenVpnTunnelOptions,
    tunnel_alias: Option<OsString>,
}

impl OpenVpnCommand {
    /// Constructs a new `OpenVpnCommand` for launching OpenVPN processes from the binary at
    /// `openvpn_bin`.
    pub fn new<P: AsRef<OsStr>>(openvpn_bin: P) -> Self {
        OpenVpnCommand {
            openvpn_bin: OsString::from(openvpn_bin.as_ref()),
            config: None,
            remote: None,
            user_pass_path: None,
            ca: None,
            crl: None,
            iproute_bin: None,
            plugin: None,
            log: None,
            tunnel_options: net::OpenVpnTunnelOptions::default(),
            tunnel_alias: None,
        }
    }

    /// Sets what configuration file will be given to OpenVPN
    pub fn config<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.config = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the address and protocol that OpenVPN will connect to.
    pub fn remote(&mut self, remote: net::Endpoint) -> &mut Self {
        self.remote = Some(remote);
        self
    }

    /// Sets the path to the file where the username and password for user-pass authentication
    /// is stored. See the `--auth-user-pass` OpenVPN documentation for details.
    pub fn user_pass<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.user_pass_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the path to the CA certificate file.
    pub fn ca<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.ca = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the path to the CRL (Certificate revocation list) file.
    pub fn crl<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.crl = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the path to the ip route command.
    pub fn iproute_bin<S: Into<OsString>>(&mut self, iproute_bin: S) -> &mut Self {
        self.iproute_bin = Some(iproute_bin.into());
        self
    }

    /// Sets a plugin and its arguments that OpenVPN will be started with.
    pub fn plugin<P: AsRef<Path>>(&mut self, path: P, args: Vec<String>) -> &mut Self {
        self.plugin = Some((path.as_ref().to_path_buf(), args));
        self
    }

    /// Sets a log file path.
    pub fn log<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.log = Some(path.as_ref().to_path_buf());
        self
    }

    /// Build a runnable expression from the current state of the command.
    pub fn build(&self) -> duct::Expression {
        debug!("Building expression: {}", &self);
        duct::cmd(&self.openvpn_bin, self.get_arguments()).unchecked()
    }

    /// Sets extra options
    pub fn tunnel_options(&mut self, tunnel_options: &net::OpenVpnTunnelOptions) -> &mut Self {
        self.tunnel_options = *tunnel_options;
        self
    }

    /// Sets the tunnel alias which will be used to identify a tunnel device that will be used by
    /// OpenVPN.
    pub fn tunnel_alias(&mut self, tunnel_alias: Option<OsString>) -> &mut Self {
        self.tunnel_alias = tunnel_alias;
        self
    }

    /// Returns all arguments that the subprocess would be spawned with.
    pub fn get_arguments(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = Self::base_arguments().iter().map(OsString::from).collect();

        if let Some(ref config) = self.config {
            args.push(OsString::from("--config"));
            args.push(OsString::from(config.as_os_str()));
        }

        args.extend(self.remote_arguments().iter().map(OsString::from));
        args.extend(self.authentication_arguments());

        if let Some(ref iproute_bin) = self.iproute_bin {
            args.push(OsString::from("--iproute"));
            args.push(iproute_bin.clone());
        }

        if let Some(ref ca) = self.ca {
            args.push(OsString::from("--ca"));
            args.push(OsString::from(ca.as_os_str()));
        }
        if let Some(ref crl) = self.crl {
            args.push(OsString::from("--crl-verify"));
            args.push(OsString::from(crl.as_os_str()));
        }

        if let Some((ref path, ref plugin_args)) = self.plugin {
            args.push(OsString::from("--plugin"));
            args.push(OsString::from(path));
            args.extend(plugin_args.iter().map(OsString::from));
        }

        if let Some(ref path) = self.log {
            args.push(OsString::from("--log"));
            args.push(OsString::from(path))
        }

        if let Some(mssfix) = self.tunnel_options.mssfix {
            args.push(OsString::from("--mssfix"));
            args.push(OsString::from(mssfix.to_string()));
        }

        if !self.tunnel_options.enable_ipv6 {
            args.push(OsString::from("--pull-filter"));
            args.push(OsString::from("ignore"));
            args.push(OsString::from("route-ipv6"));

            args.push(OsString::from("--pull-filter"));
            args.push(OsString::from("ignore"));
            args.push(OsString::from("ifconfig-ipv6"));
        }

        if let Some(ref tunnel_device) = self.tunnel_alias {
            args.push(OsString::from("--dev-node"));
            args.push(tunnel_device.clone());
        }

        args.extend(Self::security_arguments().iter().map(OsString::from));

        args
    }

    fn base_arguments() -> Vec<&'static str> {
        let mut args = vec![];
        for arglist in BASE_ARGUMENTS.iter() {
            for arg in arglist.iter() {
                args.push(*arg);
            }
        }
        args
    }

    fn security_arguments() -> Vec<String> {
        let mut args = vec![];
        args.push("--tls-cipher".to_owned());
        args.push(ALLOWED_TLS_CIPHERS.join(":"));
        args
    }

    fn remote_arguments(&self) -> Vec<String> {
        let mut args: Vec<String> = vec![];
        if let Some(ref endpoint) = self.remote {
            args.push("--proto".to_owned());
            args.push(match endpoint.protocol {
                net::TransportProtocol::Udp => "udp".to_owned(),
                net::TransportProtocol::Tcp => "tcp-client".to_owned(),
            });

            args.push("--remote".to_owned());
            args.push(endpoint.address.ip().to_string());
            args.push(endpoint.address.port().to_string());
            if !self.tunnel_options.enable_ipv6 {
                args.push(match endpoint.protocol {
                    net::TransportProtocol::Udp => "udp4".to_owned(),
                    net::TransportProtocol::Tcp => "tcp4".to_owned(),
                });
            }
        }
        args
    }

    fn authentication_arguments(&self) -> Vec<OsString> {
        let mut args = vec![];
        if let Some(ref user_pass_path) = self.user_pass_path {
            args.push(OsString::from("--auth-user-pass"));
            args.push(OsString::from(user_pass_path));
        }
        args
    }
}

impl fmt::Display for OpenVpnCommand {
    /// Format the program and arguments of an `OpenVpnCommand` for display. Any non-utf8 data
    /// is lossily converted using the utf8 replacement character.
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&shell_escape::escape(self.openvpn_bin.to_string_lossy()))?;
        for arg in &self.get_arguments() {
            fmt.write_str(" ")?;
            fmt.write_str(&shell_escape::escape(arg.to_string_lossy()))?;
        }
        Ok(())
    }
}

/// Proc handle for an openvpn process
pub struct OpenVpnProcHandle {
    /// Duct handle
    pub inner: duct::Handle,
    /// Standard input handle
    pub stdin: Mutex<Option<PipeWriter>>,
}

/// Impl for proc handle
impl OpenVpnProcHandle {
    /// Constructor for a new openvpn proc handle
    pub fn new(mut cmd: duct::Expression) -> io::Result<Self> {
        if !atty::is(atty::Stream::Stdout) {
            cmd = cmd.stdout_null();
        }

        if !atty::is(atty::Stream::Stderr) {
            cmd = cmd.stderr_null();
        }

        let (reader, writer) = pipe()?;
        let proc_handle = cmd.stdin_handle(reader).start()?;

        Ok(Self {
            inner: proc_handle,
            stdin: Mutex::new(Some(writer)),
        })
    }
}

impl StoppableProcess for OpenVpnProcHandle {
    /// Closes STDIN to stop the openvpn process
    fn stop(&self) {
        let mut stdin = self.stdin.lock().unwrap();
        // Dropping our stdin handle so that it is closed once. Closing the handle should
        // gracefully stop our openvpn child process.
        let _ = stdin.take();
    }

    fn kill(&self) -> io::Result<()> {
        self.inner.kill()
    }

    fn has_stopped(&self) -> io::Result<bool> {
        match self.inner.try_wait() {
            Ok(None) => Ok(false),
            Ok(Some(_)) => Ok(true),
            Err(e) => Err(e),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::OpenVpnCommand;
    use std::ffi::OsString;
    use std::net::Ipv4Addr;
    use talpid_types::net::{Endpoint, TransportProtocol};

    #[test]
    fn passes_one_remote() {
        let remote = Endpoint::new(Ipv4Addr::new(127, 0, 0, 1), 3333, TransportProtocol::Udp);

        let testee_args = OpenVpnCommand::new("").remote(remote).get_arguments();

        assert!(testee_args.contains(&OsString::from("udp")));
        assert!(testee_args.contains(&OsString::from("127.0.0.1")));
        assert!(testee_args.contains(&OsString::from("3333")));
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
