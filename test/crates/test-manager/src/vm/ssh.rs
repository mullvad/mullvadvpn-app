/// A very thin wrapper on top of `ssh2`.
use anyhow::{Context, Result};
use ssh2::Session;
use std::{
    io::Read,
    net::{IpAddr, SocketAddr, TcpStream},
};

/// Default `ssh` port.
const PORT: u16 = 22;

/// Handle to an `ssh` session.
pub struct SSHSession {
    session: ssh2::Session,
}

impl SSHSession {
    /// Create a new `ssh` session.
    /// This function is blocking while connecting.
    ///
    /// The tunnel is closed when the `SSHSession` is dropped.
    pub fn connect(username: String, password: String, ip: IpAddr) -> Result<Self> {
        // Set up the SSH connection
        log::info!("initializing a new SSH session ..");
        let stream = TcpStream::connect(SocketAddr::new(ip, PORT)).context("TCP connect failed")?;
        let mut session = Session::new().context("Failed to connect to SSH server")?;
        session.set_tcp_stream(stream);
        session.handshake()?;
        session
            .userauth_password(&username, &password)
            .context("SSH auth failed")?;
        Ok(Self { session })
    }

    /// Execute an arbitrary string of commands via ssh.
    pub fn exec_blocking(&self, command: &str) -> Result<String> {
        let session = &self.session;
        let mut channel = session.channel_session()?;
        channel.exec(command)?;
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.send_eof()?;
        channel.wait_eof()?;
        channel.wait_close()?;
        Ok(output)
    }
}
