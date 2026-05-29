use std::{
    collections::{BTreeSet, BTreeMap, HashMap},
    net::IpAddr,
};
use crate::web::routes::TransportProtocol;

use ipnetwork::IpNetwork;

#[derive(Clone, Copy, serde::Serialize)]
pub struct Endpoints {
    pub src: IpNetwork,
    pub dst: IpNetwork,
    /// Normally a packet sent to `dst` would match the block rule, but this option inverts that
    /// so that any packet *not* sent to `dst` will match the block rule.
    pub invert_dst: bool,
}

#[derive(Clone)]
pub struct LabeledRule {
    label: uuid::Uuid,
    rule: BlockRule,
}

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

impl BlockRule {
    /// The source and destination this rule applies to.
    pub fn endpoints(&self) -> &Endpoints {
        match self {
            BlockRule::Host { endpoints, .. } | BlockRule::WireGuard { endpoints } => endpoints,
        }
    }
}

/// The block rules bucketed by their source, so a packet is only ever tested against the rules
/// that could name it. Snapshotted as a whole so a packet is never judged by half an update.
#[derive(Clone, Default)]
pub struct RuleSet {
    /// Rules whose source is a single host, found by the packet's source address.
    by_host: HashMap<IpAddr, Vec<LabeledRule>>,
    /// Rules whose source is a wider network, tried one by one.
    by_prefix: Vec<LabeledRule>,
}

impl RuleSet {
    pub fn insert(&mut self, rule: BlockRule, label: uuid::Uuid) {
        let host = match rule.endpoints().src {
            IpNetwork::V4(network) if network.prefix() == 32 => Some(IpAddr::from(network.ip())),
            IpNetwork::V6(network) if network.prefix() == 128 => Some(IpAddr::from(network.ip())),
            _ => None,
        };
        let entry = LabeledRule { label, rule };
        match host {
            Some(host) => self.by_host.entry(host).or_default().push(entry),
            None => self.by_prefix.push(entry),
        }
    }

    pub fn remove_label(&mut self, label: &uuid::Uuid) {
        self.by_host.retain(|_, rules| {
            rules.retain(|entry| entry.label != *label);
            !rules.is_empty()
        });
        self.by_prefix.retain(|entry| entry.label != *label);
    }

    pub fn is_empty(&self) -> bool {
        self.by_host.is_empty() && self.by_prefix.is_empty()
    }

    /// The rules whose source could cover this address. Prefix sources are not checked here, so
    /// the caller must still match each candidate in full.
    pub fn candidates(&self, src: IpAddr) -> impl Iterator<Item = &BlockRule> {
        let hosted = self.by_host.get(&src).into_iter().flatten();
        hosted.chain(&self.by_prefix).map(|entry| &entry.rule)
    }

    /// The rules regrouped by label, which is the shape the HTTP API speaks.
    pub fn by_label(&self) -> BTreeMap<uuid::Uuid, Vec<BlockRule>> {
        let mut labeled = BTreeMap::<_, Vec<BlockRule>>::new();
        for entry in self.by_host.values().flatten().chain(&self.by_prefix) {
            labeled
                .entry(entry.label)
                .or_default()
                .push(entry.rule.clone());
        }
        labeled
    }
}
