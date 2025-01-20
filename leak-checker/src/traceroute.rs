use std::{net::IpAddr, ops::Range, time::Duration};

use crate::{Interface, LeakStatus};

/// Traceroute implementation for windows.
#[cfg(target_os = "windows")]
mod windows;

/// Traceroute implementation for unix.
#[cfg(unix)]
mod unix;

#[derive(Clone, clap::Args)]
pub struct TracerouteOpt {
    /// Try to bind to a specific interface
    #[clap(short, long)]
    pub interface: Interface,

    /// Destination IP of the probe packets
    #[clap(short, long)]
    pub destination: IpAddr,

    /// Avoid sending UDP probe packets to this port.
    #[clap(long, conflicts_with = "icmp")]
    #[cfg(unix)]
    pub exclude_port: Option<u16>,

    /// Send UDP probe packets only to this port, instead of the default ports.
    #[clap(long, conflicts_with = "icmp")]
    #[cfg(unix)]
    pub port: Option<u16>,

    /// Use ICMP-Echo for the probe packets instead of UDP.
    #[clap(long)]
    #[cfg(unix)]
    pub icmp: bool,
}

/// Timeout of the leak test as a whole. Should be more than [SEND_TIMEOUT] + [RECV_TIMEOUT].
const LEAK_TIMEOUT: Duration = Duration::from_secs(5);

/// Timeout of sending probe packets
const SEND_TIMEOUT: Duration = Duration::from_secs(1);

/// Timeout of receiving additional probe packets after the first one
const RECV_GRACE_TIME: Duration = Duration::from_millis(220);

/// Time in-between send of each probe packet.
const PROBE_INTERVAL: Duration = Duration::from_millis(100);

/// Range of TTL values for the probe packets.
const DEFAULT_TTL_RANGE: Range<u16> = 1..6;

/// [try_run_leak_test], but on an error, assume we aren't leaking.
pub async fn run_leak_test(opt: &TracerouteOpt) -> LeakStatus {
    try_run_leak_test(opt)
        .await
        .inspect_err(|e| log::debug!("Leak test errored, assuming no leak. {e:?}"))
        .unwrap_or(LeakStatus::NoLeak)
}

/// Run a traceroute-based leak test.
///
/// This test will try to create a socket and bind it to `interface`. Then it will send either UDP
/// or ICMP Echo packets to `destination` with very low TTL values. If any network nodes between
/// this one and `destination` see a packet with a TTL value of 0, they will _probably_ return an
/// ICMP/TimeExceeded response.
///
/// If we receive the response, we know the outgoing packet was NOT blocked by the firewall, and
/// therefore we are leaking. Since we set the TTL very low, this also means that in the event of a
/// leak, the packet will _probably_ not make it out of the users local network, e.g. the local
/// router will probably be the first node that gives a reply. Since the packet should not actually
/// reach `destination`, this testing method is resistant to being fingerprinted or censored.
///
/// This test needs a raw socket to be able to listen for the ICMP responses, therefore it requires
/// root/admin priviliges.
pub async fn try_run_leak_test(opt: &TracerouteOpt) -> anyhow::Result<LeakStatus> {
    #[cfg(unix)]
    return {
        #[cfg(target_os = "android")]
        type Impl = unix::android::TracerouteAndroid;
        #[cfg(target_os = "linux")]
        type Impl = unix::linux::TracerouteLinux;
        #[cfg(target_os = "macos")]
        type Impl = unix::macos::TracerouteMacos;

        unix::try_run_leak_test::<Impl>(opt).await
    };

    #[cfg(target_os = "windows")]
    return windows::traceroute_using_ping(opt).await;
}

/// IP version, v4 or v6, with some associated data.
#[derive(Clone, Copy)]
enum Ip<V4 = (), V6 = ()> {
    V4(V4),
    V6(V6),
}
