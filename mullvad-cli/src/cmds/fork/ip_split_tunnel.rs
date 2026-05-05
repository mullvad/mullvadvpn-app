use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;
use std::{net::Ipv4Addr, str::FromStr};

const CGNAT_RANGE: &str = "100.64.0.0/10";
const TENANT_RANGE_SAMPLE: &str = "10.0.0.0/16";
const TEMPLATE_RANGES: &[&str] = &[CGNAT_RANGE];

#[derive(Subcommand, Debug)]
pub enum IpSplitTunnel {
    /// List all IPv4 ranges that are excluded from the tunnel
    List,
    /// Add an IPv4 address or CIDR range to exclude from the tunnel
    Add {
        #[arg(value_parser = parse_ipv4_range_arg)]
        range: String,
    },
    /// Stop excluding an IPv4 address or CIDR range from the tunnel
    Delete {
        #[arg(value_parser = parse_ipv4_range_arg)]
        range: String,
    },
    /// Stop excluding all IPv4 ranges from the tunnel
    Clear,
    /// Toggle IP split tunneling on or off
    Toggle,
    /// Add known Tailscale and NetBird IPv4 overlay ranges
    ApplyTemplates,
    /// Check whether configured IPv4 ranges would route outside the tunnel
    Check,
}

impl IpSplitTunnel {
    pub async fn handle(self) -> Result<()> {
        match self {
            IpSplitTunnel::List => {
                let ranges = MullvadProxyClient::new()
                    .await?
                    .get_split_tunnel_ip_ranges()
                    .await?;

                println!("Excluded IPv4 ranges:");
                for range in &ranges {
                    println!("{range}");
                }

                Ok(())
            }
            IpSplitTunnel::Add { range } => {
                MullvadProxyClient::new()
                    .await?
                    .add_split_tunnel_ip_range(range)
                    .await?;
                println!("Excluding IPv4 range");
                Ok(())
            }
            IpSplitTunnel::Delete { range } => {
                MullvadProxyClient::new()
                    .await?
                    .remove_split_tunnel_ip_range(range)
                    .await?;
                println!("Stopped excluding IPv4 range");
                Ok(())
            }
            IpSplitTunnel::Clear => {
                MullvadProxyClient::new()
                    .await?
                    .clear_split_tunnel_ip_ranges()
                    .await?;
                println!("Stopped excluding all IPv4 ranges");
                Ok(())
            }
            IpSplitTunnel::Toggle => {
                let enabled = MullvadProxyClient::new()
                    .await?
                    .toggle_split_tunnel_ip()
                    .await?;
                if enabled {
                    println!("IP split tunneling enabled");
                } else {
                    println!("IP split tunneling disabled");
                }
                Ok(())
            }
            IpSplitTunnel::ApplyTemplates => {
                let mut client = MullvadProxyClient::new().await?;
                for range in TEMPLATE_RANGES {
                    client.add_split_tunnel_ip_range((*range).to_owned()).await?;
                }
                println!("Applied IP split-tunnel templates:");
                println!("CGNAT range (Tailscale/NetBird): {CGNAT_RANGE}");
                println!(
                    "Additional tenant ranges like {TENANT_RANGE_SAMPLE} are accepted and normalized"
                );
                Ok(())
            }
            IpSplitTunnel::Check => {
                let checks = MullvadProxyClient::new()
                    .await?
                    .check_split_tunnel_ip_ranges()
                    .await?;

                if checks.is_empty() {
                    println!("No excluded IPv4 ranges configured");
                    return Ok(());
                }

                println!("IPv4 split-tunnel checks:");
                for check in checks {
                    let status = if check.bypasses_tunnel {
                        "outside tunnel"
                    } else {
                        "inside tunnel"
                    };
                    let interface = check
                        .interface
                        .as_deref()
                        .map(|interface| format!(" via {interface}"))
                        .unwrap_or_default();
                    let error = check
                        .error
                        .as_deref()
                        .map(|error| format!(" ({error})"))
                        .unwrap_or_default();
                    println!(
                        "{} sample {}: {status}{interface}{error}",
                        check.range, check.sample_ip
                    );
                }
                Ok(())
            }
        }
    }
}

fn parse_ipv4_range_arg(input: &str) -> std::result::Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("expected an IPv4 address or CIDR range".to_owned());
    }

    if let Some((address, prefix)) = trimmed.split_once('/') {
        let address = Ipv4Addr::from_str(address)
            .map_err(|_| format!("'{trimmed}' is not a valid IPv4 CIDR range"))?;
        let prefix = prefix
            .parse::<u8>()
            .map_err(|_| format!("'{trimmed}' has an invalid CIDR prefix"))?;
        if prefix > 32 {
            return Err(format!("'{trimmed}' has an invalid CIDR prefix"));
        }

        let mask = if prefix == 0 {
            0
        } else {
            u32::MAX << (32 - prefix)
        };
        let network = Ipv4Addr::from(u32::from(address) & mask);
        return Ok(format!("{network}/{prefix}"));
    }

    let address = Ipv4Addr::from_str(trimmed)
        .map_err(|_| format!("'{trimmed}' is not a valid IPv4 address or CIDR range"))?;
    Ok(format!("{address}/32"))
}

#[cfg(test)]
mod tests {
    use super::parse_ipv4_range_arg;

    #[test]
    fn parses_single_ipv4_address() {
        assert_eq!(parse_ipv4_range_arg("100.64.0.1").unwrap(), "100.64.0.1/32");
    }

    #[test]
    fn canonicalizes_cidr() {
        assert_eq!(
            parse_ipv4_range_arg("100.64.12.34/10").unwrap(),
            "100.64.0.0/10"
        );
    }

    #[test]
    fn canonicalizes_netbird_tenant_cidr() {
        assert_eq!(
            parse_ipv4_range_arg("100.114.4.17/16").unwrap(),
            "100.114.0.0/16"
        );
    }

    #[test]
    fn rejects_invalid_prefix() {
        assert!(parse_ipv4_range_arg("100.64.0.0/33").is_err());
    }

    #[test]
    fn rejects_ipv6() {
        assert!(parse_ipv4_range_arg("fd7a:115c:a1e0::/48").is_err());
    }
}
