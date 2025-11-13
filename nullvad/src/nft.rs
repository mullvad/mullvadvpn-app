use std::ffi::{CStr, CString};

use anyhow::{Context, anyhow};
use nftnl::{
    Batch, Chain, ChainType, FinalizedBatch, Hook, MsgType, Policy, ProtoFamily, Rule, Table,
    nft_expr,
};

use crate::VETH1;

/// nftables table name
const TABLE_NAME: &CStr = c"nullvad";

// TODO: use constants defined in mullvad crates
pub const SPLIT_TUNNEL_MARK: i32 = 0xf41;
pub const TUNNEL_FWMARK: u32 = 0x6d6f6c65;

pub async fn add_nft_rules() -> anyhow::Result<()> {
    let mut batch = Batch::new();

    let table = Table::new(TABLE_NAME, ProtoFamily::Inet);
    batch.add(&table, MsgType::Add);

    let mut input_chain = Chain::new(c"input", &table);
    input_chain.set_type(ChainType::Filter);
    input_chain.set_hook(Hook::PreRouting, 0);
    input_chain.set_policy(Policy::Accept);
    batch.add(&input_chain, MsgType::Add);

    let veth1 = CString::new(VETH1).unwrap();
    {
        let mut rule = Rule::new(&input_chain);
        rule.add_expr(&nft_expr!(meta iifname));
        rule.add_expr(&nft_expr!(cmp == veth1.as_c_str()));

        // TODO: figure out how to route traffic from nullvad namespace outside the tunnel,
        // because this aint it...
        //
        // Load `MARK` into first nftnl register
        rule.add_expr(&nft_expr!(immediate data SPLIT_TUNNEL_MARK));
        // Set `MARK` as connection tracker mark
        rule.add_expr(&nft_expr!(ct mark set));
        // Load `fwmark` into first nftnl register
        rule.add_expr(&nft_expr!(immediate data TUNNEL_FWMARK));
        // Set `fwmark` as metadata mark for packet
        rule.add_expr(&nft_expr!(meta mark set));
        rule.add_expr(&nft_expr!(counter));
        batch.add(&rule, nftnl::MsgType::Add);
    }

    // TODO: it appears that nullvad traffic is caught by the existing snat rules, probably this one:
    // oif != "lo" ct mark 0x00000f41 masquerade

    /*
    let mut nat_chain = Chain::new(c"nat", &table);
    nat_chain.set_type(ChainType::Nat);
    nat_chain.set_hook(Hook::PostRouting, 0);
    nat_chain.set_policy(Policy::Accept);
    batch.add(&nat_chain, MsgType::Add);

    let veth1 = CString::new(VETH1).unwrap();
    {
        let mut rule = Rule::new(&nat_chain);
        rule.add_expr(&nft_expr!(meta iifname));
        rule.add_expr(&nft_expr!(cmp == veth1.as_c_str()));
        rule.add_expr(&nft_expr!(masquerade));
        rule.add_expr(&nft_expr!(counter));

        batch.add(&rule, nftnl::MsgType::Add);
    }
    */

    let batch = batch.finalize();
    send_and_process(&batch).with_context(|| anyhow!("Failed to add nft table {TABLE_NAME:?}"))?;

    Ok(())
}

pub async fn remove_nft_rules() -> anyhow::Result<()> {
    let mut batch = Batch::new();

    let table = Table::new(TABLE_NAME, ProtoFamily::Inet);
    batch.add(&table, MsgType::Del);

    let batch = batch.finalize();
    send_and_process(&batch)
        .with_context(|| anyhow!("Failed to remove nft table {TABLE_NAME:?}"))?;

    Ok(())
}

fn send_and_process(batch: &FinalizedBatch) -> anyhow::Result<()> {
    // Create a netlink socket to netfilter.
    let socket = mnl::Socket::new(mnl::Bus::Netfilter)?;
    // Send all the bytes in the batch.
    socket.send_all(batch)?;

    let portid = socket.portid();
    let mut buffer = vec![0; nftnl::nft_nlmsg_maxsize() as usize];

    for expected_seq in batch.sequence_numbers() {
        while let Some(message) = socket_recv(&socket, &mut buffer[..])? {
            // TODO: tbh, passing expected_seq is not very useful, since the error sucks
            match mnl::cb_run(message, expected_seq, portid)? {
                mnl::CbResult::Stop => break,
                // TODO: can this ever happen? not for these messages.
                mnl::CbResult::Ok => {}
            }
        }
    }
    Ok(())
}

fn socket_recv<'a>(socket: &mnl::Socket, buf: &'a mut [u8]) -> anyhow::Result<Option<&'a [u8]>> {
    let ret = socket.recv(buf)?;
    Ok((ret > 0).then_some(&buf[..ret]))
}
