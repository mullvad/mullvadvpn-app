use nftnl::{expr, nft_expr, nft_expr_payload, Batch, Chain, FinalizedBatch, ProtoFamily, Rule, Table};
use mnl::mnl_sys::libc;
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, ffi::CString, io};

use super::rule::{BlockRule, Endpoints};
use crate::web::routes::TransportProtocol;
use ipnetwork::IpNetwork;

static TABLE_NAME: Lazy<CString> = Lazy::new(|| CString::new("raas").unwrap());
static FORWARD_CHAIN_NAME: Lazy<CString> = Lazy::new(|| CString::new("forward").unwrap());

#[derive(Default)]
pub struct BlockList {
    rules: BTreeMap<uuid::Uuid, Vec<BlockRule>>,
}

impl BlockList {
    pub fn add_rules(&mut self, rules: &[BlockRule], label: uuid::Uuid) -> io::Result<()> {
        let rules_for_label = self.rules.entry(label).or_default();
        rules_for_label.extend_from_slice(rules);
        self.apply_rules()
    }

    pub fn clear_rules_with_label(&mut self, label: &uuid::Uuid) -> io::Result<()> {
        let _ = self.rules.remove(label);
        self.apply_rules()
    }

    pub fn rules(&self) -> &BTreeMap<uuid::Uuid, Vec<BlockRule>> {
        &self.rules
    }

    fn apply_rules(&mut self) -> io::Result<()> {
        let table = Table::new(&*TABLE_NAME, ProtoFamily::Inet);
        let batch = self.create_batch(&table);
        self.send_netlink_batch(&batch)
    }

    fn create_batch(&mut self, table: &Table) -> FinalizedBatch {
        let mut batch = Batch::new();

        // Create the table if it does not exist and clear it otherwise.
        batch.add(table, nftnl::MsgType::Add);
        batch.add(table, nftnl::MsgType::Del);
        batch.add(table, nftnl::MsgType::Add);

        let mut forward_chain = Chain::new(&*FORWARD_CHAIN_NAME, table);
        forward_chain.set_hook(nftnl::Hook::Forward, 0);
        forward_chain.set_policy(nftnl::Policy::Accept);
        batch.add(&forward_chain, nftnl::MsgType::Add);
        for rule in self.nft_forward_rules(&forward_chain) {
            batch.add(&rule, nftnl::MsgType::Add);
        }

        batch.finalize()
    }

    fn send_netlink_batch(&self, batch: &FinalizedBatch) -> io::Result<()> {
        let socket = mnl::Socket::new(mnl::Bus::Netfilter)?;
        socket.send_all(batch)?;

        let portid = socket.portid();
        let mut buffer = vec![0; nftnl::nft_nlmsg_maxsize() as usize];

        let seq = 0;
        for message in socket.recv(&mut buffer[..])? {
            let message = message?;
            match mnl::cb_run(message, seq, portid)? {
                mnl::CbResult::Stop => {
                    break;
                }
                mnl::CbResult::Ok => (),
            };
        }

        Ok(())
    }

    fn nft_forward_rules<'a>(&'a self, chain: &'a Chain<'a>) -> impl Iterator<Item = Rule<'a>> {
        self.rules
            .values()
            .flatten()
            .flat_map(move |rule| create_nft_rules(rule, chain))
    }
}

/// Creates one or more nft rules that correspond to a BlockRule.
fn create_nft_rules<'a>(block_rule: &BlockRule, chain: &'a Chain<'a>) -> Vec<Rule<'a>> {
    let rules = match block_rule {
        BlockRule::Host { protocols, .. } if !protocols.is_empty() => protocols
            .iter()
            .flat_map(|protocol| create_nft_rules_inner(block_rule, chain, Some(*protocol)))
            .collect(),
        _ => create_nft_rules_inner(block_rule, chain, None),
    };
    assert!(!rules.is_empty());
    rules
}

fn create_nft_rules_inner<'a>(
    block_rule: &BlockRule,
    chain: &'a Chain<'a>,
    transport_protocol: Option<TransportProtocol>,
) -> Vec<Rule<'a>> {
    let mut rules = Vec::new();

    match *block_rule {
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
                main_rule.add_expr(&expr::Verdict::Accept);
                rules.push(main_rule);

                let mut block_rule = nft_rule_host(chain, src, None, transport_protocol);
                block_rule.add_expr(&expr::Verdict::Drop);
                rules.push(block_rule);
            } else {
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
