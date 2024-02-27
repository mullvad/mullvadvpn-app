use crate::web::routes::TransportProtocol;
use mnl::mnl_sys::libc;
use nftnl::{expr, nft_expr, Chain, Rule};

use std::{collections::BTreeSet, iter, net::IpAddr};

#[derive(Clone)]
pub struct BlockRule {
    pub src: IpAddr,
    pub dst: IpAddr,
    pub protocols: BTreeSet<TransportProtocol>,
}

impl BlockRule {
    pub fn create_nft_rule<'a>(
        &'a self,
        chain: &'a Chain<'a>,
    ) -> Box<dyn Iterator<Item = Rule<'a>> + 'a> {
        if self.protocols.is_empty() {
            return Box::new(iter::once(self.create_nft_rule_inner(chain, None)));
        }

        let iter = self
            .protocols
            .iter()
            .map(|protocol| self.create_nft_rule_inner(chain, Some(*protocol)));
        Box::new(iter)
    }

    fn create_nft_rule_inner<'a>(
        &self,
        chain: &'a Chain<'a>,
        transport_protocol: Option<TransportProtocol>,
    ) -> Rule<'a> {
        let mut rule = Rule::new(chain);
        check_l3proto(&mut rule, self.src);
        if let Some(protocol) = transport_protocol {
            check_l4proto(&mut rule, protocol);
        };

        // Add source checking
        rule.add_expr(match self.src {
            IpAddr::V4(_) => &nft_expr!(payload ipv4 saddr),
            IpAddr::V6(_) => &nft_expr!(payload ipv6 saddr),
        });
        match self.src {
            IpAddr::V4(addr) => rule.add_expr(&nft_expr!(cmp == addr)),
            IpAddr::V6(addr) => rule.add_expr(&nft_expr!(cmp == addr)),
        };

        // Add destination check
        rule.add_expr(match self.dst {
            IpAddr::V4(_) => &nft_expr!(payload ipv4 daddr),
            IpAddr::V6(_) => &nft_expr!(payload ipv6 daddr),
        });
        match self.dst {
            IpAddr::V4(addr) => rule.add_expr(&nft_expr!(cmp == addr)),
            IpAddr::V6(addr) => rule.add_expr(&nft_expr!(cmp == addr)),
        };
        rule.add_expr(&nft_expr!(counter));
        rule.add_expr(&expr::Verdict::Drop);

        rule
    }
}

fn check_l3proto(rule: &mut Rule<'_>, ip: IpAddr) {
    rule.add_expr(&nft_expr!(meta nfproto));
    rule.add_expr(&nft_expr!(cmp == l3proto(ip)));
}

fn l3proto(addr: IpAddr) -> u8 {
    match addr {
        IpAddr::V4(_) => libc::NFPROTO_IPV4 as u8,
        IpAddr::V6(_) => libc::NFPROTO_IPV6 as u8,
    }
}

fn check_l4proto(rule: &mut Rule<'_>, protocol: TransportProtocol) {
    rule.add_expr(&nft_expr!(meta l4proto));
    rule.add_expr(&nft_expr!(cmp == protocol.as_ipproto()));
}
