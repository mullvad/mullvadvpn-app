//! Fork-owned IP split-tunnel firewall rules.

use super::{
    ADD_COUNTERS, End, Firewall, MANGLE_CHAIN_NAME, MANGLE_CHAIN_PRIORITY, Result, check_net,
};
use crate::split_tunnel;
use ipnetwork::IpNetwork;
use nftnl::{Batch, Chain, ProtoFamily, Rule, Table, nft_expr};
use std::ffi::CStr;

const TABLE_NAME: &CStr = c"mullvad-ip-split-tunnel";

/// Apply fork-owned IP split-tunnel marking rules.
///
/// These rules only mark matching destination traffic. The main Mullvad firewall still owns the
/// accept, masquerade, and return-traffic rules for split-tunneled connections.
pub fn apply_ranges(fwmark: u32, ranges: &[ipnetwork::Ipv4Network]) -> Result<()> {
    if ranges.is_empty() {
        return reset_ranges();
    }

    let table = Table::new(TABLE_NAME, ProtoFamily::Inet);
    let mut batch = Batch::new();

    batch.add(&table, nftnl::MsgType::Add);
    batch.add(&table, nftnl::MsgType::Del);
    batch.add(&table, nftnl::MsgType::Add);

    let mut mangle_chain = Chain::new(MANGLE_CHAIN_NAME, &table);
    mangle_chain.set_hook(nftnl::Hook::Out, MANGLE_CHAIN_PRIORITY);
    mangle_chain.set_type(nftnl::ChainType::Route);
    mangle_chain.set_policy(nftnl::Policy::Accept);
    batch.add(&mangle_chain, nftnl::MsgType::Add);

    for range in ranges {
        let mut rule = Rule::new(&mangle_chain);
        check_net(&mut rule, End::Dst, IpNetwork::V4(*range));
        rule.add_expr(&nft_expr!(immediate data split_tunnel::MARK));
        rule.add_expr(&nft_expr!(ct mark set));
        rule.add_expr(&nft_expr!(immediate data fwmark));
        rule.add_expr(&nft_expr!(meta mark set));
        if *ADD_COUNTERS {
            rule.add_expr(&nft_expr!(counter));
        }
        batch.add(&rule, nftnl::MsgType::Add);
    }

    let batch = batch.finalize();
    Firewall::send_and_process(&batch)?;
    Firewall::verify_tables(&[TABLE_NAME])
}

/// Remove fork-owned IP split-tunnel marking rules.
pub fn reset_ranges() -> Result<()> {
    let table = Table::new(TABLE_NAME, ProtoFamily::Inet);
    let mut batch = Batch::new();

    batch.add(&table, nftnl::MsgType::Add);
    batch.add(&table, nftnl::MsgType::Del);

    let batch = batch.finalize();
    Firewall::send_and_process(&batch)
}
