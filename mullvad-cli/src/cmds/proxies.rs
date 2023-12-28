use anyhow::{anyhow, Result};
use talpid_types::net::{openvpn::SHADOWSOCKS_CIPHERS, TransportProtocol};
use std::net::IpAddr;

use clap::Args;

/// A minimal wrapper type allowing the user to supply a list index to some
/// Access Method.
#[derive(Args, Debug, Clone)]
pub struct SelectItem {
    /// Which access method to pick
    index: usize,
}

impl SelectItem {
    /// Transform human-readable (1-based) index to 0-based indexing.
    pub fn as_array_index(&self) -> Result<usize> {
        self.index
            .checked_sub(1)
            .ok_or(anyhow!("Access method 0 does not exist"))
    }
}

impl std::fmt::Display for SelectItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.index)
    }
}

#[derive(Args, Debug, Clone)]
pub struct Socks5LocalAdd {
    /// The port that the server on localhost is listening on
    pub local_port: u16,
    /// The IP of the remote peer
    pub remote_ip: IpAddr,
    /// The port of the remote peer
    pub remote_port: u16,
    /// The Mullvad App can not know which transport protocol that the
    /// remote peer accepts, but it needs to know this in order to correctly
    /// exempt the connection traffic in the firewall.
    ///
    /// By default, the transport protocol is assumed to be `TCP`, but it
    /// can optionally be set to `UDP` as well.
    #[arg(long, default_value_t = TransportProtocol::Tcp)]
    pub transport_protocol: TransportProtocol,
}

// TODO: Write comment about why remote does not allow you to set the transport protocol
#[derive(Args, Debug, Clone)]
pub struct Socks5RemoteAdd {
        /// The IP of the remote proxy server
        pub remote_ip: IpAddr,
        /// The port of the remote proxy server
        pub remote_port: u16,

        #[clap(flatten)]
        pub authentication: Option<SocksAuthentication>,
}

#[derive(Args, Debug, Clone)]
pub struct ShadowsocksAdd {
    /// The IP of the remote Shadowsocks-proxy
    pub remote_ip: IpAddr,
    /// Port on which the remote Shadowsocks-proxy listens for traffic
    pub remote_port: u16,
    /// Password for authentication
    pub password: String,
    /// Cipher to use
    #[arg(long, value_parser = SHADOWSOCKS_CIPHERS)]
    pub cipher: String,
}

#[derive(Args, Debug, Clone)]
#[group(requires_all = ["username", "password"])] // https://github.com/clap-rs/clap/issues/5092
pub struct SocksAuthentication {
    /// Username for authentication against a remote SOCKS5 proxy
    #[arg(short, long, required = false)]
    pub username: String,
    /// Password for authentication against a remote SOCKS5 proxy
    #[arg(short, long, required = false)]
    pub password: String,
}

#[derive(Args, Debug, Clone)]
pub struct EditParams {
    /// Username for authentication [Socks5 (Remote proxy)]
    #[arg(long)]
    username: Option<String>,
    /// Password for authentication [Socks5 (Remote proxy), Shadowsocks]
    #[arg(long)]
    password: Option<String>,
    /// Cipher to use [Shadowsocks]
    #[arg(value_parser = SHADOWSOCKS_CIPHERS, long)]
    cipher: Option<String>,
    /// The IP of the remote proxy server [Socks5 (Local & Remote proxy), Shadowsocks]
    #[arg(long)]
    ip: Option<IpAddr>,
    /// The port of the remote proxy server [Socks5 (Local & Remote proxy), Shadowsocks]
    #[arg(long)]
    port: Option<u16>,
    /// The port that the server on localhost is listening on [Socks5 (Local proxy)]
    #[arg(long)]
    local_port: Option<u16>,
    /// The transport protocol used by the remote proxy [Socks5 (Local proxy)]
    #[arg(long)]
    transport_protocol: Option<TransportProtocol>,
}
