#![cfg(target_os = "ios")]

use std::{io, net::SocketAddr};
use tokio::sync::oneshot;
use tunnel_obfuscation::{create_obfuscator, Settings as ObfuscationSettings, Udp2TcpSettings};

mod ffi;
pub use ffi::{start_tunnel_obfuscator_proxy, stop_tunnel_obfuscator_proxy, ProxyHandle};

pub struct TunnelObfuscatorRuntime {
    runtime: tokio::runtime::Runtime,
    settings: ObfuscationSettings,
}

impl TunnelObfuscatorRuntime {
    pub fn new(peer: SocketAddr) -> io::Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let settings = ObfuscationSettings::Udp2Tcp(Udp2TcpSettings { peer });

        Ok(Self { runtime, settings })
    }

    pub fn run(self) -> io::Result<(SocketAddr, TunnelObfuscatorHandle)> {
        let (tx, rx) = oneshot::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (startup_tx, startup_rx) = oneshot::channel();
        std::thread::spawn(move || {
            self.run_service_inner(rx, startup_tx);
        });

        match startup_rx.blocking_recv() {
            Ok(Ok(endpoint)) => Ok((endpoint, TunnelObfuscatorHandle { tx })),
            Ok(Err(err)) => {
                let _ = tx.send(shutdown_tx);
                let _ = shutdown_rx.blocking_recv();
                Err(io::Error::new(io::ErrorKind::Other, err))
            }
            Err(_) => {
                let _ = tx.send(shutdown_tx);
                let _ = shutdown_rx.blocking_recv();
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Tokio runtime crashed",
                ))
            }
        }
    }

    fn run_service_inner(
        self,
        shutdown_rx: oneshot::Receiver<oneshot::Sender<()>>,
        startup_done_tx: oneshot::Sender<io::Result<SocketAddr>>,
    ) {
        let Self {
            settings, runtime, ..
        } = self;

        std::thread::spawn(move || {
            runtime.spawn(async move {
                match create_obfuscator(&settings).await {
                    Ok(obfuscator) => {
                        let endpoint = obfuscator.endpoint();
                        let _ = startup_done_tx.send(Ok(endpoint));
                        let _ = obfuscator.run().await;
                    }
                    Err(err) => {
                        let _ =
                            startup_done_tx.send(Err(io::Error::new(io::ErrorKind::Other, err)));
                    }
                }
            });
            if let Ok(shutdown_tx) = runtime.block_on(shutdown_rx) {
                std::mem::drop(runtime);
                let _ = shutdown_tx.send(());
            }
        });
    }
}

pub struct TunnelObfuscatorHandle {
    tx: oneshot::Sender<oneshot::Sender<()>>,
}

impl TunnelObfuscatorHandle {
    pub fn stop(self) {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let _ = self.tx.send(shutdown_tx);
        let _ = shutdown_rx.blocking_recv();
    }
}
