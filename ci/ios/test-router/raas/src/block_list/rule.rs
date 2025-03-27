use crate::web::routes::TransportProtocol;
use mnl::mnl_sys::libc;
use nftnl::{expr, nft_expr, nft_expr_payload, Chain, Rule};

use ipnetwork::IpNetwork;
use std::{collections::BTreeSet, iter};

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

#[derive(Clone, serde::Serialize)]
pub struct Endpoints {
    pub src: IpNetwork,
    pub dst: IpNetwork,
}

impl BlockRule {
    pub fn create_nft_rule<'a>(
        &'a self,
        chain: &'a Chain<'a>,
    ) -> Box<dyn Iterator<Item = Rule<'a>> + 'a> {
        match self {
            BlockRule::Host { protocols, .. } if !protocols.is_empty() => {
                let iter = protocols
                    .iter()
                    .map(|protocol| self.create_nft_rule_inner(chain, Some(*protocol)));
                Box::new(iter)
            }
            _ => Box::new(iter::once(self.create_nft_rule_inner(chain, None))),
        }
    }

    fn create_nft_rule_inner<'a>(
        &self,
        chain: &'a Chain<'a>,
        transport_protocol: Option<TransportProtocol>,
    ) -> Rule<'a> {
        let mut rule = Rule::new(chain);

        match *self {
            BlockRule::Host {
                endpoints: Endpoints { src, dst },
                ..
            } => {
                check_l3proto(&mut rule, src);
                if let Some(protocol) = transport_protocol {
                    check_l4proto(&mut rule, protocol);
                };
                check_ip_addrs(&mut rule, src, dst);
            }
            BlockRule::WireGuard {
                endpoints: Endpoints { src, dst },
            } => {
                check_l3proto(&mut rule, src);
                check_ip_addrs(&mut rule, src, dst);
                check_wireguard_traffic(&mut rule);
            }
        }

        rule.add_expr(&nft_expr!(counter));
        rule.add_expr(&expr::Verdict::Drop);
        rule
    }
}

fn check_l3proto(rule: &mut Rule<'_>, ip: IpNetwork) {
    rule.add_expr(&nft_expr!(meta nfproto));
    rule.add_expr(&nft_expr!(cmp == l3proto(ip)));
}

fn l3proto(addr: IpNetwork) -> u8 {
    match addr {
        IpNetwork::V4(_) => libc::NFPROTO_IPV4 as u8,
        IpNetwork::V6(_) => libc::NFPROTO_IPV6 as u8,
    }
}

fn check_l4proto(rule: &mut Rule<'_>, protocol: TransportProtocol) {
    rule.add_expr(&nft_expr!(meta l4proto));
    rule.add_expr(&nft_expr!(cmp == protocol.as_ipproto()));
}

fn check_ip_addrs(rule: &mut Rule, src: IpNetwork, dst: IpNetwork) {
    // Add source checking
    rule.add_expr(match src {
        IpNetwork::V4(_) => &nft_expr!(payload ipv4 saddr),
        IpNetwork::V6(_) => &nft_expr!(payload ipv6 saddr),
    });
    check_matches_prefix(rule, src);

    // Add destination check
    rule.add_expr(match dst {
        IpNetwork::V4(_) => &nft_expr!(payload ipv4 daddr),
        IpNetwork::V6(_) => &nft_expr!(payload ipv6 daddr),
    });
    check_matches_prefix(rule, dst);

    fn check_matches_prefix(rule: &mut Rule, network: IpNetwork) {
        // Compute the bitwise AND of the incoming packet IP address and the mask, and then
        // compare this value to the bitwise AND of the rule IP address and the mask.
        // This will match when the network prefixes of the IP address to filter and rule
        // IP address matches. E.g. the rule IP address/network 34.117.0.0/16 will match
        // an incoming packet IP address 34.117.105.189.
        match network {
            IpNetwork::V4(addr) => {
                rule.add_expr(&nft_expr!(bitwise mask addr.mask(), xor 0x0));
                rule.add_expr(&nft_expr!(cmp == addr.ip() & addr.mask()));
            }
            IpNetwork::V6(addr) => {
                rule.add_expr(&nft_expr!(bitwise mask addr.mask(), xor 0x0));
                rule.add_expr(&nft_expr!(cmp == addr.ip() & addr.mask()));
            }
        };
    }
}

fn check_wireguard_traffic(rule: &mut Rule) {
    rule.add_expr(&nft_expr!(meta l4proto));
    rule.add_expr(&nft_expr!(cmp == libc::IPPROTO_UDP));

    // UDP header is 8 bytes, after which we have the WireGuard header which is 4 bytes,
    // where the first byte can be 1 to 4 (inclusive) and the last 3 bytes are 0.
    // See: https://wiki.wireshark.org/WireGuard
    rule.add_expr(&nft_expr_payload!(th 8, 1));
    rule.add_expr(&nft_expr!(cmp >= 1));

    rule.add_expr(&nft_expr_payload!(th 8, 1));
    rule.add_expr(&nft_expr!(cmp <= 4));

    rule.add_expr(&nft_expr_payload!(th 9, 3));
    rule.add_expr(&nft_expr!(cmp == 0));
}
