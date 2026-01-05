use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};
use tokio::task::JoinHandle;
use tunnel_obfuscation::{
    Settings as ObfuscationSettings, create_obfuscator, quic, shadowsocks, udp2tcp,
};

mod ffi;

use crate::mullvad_ios_runtime;

pub struct TunnelObfuscatorRuntime {
    settings: ObfuscationSettings,
}

impl TunnelObfuscatorRuntime {
    pub fn new_udp2tcp(peer: SocketAddr) -> Self {
        let settings = ObfuscationSettings::Udp2Tcp(udp2tcp::Settings { peer });
        Self { settings }
    }

    pub fn new_shadowsocks(peer: SocketAddr) -> Self {
        let wireguard_endpoint = if peer.is_ipv4() {
            SocketAddr::from((Ipv4Addr::LOCALHOST, 51820))
        } else {
            SocketAddr::from((Ipv6Addr::LOCALHOST, 51820))
        };
        let settings = ObfuscationSettings::Shadowsocks(shadowsocks::Settings {
            shadowsocks_endpoint: peer,
            wireguard_endpoint,
        });
        Self { settings }
    }

    pub fn new_quic(peer: SocketAddr, hostname: String, token: String) -> Self {
        let wireguard_endpoint = if peer.is_ipv4() {
            SocketAddr::from((Ipv4Addr::LOCALHOST, 51820))
        } else {
            SocketAddr::from((Ipv6Addr::LOCALHOST, 51820))
        };
        let token: quic::AuthToken = token.parse().unwrap();
        let quic = quic::Settings::new(peer, hostname, token, wireguard_endpoint);
        let settings = ObfuscationSettings::Quic(quic);
        Self { settings }
    }

    pub fn run(self) -> io::Result<(SocketAddr, TunnelObfuscatorHandle)> {
        let runtime = mullvad_ios_runtime().map_err(io::Error::other)?;

        let obfuscator = runtime.block_on(async move {
            create_obfuscator(&self.settings)
                .await
                .map_err(io::Error::other)
        })?;

        let endpoint = obfuscator.endpoint();
        let join_handle = runtime.spawn(async move {
            let _ = obfuscator.run().await;
        });

        Ok((
            endpoint,
            TunnelObfuscatorHandle {
                obfuscator_abort_handle: join_handle,
            },
        ))
    }
}

pub struct TunnelObfuscatorHandle {
    obfuscator_abort_handle: JoinHandle<()>,
}

impl TunnelObfuscatorHandle {
    pub fn stop(self) {
        self.obfuscator_abort_handle.abort();
    }
}
