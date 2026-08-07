use std::{
    collections::{BTreeMap, HashMap},
    io,
    net::IpAddr,
    process::{Command, Stdio},
    sync::Arc,
};

use arc_swap::ArcSwap;
use ipnetwork::IpNetwork;
use tun_rs::{AsyncDevice, DeviceBuilder};

use super::rule::BlockRule;

/// MTU to route with when the tunnel device will not tell us its own.
const DEFAULT_MTU: u16 = 1500;

mod utun_router;

/// The block rules bucketed by their source, so a packet is only ever tested against the rules
/// that could name it. Snapshotted as a whole so a packet is never judged by half an update.
#[derive(Clone, Default)]
pub(crate) struct RuleSet {
    /// Rules whose source is a single host, found by the packet's source address.
    by_host: HashMap<IpAddr, Vec<LabeledRule>>,
    /// Rules whose source is a wider network, tried one by one.
    by_prefix: Vec<LabeledRule>,
}

#[derive(Clone)]
struct LabeledRule {
    label: uuid::Uuid,
    rule: BlockRule,
}

impl RuleSet {
    pub(crate) fn insert(&mut self, rule: BlockRule, label: uuid::Uuid) {
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

    fn remove_label(&mut self, label: &uuid::Uuid) {
        self.by_host.retain(|_, rules| {
            rules.retain(|entry| entry.label != *label);
            !rules.is_empty()
        });
        self.by_prefix.retain(|entry| entry.label != *label);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.by_host.is_empty() && self.by_prefix.is_empty()
    }

    /// The rules whose source could cover this address. Prefix sources are not checked here, so
    /// the caller must still match each candidate in full.
    pub(crate) fn candidates(&self, src: IpAddr) -> impl Iterator<Item = &BlockRule> {
        let hosted = self.by_host.get(&src).into_iter().flatten();
        hosted.chain(&self.by_prefix).map(|entry| &entry.rule)
    }

    /// The rules regrouped by label, which is the shape the HTTP API speaks.
    fn by_label(&self) -> BTreeMap<uuid::Uuid, Vec<BlockRule>> {
        let mut labeled = BTreeMap::<_, Vec<BlockRule>>::new();
        for entry in self.by_host.values().flatten().chain(&self.by_prefix) {
            labeled.entry(entry.label).or_default().push(entry.rule.clone());
        }
        labeled
    }
}

pub struct BlockList {
    rules: Arc<ArcSwap<RuleSet>>,
}

impl BlockList {
    pub(crate) fn new(tunnel_devices: TunnelDevices) -> Self {
        // let mtu = tunnel_device.mtu().unwrap_or_else(|err| {
        //     log::warn!("Failed to read the tunnel device MTU, assuming {DEFAULT_MTU}: {err}");
        //     DEFAULT_MTU
        // });

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

pub struct TunnelDevices {
    pub source: AsyncDevice,
    pub sink: AsyncDevice,
}

pub fn setup_utun() -> Result<TunnelDevices, io::Error> {
    // The MTU is left at the device default. It does not have to track the default route's: we
    // terminate the client's TCP connections ourselves, so the client's leg and the upstream leg
    // negotiate their MSS independently and the upstream path MTU never constrains this one.
    let source = DeviceBuilder::new()
        .ipv4("10.0.0.2", 24, Some("10.0.0.1"))
        .packet_information(false)
        .build_async()?;
    let source_name = source.name()?;
    let sink = DeviceBuilder::new()
        .ipv4("10.0.0.3", 24, Some("10.0.0.2"))
        .packet_information(false)
        .build_async()?;
    let sink_name = sink.name()?;
    println!("using source utun with name {source_name}, sink utun with name {sink_name}");
    execute_setup_script(source_name, sink_name)?;

    Ok(TunnelDevices { source, sink })
}

fn execute_setup_script(source_name: String, sink_name: String) -> Result<(), io::Error> {
    let status = Command::new("zsh")
        .arg("./poc/post_up.sh")
        .arg(source_name)
        .arg(sink_name)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if let Some(exit_code) = status.code()
        && exit_code != 0
    {
        return Err(io::Error::other(format!(
            "post_up.sh script failed with exit code {exit_code}"
        )));
    }

    Ok(())
}
