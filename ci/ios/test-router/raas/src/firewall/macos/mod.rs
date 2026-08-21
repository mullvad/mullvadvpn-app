use std::{
    collections::{BTreeMap, HashMap},
    io::{self, Write},
    net::IpAddr,
    path::PathBuf,
    process::{Command, Stdio},
    result::Result::Ok,
    sync::Arc,
};

use sysctl::{Ctl, CtlValue, Sysctl};

use arc_swap::ArcSwap;
use ipnetwork::IpNetwork;
use tun::{AbstractDevice, AsyncDevice};

use super::rule::BlockRule;

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
            labeled
                .entry(entry.label)
                .or_default()
                .push(entry.rule.clone());
        }
        labeled
    }
}

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

pub struct TunnelDevices {
    pub source: AsyncDevice,
    pub sink: AsyncDevice,
}

pub struct FirewallConfiguration {
    anchor_file: String,
    rules_file: String,
}

impl FirewallConfiguration {
    pub fn new(output_dir: PathBuf) -> Self {
        let mut anchors = output_dir.clone();
        anchors.push("raas-anchors");
        anchors.set_extension("conf");

        let mut rules = output_dir.clone();
        rules.push("raas-rules");
        rules.set_extension("conf");

        return FirewallConfiguration {
            anchor_file: anchors
                .to_str()
                .unwrap_or_else(|| panic!("Unable to convert {anchors:?} into a str"))
                .to_string(),
            rules_file: rules
                .to_str()
                .unwrap_or_else(|| panic!("Unable to convert {rules:?} into a str"))
                .to_string(),
        };
    }

    pub fn apply(
        &self,
        wan_if: &str,
        source_name: String,
        sink_peer: &str,
        sink_name: String,
    ) -> Result<(), io::Error> {
        let anchors = &self.anchor_file;
        let rules = &self.rules_file;

        // We need to write a file with anchors that includes the rules,
        // as `pf` does not support adding anchors and rules under that anchor
        // in one go. This is stored in a unique tmp directory.
        // We do load the existing pf rules to ensure applications keep
        // working.
        let script = r#"
set -eu

cat > "$ANCHOR_FILE" <<EOF

nat-anchor "$ANCHOR_NAME" all
rdr-anchor "$ANCHOR_NAME" all
anchor "$ANCHOR_NAME" all
EOF

sed -e "s|source_utun|$SOURCE_IFACE|g" \
    -e "s|sink_utun|$SINK_IFACE|g" \
    -e "s|sink_peer|$SINK_PEER|g" \
    -e "s|wan_if|$WAN_IF|g" \
    ./poc/pf.utun4.conf > "$RULE_FILE"

pfctl -f "$ANCHOR_FILE"
pfctl -a "$ANCHOR_NAME" -f "$RULE_FILE" || { echo "pfctl rejected the rules in $RULE_FILE" >&2; exit 1; }
pfctl -e || true
"#;

        let mut shell = Command::new("sh")
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env("SOURCE_IFACE", &source_name)
            .env("SINK_IFACE", &sink_name)
            .env("WAN_IF", &wan_if)
            .env("SINK_PEER", sink_peer)
            .env("ANCHOR_NAME", "net.mullvad.raas")
            .env("ANCHOR_FILE", anchors)
            .env("RULE_FILE", rules)
            .spawn()?;

        let mut stdin = shell
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("failed to get stdin"))?;

        println!("Source interface - {}", source_name);
        println!("WAN interface - {}", wan_if);
        println!("Sink interface - {}", sink_name);

        println!(
            "Applying forwarding rules: {} -> {}, replies via {}",
            wan_if, source_name, sink_name
        );
        stdin.write_all(script.as_bytes())?;
        drop(stdin); // close the pipe so the shell sees EOF

        let status = shell.wait()?;

        if !status.success() {
            return Err(io::Error::other(format!(
                "Failed to apply firewall configuration: {status}"
            )));
        }

        Ok(())
    }

    pub fn reset(&self) -> Result<(), io::Error> {
        let anchor_file = &self.rules_file;
        let rules_file = &self.rules_file;

        let mut shell = Command::new("sh")
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env("ANCHOR_FILE", anchor_file)
            .env("RULES_FILE", rules_file)
            .spawn()?;

        let mut stdin = shell
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("failed to get stdin"))?;

        let script = format!(
            r#"
            pfctl -f "/etc/pf.conf" && \
            rm -f "$RULES_FILE" "$ANCHOR_FILE"
            "#
        );

        println!("Loading main rules in /etc/pf.conf");
        stdin.write_all(script.as_bytes())?;
        drop(stdin); // close the pipe so the shell sees EOF
        //
        let status = shell.wait()?;
        if !status.success() {
            return Err(io::Error::other(format!(
                "Failed to load reset firewall to initial state. Something wrong in /etc/pf.conf? Failure: {status}"
            )));
        }

        Ok(())
    }
}

pub fn setup_utun() -> Result<(TunnelDevices, FirewallConfiguration), io::Error> {
    // The MTU is left at the device default. It does not have to track the default route's: we
    // terminate the client's TCP connections ourselves, so the client's leg and the upstream leg
    // negotiate their MSS independently and the upstream path MTU never constrains this one.
    let source = build_utun("10.0.0.2", "10.0.0.1")?;
    let source_name = source.tun_name().map_err(io::Error::other)?;
    let sink_peer = "10.0.0.2";
    let sink = build_utun("10.0.0.3", sink_peer)?;
    let sink_name = sink.tun_name().map_err(io::Error::other)?;
    println!("using source utun with name {source_name}, sink utun with name {sink_name}");
    let configuration = configure_firewall_rules(source_name, sink_name, sink_peer)?;

    Ok((TunnelDevices { source, sink }, configuration))
}

fn build_utun(address: &str, destination: &str) -> Result<AsyncDevice, io::Error> {
    let mut config = tun::Configuration::default();
    config
        .address(address)
        .destination(destination)
        .netmask("255.255.255.0")
        .up();
    // A macOS utun fd always carries a four byte address family header. Leaving packet
    // information on hands that header to the crate, so the router only ever sees raw IP.
    config.platform_config(|platform| {
        platform.packet_information(true);
    });
    tun::create_as_async(&config).map_err(io::Error::other)
}

fn configure_firewall_rules(
    source_name: String,
    sink_name: String,
    sink_peer: &str,
) -> Result<FirewallConfiguration, io::Error> {
    let wan_if = netdev::interface::get_default_interface()
        .map_err(|e| io::Error::other(format!("failed to get default interface: {e}")))?
        .name;

    // This is MacOS specific
    let ip_forwarding =
        Ctl::new("net.inet.ip.forwarding").unwrap_or_else(|f| panic!("Could not read {}", f));

    let ctl_enabled = CtlValue::Int(1);
    ip_forwarding
        .set_value(ctl_enabled)
        .expect("failed to set net.inet.ip.forwarding");

    let tmp_dir = std::env::temp_dir().join("raas");
    let configuration = FirewallConfiguration::new(tmp_dir);

    configuration
        .apply(&wan_if, source_name, sink_peer, sink_name)
        .expect("Failed applying firewall configuration");

    return Ok(configuration);
}
