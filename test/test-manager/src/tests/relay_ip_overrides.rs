use super::{
    helpers::{self, AbortOnDrop},
    TestContext,
};
use crate::vm;
use anyhow::{anyhow, bail, ensure, Context};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    location::CountryCode,
    relay_constraints::{
        Constraint, GeographicLocationConstraint, ObfuscationSettings, RelayConstraints,
        RelayOverride, SelectedObfuscation, WireguardConstraints,
    },
    relay_list::RelayEndpointData,
};
use pnet_packet::PrimitiveValues;
use std::net::IpAddr;
use std::net::{Ipv4Addr, SocketAddr};
use test_macro::test_function;
use test_rpc::ServiceClient;
use tokio::{net::UdpSocket, task};

/// Test that IP overrides work for wireguard relays by:
/// - Picking an arbitrary wireguard relay.
/// - Block the VM from communicating with the relays IP address.
/// - Set up a UDP proxy on the host machine and override the relay IP with the host IP
#[cfg(target_os = "linux")] // the test requires nftables
#[test_function]
pub async fn test_wireguard_ip_override(
    _ctx: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
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

    let wg_port: u16 = 51800;
    let country = CountryCode::from("se");

    // pick any wg relay to use with the test
    log::info!("looking for an appropriate relay");
    let relays = mullvad_client.get_relay_locations().await?;
    let relays = relays
        .lookup_country(country.clone())
        .ok_or(anyhow!("Sweden doesn't appear to exist. Oh dear."))?;
    let (wg_relay, _endpoint) = relays
        .cities
        .iter()
        .flat_map(|city| &city.relays)
        .find_map(|relay| match &relay.endpoint_data {
            RelayEndpointData::Wireguard(data) => Some((relay, data)),
            _ => None,
        })
        .ok_or(anyhow!("No wireguard relays in Sweden?!"))?;

    let relay_ip = wg_relay.ipv4_addr_in;
    let hostname = wg_relay.hostname.clone();
    let city = wg_relay.location.as_ref().unwrap().city_code.clone();

    log::info!("using {hostname} ({relay_ip})");

    // constrain client to only use that relay
    let constraints = RelayConstraints {
        location: Constraint::Only(
            GeographicLocationConstraint::Hostname(country, city, hostname).into(),
        ),
        wireguard_constraints: WireguardConstraints {
            port: wg_port.into(),
            use_multihop: false,
            ..Default::default()
        },
        ..Default::default()
    };

    mullvad_client
        .set_relay_settings(constraints.into())
        .await
        .with_context(|| "Failed to set relay constraints")?;

    log::info!("connecting to selected relay");
    helpers::connect_and_wait(&mut mullvad_client).await?;

    log::info!("checking that the connection works");
    let _ = helpers::geoip_lookup_with_retries(&rpc).await?;

    log::info!("blocking connection to relay from guest");
    const NFT_TABLE_NAME: &str = "relay_override_test";
    vm::network::linux::run_nft(&format!(
        "table inet {NFT_TABLE_NAME} {{
            chain postrouting {{
                type filter hook postrouting priority 0; policy accept;
                ip saddr {guest_ip} ip daddr {relay_ip} drop;
            }}
        }}"
    ))
    .await
    .with_context(|| "Failed to set NFT ruleset that blocks traffic to relay")?;

    let _remove_nft_rule_on_drop = scopeguard::guard((), |()| {
        log::info!("unblocking connection to relay");
        let mut cmd = std::process::Command::new("nft");
        cmd.args(["delete", "table", "inet", NFT_TABLE_NAME]);
        let output = cmd.output().unwrap();
        if !output.status.success() {
            panic!("{}", std::str::from_utf8(&output.stderr).unwrap());
        }
    });

    log::info!("checking that the connection does not work with nft rule");
    // FIXME: this fails because of rpc timeouts, which is sort of fine but not ideal
    ensure!(
        helpers::geoip_lookup_with_retries(&rpc).await.is_err(),
        "Assert that relay is blocked by firewall rule"
    );

    log::info!("spawning relay udp proxy");
    let (_proxy_abort_handle, _) =
        spawn_udp_proxy(SocketAddr::new(relay_ip.into(), wg_port), Some(wg_port))
            .await
            .with_context(|| "Failed to spawn UDP proxy")?;

    log::info!("adding proxy to relay ip overrides");
    // TODO: find a better way of getting the gateway ip
    let (a, b, c, _) = guest_ip.to_primitive_values();
    let proxy_ip = Ipv4Addr::new(a, b, c, 1);
    mullvad_client
        .set_relay_override(RelayOverride {
            hostname: wg_relay.hostname.clone(),
            ipv4_addr_in: Some(proxy_ip),
            ipv6_addr_in: None,
        })
        .await?;

    log::info!("checking that the connection works again with the added overrides");
    let _ = helpers::geoip_lookup_with_retries(&rpc)
        .await
        .with_context(|| "Can't access internet through relay ip override")?;

    Ok(())
}

/// Spawn a UPD socket that forwards packets between `destination` and anyone that connects to it.
///
/// NOTE: Doesn't work with multiple concurrent clients.
async fn spawn_udp_proxy(
    destination: SocketAddr,
    port: Option<u16>,
) -> anyhow::Result<(AbortOnDrop<()>, u16)> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port.unwrap_or(0))).await?;
    let bind_port = socket.local_addr()?.port();

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
                let Some(client) = client else {
                    log::warn!("Proxy destination is talking to us, aaaah");
                    continue;
                };

                client
            } else {
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

    Ok((on_drop, bind_port))
}
