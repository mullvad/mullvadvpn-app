use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::wg_message::{DeviceMessage, DeviceNla, PeerNla};
use crate::stats::{Stats, StatsMap};

impl Stats {
    pub fn parse_device_message(message: &DeviceMessage) -> StatsMap {
        let mut map = StatsMap::new();

        for nla in &message.nlas {
            if let DeviceNla::Peers(peers) = nla {
                for msg in peers {
                    let mut tx_bytes = 0;
                    let mut rx_bytes = 0;
                    let mut last_handshake_time = None;
                    let mut pub_key = None;

                    for nla in &msg.0 {
                        match nla {
                            PeerNla::TxBytes(bytes) => tx_bytes = *bytes,
                            PeerNla::RxBytes(bytes) => rx_bytes = *bytes,
                            PeerNla::LastHandshakeTime(time) => {
                                last_handshake_time = || -> Option<SystemTime> {
                                    // handshake_{sec,nsec} are relative to UNIX_EPOCH
                                    // https://www.wireguard.com/xplatform/
                                    Some(
                                        UNIX_EPOCH
                                            + Duration::new(
                                                time.tv_sec().try_into().ok()?,
                                                time.tv_nsec().try_into().ok()?,
                                            ),
                                    )
                                }();
                            }
                            PeerNla::PublicKey(key) => pub_key = Some(*key),
                            _ => continue,
                        }
                    }
                    if let Some(key) = pub_key {
                        map.insert(
                            key,
                            Stats {
                                tx_bytes,
                                rx_bytes,
                                last_handshake_time,
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }

        map
    }
}
