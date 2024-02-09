use nftnl::{Batch, Chain, FinalizedBatch, ProtoFamily, Rule, Table};
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, ffi::CString, io};

static TABLE_NAME: Lazy<CString> = Lazy::new(|| CString::new("raas").unwrap());
static FORWARD_CHAIN_NAME: Lazy<CString> = Lazy::new(|| CString::new("forward").unwrap());

mod rule;
pub use rule::BlockRule;

#[derive(Default)]
pub struct BlockList {
    rules: BTreeMap<uuid::Uuid, Vec<BlockRule>>,
}

impl BlockList {
    pub fn add_rule(&mut self, rule: BlockRule, label: uuid::Uuid) -> io::Result<()> {
        {
            let rules = self.rules.entry(label).or_default();
            rules.push(rule);
        }
        self.apply_rules()
    }

    pub fn clear_rules_with_label(&mut self, label: &uuid::Uuid) -> io::Result<()> {
        let _ = self.rules.remove(label);
        self.apply_rules()
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
        while let Some(message) = Self::socket_recv(&socket, &mut buffer[..])? {
            match mnl::cb_run(message, seq, portid)? {
                mnl::CbResult::Stop => {
                    break;
                }
                mnl::CbResult::Ok => (),
            };
        }

        Ok(())
    }

    fn socket_recv<'a>(socket: &mnl::Socket, buf: &'a mut [u8]) -> io::Result<Option<&'a [u8]>> {
        let ret = socket.recv(buf)?;
        if ret > 0 {
            Ok(Some(&buf[..ret]))
        } else {
            Ok(None)
        }
    }

    fn nft_forward_rules<'a>(
        &'a self,
        chain: &'a Chain<'a>,
    ) -> impl Iterator<Item = Rule<'a>> + '_ {
        self.rules
            .values()
            .flatten()
            .flat_map(move |rule| rule.create_nft_rule(chain))
    }
}
