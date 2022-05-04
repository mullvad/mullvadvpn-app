use mullvad_types::settings::{DnsOptions, DnsState};
use std::net::{IpAddr, Ipv4Addr};

/// When we want to block certain contents with the help of DNS server side,
/// we compute the resolver IP to use based on these constants. The last
/// byte can be ORed together to combine multiple block lists.
const DNS_BLOCKING_IP_BASE: Ipv4Addr = Ipv4Addr::new(100, 64, 0, 0);
const DNS_AD_BLOCKING_IP_BIT: u8 = 1 << 0; // 0b00000001
const DNS_TRACKER_BLOCKING_IP_BIT: u8 = 1 << 1; // 0b00000010
const DNS_MALWARE_BLOCKING_IP_BIT: u8 = 1 << 2; // 0b00000100
const DNS_ADULT_BLOCKING_IP_BIT: u8 = 1 << 3; // 0b00001000
const DNS_GAMBLING_BLOCKING_IP_BIT: u8 = 1 << 4; // 0b00010000

/// Return the resolvers as a vector of `IpAddr`s. Returns `None` when no special resolvers
/// are requested and the tunnel default gateway should be used.
pub fn addresses_from_options(options: &DnsOptions) -> Option<Vec<IpAddr>> {
    match options.state {
        DnsState::Default => {
            // Check if we should use a custom blocking DNS resolver.
            // And if so, compute the IP.
            let mut last_byte: u8 = 0;

            if options.default_options.block_ads {
                last_byte |= DNS_AD_BLOCKING_IP_BIT;
            }
            if options.default_options.block_trackers {
                last_byte |= DNS_TRACKER_BLOCKING_IP_BIT;
            }
            if options.default_options.block_malware {
                last_byte |= DNS_MALWARE_BLOCKING_IP_BIT;
            }
            if options.default_options.block_adult_content {
                last_byte |= DNS_ADULT_BLOCKING_IP_BIT;
            }
            if options.default_options.block_gambling {
                last_byte |= DNS_GAMBLING_BLOCKING_IP_BIT;
            }

            if last_byte != 0 {
                let mut dns_ip = DNS_BLOCKING_IP_BASE.octets();
                dns_ip[dns_ip.len() - 1] |= last_byte;
                Some(vec![IpAddr::V4(Ipv4Addr::from(dns_ip))])
            } else {
                None
            }
        }
        DnsState::Custom => {
            if options.custom_options.addresses.is_empty() {
                None
            } else {
                Some(options.custom_options.addresses.clone())
            }
        }
    }
}
