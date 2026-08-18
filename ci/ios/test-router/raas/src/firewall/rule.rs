use crate::web::routes::TransportProtocol;

use ipnetwork::IpNetwork;
use std::collections::BTreeSet;

#[derive(Clone, serde::Serialize)]
pub enum BlockRule {
    Host {
        endpoints: Endpoints,
        protocols: BTreeSet<TransportProtocol>,
    },
    WireGuard {
        endpoints: Endpoints,
    },
}

#[derive(Clone, Copy, serde::Serialize)]
pub struct Endpoints {
    pub src: IpNetwork,
    pub dst: IpNetwork,
    /// Normally a packet sent to `dst` would match the block rule, but this option inverts that
    /// so that any packet *not* sent to `dst` will match the block rule.
    pub invert_dst: bool,
}
