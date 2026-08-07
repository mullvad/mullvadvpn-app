//! Routes the traffic arriving on the tunnel device out through real OS sockets.
//!
//! One task reads the device and demultiplexes by protocol; one task writes everything back. The
//! read loop never waits on an upstream socket, so a stalled connection cannot hold up the rest.

use std::sync::Arc;

use arc_swap::ArcSwap;

use super::{RuleSet, TunnelDevices};

mod filter;
mod tcp;
mod udp;

pub struct Router {
}

impl Router {
    pub fn spawn(devices: TunnelDevices, rules: Arc<ArcSwap<RuleSet>>) {
        let TunnelDevices { sink, source } = devices;
        let sink = Arc::new(sink);
        let source = Arc::new(source);
        let sink_clone = sink.clone();
        let source_clone = source.clone();
        let rules_clone = rules.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; u16::MAX.into()];

            while let Ok(bytes_received) = sink.recv(&mut buf).await {
                let packet = &buf[..bytes_received];
                if filter::blocked(&rules.load(), packet) {
                    continue;
                }
                if let Err(err) = source_clone.send(packet).await {
                    log::error!("Failed to write to source utun: {err}");
                }
            }

        });

        tokio::spawn(async move {
            let mut buf = vec![0u8; u16::MAX.into()];
            while let Ok(bytes_received) = source.recv(&mut buf).await {
                let packet = &buf[..bytes_received];
                if filter::blocked(&rules_clone.load(), packet) {
                    continue;
                }
                if let Err(err) = sink_clone.send(packet).await {
                    log::error!("Failed to write to sink utun: {err}");
                }
            }

        });
    }
}
