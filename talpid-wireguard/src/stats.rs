use std::fmt;
use std::time::{Duration, SystemTime};

/// Contains bytes sent and received through a tunnel
#[derive(Default, PartialEq, Eq, Clone)]
pub struct Stats {
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub last_handshake_time: Option<SystemTime>,
    // Optional DAITA stats
    // Currently only available for GotaTun
    pub daita: Option<DaitaStats>,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct DaitaStats {
    /// Extra bytes added due to constant-size padding of data packets
    pub tx_padding_bytes: u64,

    /// Bytes of standalone padding packets transmitted
    pub tx_padding_packet_bytes: u64,

    /// Total extra bytes removed due to constant-size padding of data packets
    pub rx_padding_bytes: u64,

    /// Bytes of standalone padding packets received
    pub rx_padding_packet_bytes: u64,
}

impl fmt::Debug for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stats = StatsDebug {
            now: SystemTime::now(),
            stats: self,
        };
        fmt::Debug::fmt(&stats, f)
    }
}

struct StatsDebug<'a> {
    pub now: SystemTime,
    pub stats: &'a Stats,
}

impl fmt::Debug for StatsDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("Stats");

        dbg.field("tx_bytes", &self.stats.tx_bytes)
            .field("rx_bytes", &self.stats.rx_bytes);

        if let Some(last_handshake) = self.stats.last_handshake_time {
            let time_since_handshake = self
                .now
                .duration_since(last_handshake)
                .unwrap_or(Duration::ZERO);

            dbg.field(
                "last_handshake",
                &format_args!("\"{} ms ago\"", time_since_handshake.as_millis()),
            );
        } else {
            dbg.field("last_handshake", &"no handshake");
        }

        dbg.field("daita", &self.stats.daita);

        dbg.finish()
    }
}

/// A map from peer pubkeys to peer stats.
pub type StatsMap = std::collections::HashMap<[u8; 32], Stats>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_stats_debug() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(60);

        let stats = Stats {
            tx_bytes: 100,
            rx_bytes: 100,
            last_handshake_time: Some(SystemTime::UNIX_EPOCH),
            daita: None,
        };

        insta::assert_debug_snapshot!(StatsDebug { now, stats: &stats });
    }
}
