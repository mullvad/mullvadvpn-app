/// Contains bytes sent and received through a tunnel
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Stats {
    pub tx_bytes: u64,
    pub rx_bytes: u64,
}

/// A map from peer pubkeys to peer stats.
pub type StatsMap = std::collections::HashMap<[u8; 32], Stats>;
