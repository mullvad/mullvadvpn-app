#![cfg(target_os = "linux")]

use super::{
    TestContext,
    helpers::{self, AbortOnDrop},
};
use crate::{
    tests::config::TEST_CONFIG,
    vm::{self, network::linux::TEST_SUBNET_IPV4},
};
use anyhow::{Context, anyhow, bail, ensure};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_relay_selector::query::builder::{IpVersion, RelayQueryBuilder};
use mullvad_types::relay_constraints::{ObfuscationSettings, RelayOverride, SelectedObfuscation};
use scopeguard::ScopeGuard;
use std::net::{IpAddr, SocketAddr};
use test_macro::test_function;
use test_rpc::ServiceClient;
use tokio::{net::UdpSocket, task};

const NFT_TABLE_NAME: &str = "relay_override_test";

/// Test that IP overrides work for wireguard relays by:
/// - Picking an arbitrary wireguard relay.
/// - Block the VM from communicating with the relays IP address.
/// - Set up a UDP proxy on the host machine and override the relay IP with the host IP
#[test_function(target_os = "linux", target_os = "windows")]
pub async fn test_wireguard_ip_override(
    _ctx: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    // Note: This should be a valid port according to the relay list
    const TUNNEL_PORT: u16 = 51819;

    // make sure udp2tcp is turned off for this test
    mullvad_client
        .set_obfuscation_settings(ObfuscationSettings {
            selected_obfuscation: SelectedObfuscation::Off,
            ..Default::default()
        })
        .await
        .with_context(|| "Failed to set disable obfuscation")?;

    let guest_interface = rpc.get_default_interface().await?;
    let IpAddr::V4(guest_ip) = rpc.get_interface_ip(guest_interface).await? else {
        bail!("Guests with IPv6 addresses are not supported.");
    };

    // pick any wireguard_constraints relay to use with the test
    let query = RelayQueryBuilder::wireguard()
        .port(TUNNEL_PORT)
        .ip_version(IpVersion::V4)
        .build();
    let relay = helpers::constrain_to_relay(&mut mullvad_client, query)
        .await
        .context("Failed to set WireGuard")?;

    log::info!("connecting to selected relay");
    helpers::connect_and_wait(&mut mullvad_client).await?;

    log::info!("checking that the connection works");
    let _ = helpers::geoip_lookup_with_retries(&rpc).await?;

    log::info!("blocking connection to relay from guest");
    let _remove_nft_rule_on_drop = block_route(guest_ip, relay.ipv4_addr_in).await?;

    log::info!("checking that the connection does not work while blocked");
    ensure!(
        helpers::geoip_lookup_with_retries(&rpc).await.is_err(),
        "Assert that relay is blocked by firewall rule"
    );

    let _proxy_abort_handle = spawn_udp_proxy(
        SocketAddr::new(relay.ipv4_addr_in.into(), TUNNEL_PORT),
        TUNNEL_PORT,
    )
    .await
    .with_context(|| "Failed to spawn UDP proxy")?;

    log::info!("adding proxy to relay ip overrides");
    mullvad_client
        .set_relay_override(RelayOverride {
            hostname: relay.hostname,
            ipv4_addr_in: Some(TEST_CONFIG.host_bridge_ip),
            ipv6_addr_in: None,
        })
        .await?;

    log::info!("checking that the connection works again with the added overrides");
    // Setting an IP override will cause the client to reconnect, so we have to wait for that
    helpers::connect_and_wait(&mut mullvad_client).await?;
    let _ = helpers::geoip_lookup_with_retries(&rpc)
        .await
        .with_context(|| "Can't access internet through relay ip override")?;

    Ok(())
}

/// Add an nftables rule that drops all packets going from `source` to `destination`.
async fn block_route(
    source: impl Into<IpAddr>,
    destination: impl Into<IpAddr>,
) -> anyhow::Result<ScopeGuard<(), impl FnOnce(()), scopeguard::Always>> {
    let (source, destination) = (source.into(), destination.into());
    log::info!("blocking route from {source} to {destination}");
    vm::network::linux::run_nft(&format!(
        "table inet {NFT_TABLE_NAME} {{
            chain postrouting {{
                type filter hook postrouting priority 0; policy accept;
                ip saddr {source} ip daddr {destination} drop;
            }}
        }}"
    ))
    .await
    .with_context(|| "Failed to set NFT ruleset that blocks traffic to relay")?;

    let drop_guard = scopeguard::guard((), |()| {
        log::info!("unblocking connection to relay");
        let mut cmd = std::process::Command::new("nft");
        cmd.args(["delete", "table", "inet", NFT_TABLE_NAME]);
        let output = cmd.output().unwrap();
        if !output.status.success() {
            panic!("{}", std::str::from_utf8(&output.stderr).unwrap());
        }
    });

    Ok(drop_guard)
}

/// Spawn a UDP socket that forwards packets between `destination` and anyone that connects to it.
///
/// NOTE: Doesn't work with multiple concurrent clients.
///
/// The proxy socket will be bound to [TEST_SUBNET_V4].
/// Returns a handle that will stop the proxy when dropped.
async fn spawn_udp_proxy(destination: SocketAddr, port: u16) -> anyhow::Result<AbortOnDrop<()>> {
    let socket = UdpSocket::bind((TEST_SUBNET_IPV4.ip(), port)).await?;
    log::info!("started UDP proxy to {destination} on port {port}");

    async fn proxy_task(destination: SocketAddr, socket: UdpSocket) -> anyhow::Result<()> {
        let mut buf = vec![0u8; 32 * 1024];

        let mut client = None;

        loop {
            let (n, from) = socket.recv_from(&mut buf[..]).await?;
            let data = &buf[..n];

            // If we receive a packet from the proxy destination,
            // forward it to the last known client.
            // Otherwise, forward it to the destination.
            let forward_to = if from.ip() == destination.ip() {
                let Some(client) = client else { continue };
                client
            } else {
                log::trace!("{from} connected to UDP proxy");
                client = Some(from);
                destination
            };

            socket
                .send_to(data, forward_to)
                .await
                .with_context(|| anyhow!("Failed to forward UDP packet to {forward_to}"))?;
        }
    }

    let task = task::spawn(async move {
        if let Err(e) = proxy_task(destination, socket).await {
            log::error!("UDP proxy task exited with error: {e:#}");
        } else {
            log::debug!("UDP proxy task exited gracefully");
        }
    });
    let on_drop = AbortOnDrop::new(task);

    Ok(on_drop)
}
