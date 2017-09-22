use duct;

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};

use talpid_types::net;

static BASE_ARGUMENTS: &[&[&str]] = &[
    &["--client"],
    &["--nobind"],
    &["--dev", "tun"],
    &["--ping", "3"],
    &["--ping-exit", "15"],
    &["--connect-retry", "0", "0"],
    &["--connect-retry-max", "1"],
    &["--comp-lzo"],
    &["--remote-cert-tls", "server"],
];

static ALLOWED_TLS_CIPHERS: &[&str] = &[
    "TLS-DHE-RSA-WITH-AES-256-GCM-SHA384",
    "TLS-DHE-RSA-WITH-AES-256-CBC-SHA",
    "TLS-DHE-RSA-WITH-CAMELLIA-256-CBC-SHA",
    "TLS-DHE-RSA-WITH-AES-128-CBC-SHA",
    "TLS-DHE-RSA-WITH-SEED-CBC-SHA",
    "TLS-DHE-RSA-WITH-CAMELLIA-128-CBC-SHA",
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
    plugin: Option<(PathBuf, Vec<String>)>,
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
            plugin: None,
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

    /// Sets a plugin and its arguments that OpenVPN will be started with.
    pub fn plugin<P: AsRef<Path>>(&mut self, path: P, args: Vec<String>) -> &mut Self {
        self.plugin = Some((path.as_ref().to_path_buf(), args));
        self
    }

    /// Build a runnable expression from the current state of the command.
    pub fn build(&self) -> duct::Expression {
        debug!("Building expression: {}", &self);
        duct::cmd(&self.openvpn_bin, self.get_arguments()).unchecked()
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

        if let Some(ref ca) = self.ca {
            args.push(OsString::from("--ca"));
            args.push(OsString::from(ca.as_os_str()));
        }

        if let Some((ref path, ref plugin_args)) = self.plugin {
            args.push(OsString::from("--plugin"));
            args.push(OsString::from(path));
            args.extend(plugin_args.iter().map(OsString::from));
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
