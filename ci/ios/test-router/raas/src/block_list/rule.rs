use crate::web::routes::TransportProtocol;
use mnl::mnl_sys::libc;
use nftnl::{expr, nft_expr, nft_expr_payload, Chain, Rule};

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

impl BlockRule {
    /// Creates one or more nft rules that correspond to this BlockRule. The returned Vec will always
    /// have at least one element.
    pub fn create_nft_rules<'a>(&'a self, chain: &'a Chain<'a>) -> Vec<Rule<'a>> {
        let rules = match self {
            BlockRule::Host { protocols, .. } if !protocols.is_empty() => protocols
                .iter()
                .flat_map(|protocol| self.create_nft_rules_inner(chain, Some(*protocol)))
                .collect(),
            _ => self.create_nft_rules_inner(chain, None),
        };
        assert!(!rules.is_empty());
        rules
    }

    fn create_nft_rules_inner<'a>(
        &self,
        chain: &'a Chain<'a>,
        transport_protocol: Option<TransportProtocol>,
    ) -> Vec<Rule<'a>> {
        let mut rules = Vec::new();

        match *self {
            BlockRule::Host {
                endpoints:
                    Endpoints {
                        src,
                        dst,
                        invert_dst,
                    },
                ..
            } => {
                let mut main_rule = nft_rule_host(chain, src, Some(dst), transport_protocol);
                if invert_dst {
                    // Inverted case - accept all traffic that matches the main rule
                    main_rule.add_expr(&expr::Verdict::Accept);
                    rules.push(main_rule);

                    // Block all other traffic from `src` that is not sent to `dst`
                    let mut block_rule = nft_rule_host(chain, src, None, transport_protocol);
                    block_rule.add_expr(&expr::Verdict::Drop);
                    rules.push(block_rule);
                } else {
                    // Normal case - drop all traffic that matches the main rule
                    main_rule.add_expr(&expr::Verdict::Drop);
                    rules.push(main_rule);
                }
            }
            BlockRule::WireGuard {
                endpoints:
                    Endpoints {
                        src,
                        dst,
                        invert_dst,
                    },
            } => {
                let mut main_rule = nft_rule_wireguard(chain, src, Some(dst));
                if invert_dst {
                    main_rule.add_expr(&expr::Verdict::Accept);
                    rules.push(main_rule);

                    let mut block_rule = nft_rule_wireguard(chain, src, None);
                    block_rule.add_expr(&expr::Verdict::Drop);
                    rules.push(block_rule);
                } else {
                    main_rule.add_expr(&expr::Verdict::Drop);
                    rules.push(main_rule);
                }
            }
        }

        rules
    }
}

fn nft_rule_host<'a>(
    chain: &'a Chain<'a>,
    src: IpNetwork,
    dst: Option<IpNetwork>,
    transport_protocol: Option<TransportProtocol>,
) -> Rule<'a> {
    let mut rule = Rule::new(chain);
    check_l3proto(&mut rule, src);
    if let Some(protocol) = transport_protocol {
        check_l4proto(&mut rule, protocol);
    };
    check_ip_addrs(&mut rule, src, dst);
    rule.add_expr(&nft_expr!(counter));
    rule
}

fn nft_rule_wireguard<'a>(
    chain: &'a Chain<'a>,
    src: IpNetwork,
    dst: Option<IpNetwork>,
) -> Rule<'a> {
    let mut rule = Rule::new(chain);
    check_l3proto(&mut rule, src);
    check_ip_addrs(&mut rule, src, dst);
    check_wireguard_traffic(&mut rule);
    rule.add_expr(&nft_expr!(counter));
    rule
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

fn check_ip_addrs(rule: &mut Rule, src: IpNetwork, dst: Option<IpNetwork>) {
    // Add source checking
    rule.add_expr(match src {
        IpNetwork::V4(_) => &nft_expr!(payload ipv4 saddr),
        IpNetwork::V6(_) => &nft_expr!(payload ipv6 saddr),
    });
    check_matches_prefix(rule, src);

    // Add destination check if dst is given
    if let Some(dst) = dst {
        rule.add_expr(match dst {
            IpNetwork::V4(_) => &nft_expr!(payload ipv4 daddr),
            IpNetwork::V6(_) => &nft_expr!(payload ipv6 daddr),
        });
        check_matches_prefix(rule, dst);
    }

    fn check_matches_prefix(rule: &mut Rule, network: IpNetwork) {
        // Check that the IP address matches the given IP network.
        // E.g. the IP network 34.117.0.0/16 will match an incoming packet IP 34.117.105.189.
        match network {
            IpNetwork::V4(addr) => {
                rule.add_expr(&nft_expr!(bitwise mask addr.mask(), xor 0u32));
            }
            IpNetwork::V6(addr) => {
                rule.add_expr(&nft_expr!(bitwise mask addr.mask(), xor &[0u16; 8][..]));
            }
        };
        rule.add_expr(&nft_expr!(cmp == network.ip()));
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
