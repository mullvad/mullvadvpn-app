use std::net::IpAddr;

pub mod am_i_mullvad;
pub mod traceroute;
mod util;

#[derive(Clone, Debug)]
pub enum LeakStatus {
    NoLeak,
    LeakDetected(LeakInfo),
}

/// Details about how a leak happened
#[derive(Clone, Debug)]
pub enum LeakInfo {
    /// Managed to reach another network node on the physical interface, bypassing firewall rules.
    NodeReachableOnInterface {
        reachable_nodes: Vec<IpAddr>,
        interface: String,
    },

    /// Queried a <https://am.i.mullvad.net>, and was not mullvad.
    AmIMullvad { ip: IpAddr },
}
