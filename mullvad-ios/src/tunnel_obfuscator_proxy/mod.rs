use std::{io, net::SocketAddr};
use tokio::task::JoinHandle;
use tunnel_obfuscation::{create_obfuscator, Settings as ObfuscationSettings, udp2tcp::Settings};

mod ffi;

use crate::mullvad_ios_runtime;

pub struct TunnelObfuscatorRuntime {
    settings: ObfuscationSettings,
}

impl TunnelObfuscatorRuntime {
    pub fn new(peer: SocketAddr) -> io::Result<Self> {
        let settings = ObfuscationSettings::Udp2Tcp(Settings { peer });

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
