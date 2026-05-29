use std::{collections::BTreeMap, io, sync::Arc};

use super::{TunnelDevices, utun_router};
use arc_swap::ArcSwap;

use crate::firewall::rule::{BlockRule, RuleSet};

pub struct BlockList {
    rules: Arc<ArcSwap<RuleSet>>,
}

impl BlockList {
    pub(crate) fn new(tunnel_devices: TunnelDevices) -> Self {
        let rules = Arc::new(ArcSwap::from_pointee(RuleSet::default()));
        utun_router::Router::spawn(tunnel_devices, rules.clone());
        Self { rules }
    }

    pub fn add_rules(&mut self, rules: &[BlockRule], label: uuid::Uuid) -> io::Result<()> {
        self.rules.rcu(|current| {
            let mut updated = RuleSet::clone(current);
            for rule in rules {
                updated.insert(rule.clone(), label);
            }
            updated
        });
        Ok(())
    }

    pub fn clear_rules_with_label(&mut self, label: &uuid::Uuid) -> io::Result<()> {
        self.rules.rcu(|current| {
            let mut updated = RuleSet::clone(current);
            updated.remove_label(label);
            updated
        });
        Ok(())
    }

    pub fn rules(&self) -> BTreeMap<uuid::Uuid, Vec<BlockRule>> {
        self.rules.load().by_label()
    }
}
