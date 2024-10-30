use ffi::TunnelObfuscatorProtocol;
use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::task::JoinHandle;
use tunnel_obfuscation::{
    create_obfuscator, shadowsocks, udp2tcp, Settings as ObfuscationSettings,
};

mod ffi;

use crate::mullvad_ios_runtime;

pub struct TunnelObfuscatorRuntime {
    settings: ObfuscationSettings,
}

impl TunnelObfuscatorRuntime {
    pub fn new(
        peer: SocketAddr,
        obfuscation_protocol: TunnelObfuscatorProtocol,
    ) -> io::Result<Self> {
        let settings: ObfuscationSettings = match obfuscation_protocol {
            TunnelObfuscatorProtocol::UdpOverTcp => {
                ObfuscationSettings::Udp2Tcp(udp2tcp::Settings { peer })
            }
            TunnelObfuscatorProtocol::Shadowsocks => {
                ObfuscationSettings::Shadowsocks(shadowsocks::Settings {
                    shadowsocks_endpoint: peer,
                    wireguard_endpoint: SocketAddr::from((Ipv4Addr::LOCALHOST, 51820)),
                })
            }
        };

        Ok(Self { settings })
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
