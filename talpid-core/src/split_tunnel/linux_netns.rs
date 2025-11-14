use std::net::Ipv4Addr;

use futures::{FutureExt, select};
use nullvad::{do_in_namespace_async, open_namespace_file};
use tokio::{net::UdpSocket, sync::mpsc, task::JoinHandle};

// TODO: "mark"? mark them for what?
// TODO: duplicated from cgroups implementation
/// Value used to mark packets and associated connections.
/// This should be an arbitrary but unique integer.
pub const MARK: i32 = 0xf41;

/// Errors related to split tunneling.
#[derive(thiserror::Error, Debug)]
#[error("Split-tunneling error")]
pub struct Error(#[source] nullvad::Error);

/// Manages PIDs in the Linux Cgroup excluded from the VPN tunnel.
pub struct PidManager {
    result: nullvad::Result<()>,
    dns_proxy: JoinHandle<()>,
}

impl PidManager {
    /// Set up network namespace used for split tunneling.
    pub async fn new() -> Self {
        // if the namespace already exists, clean it up first.
        let _ = nullvad::destroy_namespace();
        let _ = nullvad::nft::remove_nft_rules();

        Self {
            result: nullvad::up().await,
            dns_proxy: tokio::spawn(dns_proxy()),
        }
    }

    /// Add a PID to the network namespace to have it excluded from the tunnel.
    pub fn add(&self, pid: i32) -> Result<(), Error> {
        log::warn!("split tunneling not implemented");
        Ok(())
    }

    /// Remove a PID from the network namespace to have it included in the tunnel.
    pub fn remove(&self, pid: i32) -> Result<(), Error> {
        log::warn!("split tunneling not implemented");
        Ok(())
    }

    /// Return a list of all PIDs currently in the network namespace and excluded from the tunnel.
    pub fn list(&self) -> Result<Vec<i32>, Error> {
        log::warn!("split tunneling not implemented");
        Ok(vec![])
    }

    /// Removes all PIDs from the network namespace.
    pub fn clear(&self) -> Result<(), Error> {
        log::warn!("split tunneling not implemented");
        Ok(())
    }

    /// Return whether it is enabled
    pub fn is_enabled(&self) -> bool {
        self.result.is_ok()
    }
}

impl Drop for PidManager {
    fn drop(&mut self) {
        self.dns_proxy.abort();

        log::info!("Removing split-tunneling network namespace");
        if let Err(e) = nullvad::destroy_namespace() {
            log::error!("{e:#?}");
        }
        if let Err(e) = nullvad::nft::remove_nft_rules() {
            log::error!("{e:#?}");
        }
    }
}

/// A very crude and slightly broken dns proxy that forward
/// between the network namespace and systemd-resolved
// TODO: remove or replace with hickory
async fn dns_proxy() {
    log::warn!("Hacky dns proxy running!");

    let (to_ns, mut from_default) = mpsc::channel::<Vec<u8>>(100);
    let (to_default, mut from_ns) = mpsc::channel::<Vec<u8>>(100);

    let resolved_addr = Ipv4Addr::new(127, 0, 0, 53);

    let default_task = async move {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0u16)).await.unwrap();
        socket.connect((resolved_addr, 53u16)).await.unwrap();

        let mut buf = vec![0u8; 2048];
        loop {
            select! {
                result = socket.recv(&mut buf[..]).fuse() => {
                    let n = result.unwrap();
                    let packet = buf[..n].to_vec();
                    if to_ns.send(packet).await.is_err() { break };
                }
                packet = from_ns.recv().fuse() => {
                    let Some(packet) = packet else { break };
                    socket.send(&packet[..]).await.unwrap();
                }
            }
        }
    };

    let ns_file = open_namespace_file().await.unwrap().unwrap();
    let ns_task = do_in_namespace_async(ns_file, async move || {
        let socket = UdpSocket::bind((resolved_addr, 53u16)).await.unwrap();
        let mut buf = vec![0u8; 2048];
        let mut last_addr = None;
        loop {
            select! {
                result = socket.recv_from(&mut buf[..]).fuse() => {
                    let (n, from) = result.unwrap();
                    let packet = buf[..n].to_vec();
                    if to_default.send(packet).await.is_err() { break };
                    last_addr = Some(from); // HACK: this is racey
                }
                packet = from_default.recv().fuse() => {
                    let Some(packet) = packet else { break };
                    let Some(addr) = last_addr else { continue };
                    let _ = socket.send_to(&packet[..], addr).await;
                }
            }
        }
    });

    select! {
        _ = default_task.fuse() => {}
        _ = ns_task.fuse() => {}
    }
}
