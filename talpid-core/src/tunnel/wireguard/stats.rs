#[derive(err_derive::Error, Debug, PartialEq)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to parse integer from string \"_0\"")]
    IntParseError(String, #[error(source)] std::num::ParseIntError),

    #[error(display = "Config key not found")]
    KeyNotFoundError,
}

/// Contains bytes sent and received through a tunnel
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct Stats {
    pub tx_bytes: u64,
    pub rx_bytes: u64,
}

impl Stats {
    pub fn parse_config_str(config: &str) -> Result<Self, Error> {
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
                "rx_bytes" => {
                    rx_bytes = Some(
                        value
                            .trim()
                            .parse()
                            .map_err(|err| Error::IntParseError(value.to_string(), err))?,
                    );
                }
                "tx_bytes" => {
                    tx_bytes = Some(
                        value
                            .trim()
                            .parse()
                            .map_err(|err| Error::IntParseError(value.to_string(), err))?,
                    );
                }

                _ => continue,
            }
        }

        match (tx_bytes, rx_bytes) {
            (Some(tx_bytes), Some(rx_bytes)) => Ok(Self { tx_bytes, rx_bytes }),
            _ => Err(Error::KeyNotFoundError),
        }
    }
}


#[cfg(test)]
mod test {
    use super::{Error, Stats};

    #[test]
    fn test_parsing() {
        let valid_input = "private_key=0000000000000000000000000000000000000000000000000000000000000000\npublic_key=0000000000000000000000000000000000000000000000000000000000000000\npreshared_key=0000000000000000000000000000000000000000000000000000000000000000\nprotocol_version=1\nendpoint=000.000.000.000:00000\nlast_handshake_time_sec=1578420649\nlast_handshake_time_nsec=369416131\ntx_bytes=2740\nrx_bytes=2396\npersistent_keepalive_interval=0\nallowed_ip=0.0.0.0/0\n";

        let stats = Stats::parse_config_str(valid_input).expect("Failed to parse valid input");
        assert_eq!(stats.rx_bytes, 2396);
        assert_eq!(stats.tx_bytes, 2740);
    }

    #[test]
    fn test_parsing_invalid_input() {
        let invalid_input = "private_key=0000000000000000000000000000000000000000000000000000000000000000\npublic_key=0000000000000000000000000000000000000000000000000000000000000000\npreshared_key=0000000000000000000000000000000000000000000000000000000000000000\nprotocol_version=1\nendpoint=000.000.000.000:00000\nlast_handshake_time_sec=1578420649\nlast_handshake_time_nsec=369416131\ntx_bytes=27error40\npersistent_keepalive_interval=0\nallowed_ip=0.0.0.0/0\n";
        let invalid_str = "27error40".to_string();
        let int_err = invalid_str.parse::<u64>().unwrap_err();

        assert_eq!(
            Stats::parse_config_str(invalid_input),
            Err(Error::IntParseError(invalid_str, int_err))
        );
    }

    #[test]
    fn test_parsing_missing_keys() {
        let invalid_input = "private_key=0000000000000000000000000000000000000000000000000000000000000000\npublic_key=0000000000000000000000000000000000000000000000000000000000000000\npreshared_key=0000000000000000000000000000000000000000000000000000000000000000\nprotocol_version=1\nendpoint=000.000.000.000:00000\nlast_handshake_time_sec=1578420649\nlast_handshake_time_nsec=369416131\ntx_bytes=2740\npersistent_keepalive_interval=0\nallowed_ip=0.0.0.0/0\n";
        assert_eq!(
            Stats::parse_config_str(invalid_input),
            Err(Error::KeyNotFoundError)
        );
    }
}
