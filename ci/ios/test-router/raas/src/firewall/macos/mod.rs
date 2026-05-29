use std::{
    io::{self},
    result::Result::Ok,
};

use sysctl::{Ctl, CtlValue, Sysctl};
use tun::{AbstractDevice, AsyncDevice};

mod block_list;
pub use block_list::BlockList;
mod utun_router;

mod firewall_configuration;
pub use firewall_configuration::FirewallConfiguration;

pub struct TunnelDevices {
    pub source: AsyncDevice,
    pub sink: AsyncDevice,
}

pub fn configure() -> Result<(BlockList, String, FirewallConfiguration), io::Error> {
    // The MTU is left at the device default. It does not have to track the default route's: we
    // terminate the client's TCP connections ourselves, so the client's leg and the upstream leg
    // negotiate their MSS independently and the upstream path MTU never constrains this one.
    let source = build_utun("10.0.0.2", "10.0.0.1")?;
    let source_name = source.tun_name().map_err(io::Error::other)?;
    let sink_peer = "10.0.0.2";
    let sink = build_utun("10.0.0.3", sink_peer)?;
    let sink_name = sink.tun_name().map_err(io::Error::other)?;

    log::debug!("using source utun with name {source_name}, sink utun with name {sink_name}");
    let configuration = configure_firewall_rules(source_name.clone(), sink_name, sink_peer)?;

    let tunnel_device = TunnelDevices { source, sink };
    Ok((BlockList::new(tunnel_device), source_name, configuration))
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

    let configuration = FirewallConfiguration;
    configuration
        .apply(&wan_if, source_name, sink_peer, sink_name)
        .expect("Failed applying firewall configuration");

    Ok(configuration)
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
