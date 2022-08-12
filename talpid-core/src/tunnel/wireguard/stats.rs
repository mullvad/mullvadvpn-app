#[cfg(target_os = "linux")]
use super::wireguard_kernel::wg_message::{DeviceMessage, DeviceNla, PeerNla};

#[derive(err_derive::Error, Debug, PartialEq)]
pub enum Error {
    #[error(display = "Failed to parse peer pubkey from string \"_0\"")]
    PubKeyParse(String, #[error(source)] hex::FromHexError),

    #[error(display = "Failed to parse integer from string \"_0\"")]
    IntParse(String, #[error(source)] std::num::ParseIntError),

    #[error(display = "Device no longer exists")]
    NoTunnelDevice,

    #[error(display = "Failed to obtain tunnel config")]
    NoTunnelConfig,
}

/// Contains bytes sent and received through a tunnel
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Stats {
    pub tx_bytes: u64,
    pub rx_bytes: u64,
}

/// A map from peer pubkeys to peer stats.
pub type StatsMap = std::collections::HashMap<[u8; 32], Stats>;

impl Stats {
    pub fn parse_config_str(config: &str) -> Result<StatsMap, Error> {
        let mut map = StatsMap::new();

        let mut peer = None;
        let mut tx_bytes = None;
        let mut rx_bytes = None;

        // parts iterates over keys and values
        let parts = config.split('\n').filter_map(|line| {
            let mut pair = line.split('=');
            let key = pair.next()?;
            let value = pair.next()?;
            Some((key, value))
        });

        for (key, value) in parts {
            match key {
                "public_key" => {
                    let mut buffer = [0u8; 32];
                    hex::decode_to_slice(value, &mut buffer)
                        .map_err(|err| Error::PubKeyParse(value.to_string(), err))?;
                    peer = Some(buffer);
                    tx_bytes = None;
                    rx_bytes = None;
                }
                "rx_bytes" => {
                    rx_bytes = Some(
                        value
                            .trim()
                            .parse()
                            .map_err(|err| Error::IntParse(value.to_string(), err))?,
                    );
                }
                "tx_bytes" => {
                    tx_bytes = Some(
                        value
                            .trim()
                            .parse()
                            .map_err(|err| Error::IntParse(value.to_string(), err))?,
                    );
                }

                _ => continue,
            }

            if let (Some(peer_val), Some(tx_bytes_val), Some(rx_bytes_val)) =
                (peer, tx_bytes, rx_bytes)
            {
                map.insert(
                    peer_val,
                    Self {
                        tx_bytes: tx_bytes_val,
                        rx_bytes: rx_bytes_val,
                    },
                );
                peer = None;
                tx_bytes = None;
                rx_bytes = None;
            }
        }
        Ok(map)
    }

    #[cfg(target_os = "linux")]
    pub fn parse_device_message(message: &DeviceMessage) -> StatsMap {
        let mut map = StatsMap::new();

        for nla in &message.nlas {
            if let DeviceNla::Peers(peers) = nla {
                for msg in peers {
                    let mut tx_bytes = 0;
                    let mut rx_bytes = 0;
                    let mut pub_key = None;

                    for nla in &msg.0 {
                        match nla {
                            PeerNla::TxBytes(bytes) => tx_bytes = *bytes,
                            PeerNla::RxBytes(bytes) => rx_bytes = *bytes,
                            PeerNla::PublicKey(key) => pub_key = Some(*key),
                            _ => continue,
                        }
                    }
                    if let Some(key) = pub_key {
                        map.insert(key, Stats { tx_bytes, rx_bytes });
                    }
                }
            }
        }

        map
    }
}

#[cfg(test)]
mod test {
    use super::{Error, Stats};

    #[test]
    fn test_parsing() {
        let valid_input = "private_key=0000000000000000000000000000000000000000000000000000000000000000\npublic_key=0000000000000000000000000000000000000000000000000000000000000000\npreshared_key=0000000000000000000000000000000000000000000000000000000000000000\nprotocol_version=1\nendpoint=000.000.000.000:00000\nlast_handshake_time_sec=1578420649\nlast_handshake_time_nsec=369416131\ntx_bytes=2740\nrx_bytes=2396\npersistent_keepalive_interval=0\nallowed_ip=0.0.0.0/0\n";
        let pubkey = [0u8; 32];

        let stats = Stats::parse_config_str(valid_input).expect("Failed to parse valid input");
        assert_eq!(stats.len(), 1);
        let actual_keys: Vec<[u8; 32]> = stats.keys().cloned().collect();
        assert_eq!(actual_keys, [pubkey]);
        assert_eq!(stats[&pubkey].rx_bytes, 2396);
        assert_eq!(stats[&pubkey].tx_bytes, 2740);
    }

    #[test]
    fn test_parsing_invalid_input() {
        let invalid_input = "private_key=0000000000000000000000000000000000000000000000000000000000000000\npublic_key=0000000000000000000000000000000000000000000000000000000000000000\npreshared_key=0000000000000000000000000000000000000000000000000000000000000000\nprotocol_version=1\nendpoint=000.000.000.000:00000\nlast_handshake_time_sec=1578420649\nlast_handshake_time_nsec=369416131\ntx_bytes=27error40\npersistent_keepalive_interval=0\nallowed_ip=0.0.0.0/0\n";
        let invalid_str = "27error40".to_string();
        let int_err = invalid_str.parse::<u64>().unwrap_err();

        assert_eq!(
            Stats::parse_config_str(invalid_input),
            Err(Error::IntParse(invalid_str, int_err))
        );
    }
}
