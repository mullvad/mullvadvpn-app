use std::net::{IpAddr, SocketAddr};
use talpid_types::net::{
    proxy::{Shadowsocks, Socks5Local, Socks5Remote, SocksAuth, SHADOWSOCKS_CIPHERS},
    Endpoint, TransportProtocol,
};

use clap::Args;

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

impl From<Socks5LocalAdd> for Socks5Local {
    fn from(add: Socks5LocalAdd) -> Self {
        Self {
            remote_endpoint: Endpoint {
                address: SocketAddr::new(add.remote_ip, add.remote_port),
                protocol: add.transport_protocol,
            },
            local_port: add.local_port,
        }
    }
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

impl From<Socks5RemoteAdd> for Socks5Remote {
    fn from(add: Socks5RemoteAdd) -> Self {
        Self {
            endpoint: SocketAddr::new(add.remote_ip, add.remote_port),
            auth: add.authentication.map(|auth| SocksAuth {
                username: auth.username,
                password: auth.password,
            }),
        }
    }
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

impl From<ShadowsocksAdd> for Shadowsocks {
    fn from(add: ShadowsocksAdd) -> Self {
        Self {
            endpoint: SocketAddr::new(add.remote_ip, add.remote_port),
            password: add.password,
            cipher: add.cipher,
        }
    }
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
pub struct ProxyEditParams {
    /// Username for authentication [Socks5 (Remote proxy)]
    #[arg(long)]
    pub username: Option<String>,
    /// Password for authentication [Socks5 (Remote proxy), Shadowsocks]
    #[arg(long)]
    pub password: Option<String>,
    /// Cipher to use [Shadowsocks]
    #[arg(value_parser = SHADOWSOCKS_CIPHERS, long)]
    pub cipher: Option<String>,
    /// The IP of the remote proxy server [Socks5 (Local & Remote proxy), Shadowsocks]
    #[arg(long)]
    pub ip: Option<IpAddr>,
    /// The port of the remote proxy server [Socks5 (Local & Remote proxy), Shadowsocks]
    #[arg(long)]
    pub port: Option<u16>,
    /// The port that the server on localhost is listening on [Socks5 (Local proxy)]
    #[arg(long)]
    pub local_port: Option<u16>,
    /// The transport protocol used by the remote proxy [Socks5 (Local proxy)]
    #[arg(long)]
    pub transport_protocol: Option<TransportProtocol>,
}

impl ProxyEditParams {
    pub fn merge_socks_local(self, local: &Socks5Local) -> Socks5Local {
        let remote_ip = self.ip.unwrap_or(local.remote_endpoint.address.ip());
        let remote_port = self.port.unwrap_or(local.remote_endpoint.address.port());
        let local_port = self.local_port.unwrap_or(local.local_port);
        let remote_peer_transport_protocol = self
            .transport_protocol
            .unwrap_or(local.remote_endpoint.protocol);
        Socks5Local::new_with_transport_protocol(
            (remote_ip, remote_port),
            local_port,
            remote_peer_transport_protocol,
        )
    }

    pub fn merge_socks_remote(self, remote: &Socks5Remote) -> Socks5Remote {
        let ip = self.ip.unwrap_or(remote.endpoint.ip());
        let port = self.port.unwrap_or(remote.endpoint.port());
        match &remote.auth {
            None => Socks5Remote::new((ip, port)),
            Some(SocksAuth { username, password }) => {
                let username = self.username.unwrap_or(username.to_owned());
                let password = self.password.unwrap_or(password.to_owned());
                let auth = SocksAuth { username, password };
                Socks5Remote::new_with_authentication((ip, port), auth)
            }
        }
    }

    pub fn merge_shadowsocks(self, shadowsocks: &Shadowsocks) -> Shadowsocks {
        let ip = self.ip.unwrap_or(shadowsocks.endpoint.ip());
        let port = self.port.unwrap_or(shadowsocks.endpoint.port());
        let password = self.password.unwrap_or(shadowsocks.password.to_owned());
        let cipher = self.cipher.unwrap_or(shadowsocks.cipher.to_owned());
        Shadowsocks::new((ip, port), cipher, password)
    }
}

pub mod pp {
    use crate::print_option;
    use talpid_types::net::proxy::{CustomProxy, SocksAuth};

    pub struct CustomProxyFormatter<'a> {
        pub custom_proxy: &'a CustomProxy,
    }

    impl<'a> std::fmt::Display for CustomProxyFormatter<'a> {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self.custom_proxy {
                CustomProxy::Shadowsocks(shadowsocks) => {
                    print_option!("Protocol", format!("Shadowsocks [{}]", shadowsocks.cipher));
                    print_option!("Peer", shadowsocks.endpoint);
                    print_option!("Password", shadowsocks.password);
                    Ok(())
                }
                CustomProxy::Socks5Remote(remote) => {
                    print_option!("Protocol", "Socks5");
                    print_option!("Peer", remote.endpoint);
                    match &remote.auth {
                        Some(SocksAuth { username, password }) => {
                            print_option!("Username", username);
                            print_option!("Password", password);
                        }
                        None => (),
                    }
                    Ok(())
                }
                CustomProxy::Socks5Local(local) => {
                    print_option!("Protocol", "Socks5 (local)");
                    print_option!(
                        "Peer",
                        format!(
                            "{}/{}",
                            local.remote_endpoint.address, local.remote_endpoint.protocol
                        )
                    );
                    print_option!("Local port", local.local_port);
                    Ok(())
                }
            }
        }
    }
}
