#![cfg(target_os = "linux")]

use super::{
    helpers::{self, AbortOnDrop},
    TestContext,
};
use crate::{
    tests::config::TEST_CONFIG,
    vm::{self, network::linux::TEST_SUBNET},
};
use anyhow::{anyhow, bail, ensure, Context};
use futures::FutureExt;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    constraints::Constraint,
    location::CountryCode,
    relay_constraints::{
        BridgeConstraints, BridgeSettings, BridgeState, BridgeType, GeographicLocationConstraint,
        LocationConstraint, ObfuscationSettings, OpenVpnConstraints, RelayConstraints,
        RelayOverride, SelectedObfuscation, TransportPort, WireguardConstraints,
    },
    relay_list::RelayEndpointData,
};
use scopeguard::ScopeGuard;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use talpid_types::net::{TransportProtocol, TunnelType};
use test_macro::test_function;
use test_rpc::ServiceClient;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    select,
    task::{self, JoinSet},
};

const NFT_TABLE_NAME: &str = "relay_override_test";
const TUNNEL_PORT: u16 = 443;

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

    // pick any openvpn relay to use with the test
    let filter = |endpoint: &_| matches!(endpoint, RelayEndpointData::Wireguard(..));
    let (hostname, relay_ip) = constrain_to_a_relay(&mut mullvad_client, filter).await?;

    log::info!("connecting to selected relay");
    helpers::connect_and_wait(&mut mullvad_client).await?;

    log::info!("checking that the connection works");
    let _ = helpers::geoip_lookup_with_retries(&rpc).await?;

    log::info!("blocking connection to relay from guest");
    let _remove_nft_rule_on_drop = block_route(guest_ip, relay_ip).await?;

    log::info!("checking that the connection does not work while blocked");
    ensure!(
        helpers::geoip_lookup_with_retries(&rpc).await.is_err(),
        "Assert that relay is blocked by firewall rule"
    );

    let _proxy_abort_handle =
        spawn_udp_proxy(SocketAddr::new(relay_ip.into(), TUNNEL_PORT), TUNNEL_PORT)
            .await
            .with_context(|| "Failed to spawn UDP proxy")?;

    log::info!("adding proxy to relay ip overrides");
    mullvad_client
        .set_relay_override(RelayOverride {
            hostname,
            ipv4_addr_in: Some(TEST_CONFIG.host_bridge_ip),
            ipv6_addr_in: None,
        })
        .await?;

    log::info!("checking that the connection works again with the added overrides");
    let _ = helpers::geoip_lookup_with_retries(&rpc)
        .await
        .with_context(|| "Can't access internet through relay ip override")?;

    Ok(())
}

/// Test that IP overrides work for wireguard relays by:
/// - Picking an arbitrary OpenVPN relay.
/// - Block the VM from communicating with the relays IP address.
/// - Set up a TCP proxy on the host machine and override the relay IP with the host IP
#[test_function(target_os = "linux", target_os = "windows")]
pub async fn test_openvpn_ip_override(
    _ctx: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    let guest_interface = rpc.get_default_interface().await?;
    let IpAddr::V4(guest_ip) = rpc.get_interface_ip(guest_interface).await? else {
        bail!("Guests with IPv6 addresses are not supported.");
    };

    // pick any openvpn relay to use with the test
    let filter = |endpoint: &_| matches!(endpoint, RelayEndpointData::Openvpn);
    let (hostname, relay_ip) = constrain_to_a_relay(&mut mullvad_client, filter).await?;

    log::info!("connecting to selected relay");
    helpers::connect_and_wait(&mut mullvad_client).await?;

    log::info!("checking that the connection works");
    let _ = helpers::geoip_lookup_with_retries(&rpc).await?;

    log::info!("blocking connection to relay from guest");
    let _remove_nft_rule_on_drop = block_route(guest_ip, relay_ip).await?;

    log::info!("checking that the connection does not work while blocked");
    ensure!(
        helpers::geoip_lookup_with_retries(&rpc).await.is_err(),
        "Assert that relay is blocked by firewall rule"
    );

    let _proxy_abort_handle =
        spawn_tcp_proxy(SocketAddr::new(relay_ip.into(), TUNNEL_PORT), TUNNEL_PORT)
            .await
            .with_context(|| "Failed to spawn TCP proxy")?;

    log::info!("adding proxy to relay ip overrides");
    mullvad_client
        .set_relay_override(RelayOverride {
            hostname,
            ipv4_addr_in: Some(TEST_CONFIG.host_bridge_ip),
            ipv6_addr_in: None,
        })
        .await?;

    log::info!("checking that the connection works again with the added overrides");
    let _ = helpers::geoip_lookup_with_retries(&rpc)
        .await
        .with_context(|| "Can't access internet through relay ip override")?;

    Ok(())
}

/// Test that IP overrides work for bridge relays by:
/// - Picking an arbitrary bridge relay.
/// - Block the VM from communicating with the relays IP address.
/// - Set up shadowsocks proxies on the host machine and override the relay IP with the host IP
#[test_function(target_os = "linux", target_os = "windows")]
pub async fn test_bridge_ip_override(
    _ctx: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    let guest_interface = rpc.get_default_interface().await?;
    let IpAddr::V4(guest_ip) = rpc.get_interface_ip(guest_interface).await? else {
        bail!("Guests with IPv6 addresses are not supported.");
    };

    // pick any bridge relay to use with the test
    let relays = mullvad_client.get_relay_locations().await?;
    let filter = |endpoint: &_| matches!(endpoint, RelayEndpointData::Bridge);
    let (hostname, relay_ip, location) = pick_a_relay(&mut mullvad_client, filter).await?;

    // constrain client to only use this as a bridge
    let bridge_constraints = BridgeSettings {
        bridge_type: BridgeType::Normal,
        normal: BridgeConstraints {
            location: location.into(),
            ..Default::default()
        },
        ..Default::default()
    };

    let relay_constraints = RelayConstraints {
        tunnel_protocol: TunnelType::OpenVpn,
        ..Default::default()
    };

    mullvad_client
        .set_bridge_settings(bridge_constraints)
        .await
        .with_context(|| "Failed to set bridge constraints")?;
    mullvad_client
        .set_relay_settings(relay_constraints.into())
        .await
        .with_context(|| "Failed to set relay constraints")?;
    mullvad_client.set_bridge_state(BridgeState::On).await?;

    log::info!("connecting to selected relay");
    helpers::connect_and_wait(&mut mullvad_client).await?;

    log::info!("checking that the connection works");
    let _ = helpers::geoip_lookup_with_retries(&rpc).await?;

    log::info!("blocking connection to relay from guest");
    let _remove_nft_rule_on_drop = block_route(guest_ip, relay_ip).await?;

    log::info!("checking that the connection does not work while blocked");
    ensure!(
        helpers::geoip_lookup_with_retries(&rpc).await.is_err(),
        "Assert that relay is blocked by firewall rule"
    );

    log::info!("spawning shadowsocks proxies");
    let mut proxy_abort_handles = vec![];
    for shadowsocks in &relays.bridge.shadowsocks {
        let port = shadowsocks.port;
        proxy_abort_handles.push(spawn_udp_proxy((relay_ip, port).into(), port).await?);
        proxy_abort_handles.push(spawn_tcp_proxy((relay_ip, port).into(), port).await?);
    }

    log::info!("adding proxy to relay ip overrides");
    mullvad_client
        .set_relay_override(RelayOverride {
            hostname,
            ipv4_addr_in: Some(TEST_CONFIG.host_bridge_ip),
            ipv6_addr_in: None,
        })
        .await?;

    log::info!("checking that the connection works again with the added overrides");
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

/// Find a single arbitrary relay matching the given filter
///
/// Returns the hostname and IP of the relay.
async fn pick_a_relay(
    mullvad_client: &mut MullvadProxyClient,
    endpoint_filter: impl Fn(&RelayEndpointData) -> bool,
) -> anyhow::Result<(String, Ipv4Addr, LocationConstraint)> {
    let country = CountryCode::from("se");

    log::info!("looking for an appropriate relay");
    let relays = mullvad_client.get_relay_locations().await?;

    let relays = relays
        .lookup_country(country.clone())
        .ok_or(anyhow!("Sweden doesn't appear to exist. Oh dear."))?;

    let relay = relays
        .cities
        .iter()
        .flat_map(|city| &city.relays)
        .find(|relay| endpoint_filter(&relay.endpoint_data))
        .ok_or(anyhow!("No relays found matching the filter"))?;

    let relay_ip = relay.ipv4_addr_in;
    let hostname = relay.hostname.clone();
    let city = relay.location.city_code.clone();

    log::info!("selected {hostname} ({relay_ip})");
    let location = GeographicLocationConstraint::Hostname(country, city, hostname.clone()).into();

    Ok((hostname, relay_ip, location))
}

/// Find a single arbitrary relay matching the given filter and constrain the client to only use
/// that relay, and to only connect on [TUNNEL_PORT].
///
/// Returns the hostname and IP of the relay.
async fn constrain_to_a_relay(
    mullvad_client: &mut MullvadProxyClient,
    endpoint_filter: impl Fn(&RelayEndpointData) -> bool,
) -> anyhow::Result<(String, Ipv4Addr)> {
    let (hostname, relay_ip, location) = pick_a_relay(mullvad_client, endpoint_filter).await?;

    // constrain client to only use this relay
    let constraints = RelayConstraints {
        location: Constraint::Only(location),
        openvpn_constraints: OpenVpnConstraints {
            port: TransportPort {
                protocol: TransportProtocol::Tcp,
                port: TUNNEL_PORT.into(),
            }
            .into(),
        },
        wireguard_constraints: WireguardConstraints {
            port: TUNNEL_PORT.into(),
            use_multihop: false,
            ..Default::default()
        },
        ..Default::default()
    };

    mullvad_client
        .set_relay_settings(constraints.into())
        .await
        .with_context(|| "Failed to set relay constraints")?;

    Ok((hostname, relay_ip))
}

/// Spawn a TCP socket that forwards packets between `destination` and anyone that connects to it.
///
/// Returns a handle that will stop the proxy when dropped.
async fn spawn_tcp_proxy(destination: SocketAddr, port: u16) -> anyhow::Result<AbortOnDrop<()>> {
    let socket = TcpListener::bind((TEST_SUBNET.ip(), port)).await?;
    log::info!("started TCP proxy to {destination} on port {port}");

    async fn client_task(destination: SocketAddr, mut client: TcpStream) -> anyhow::Result<()> {
        let mut client_buf = vec![0u8; 32 * 1024];
        let mut server_buf = vec![0u8; 32 * 1024];
        let mut server = TcpStream::connect(destination).await?;

        loop {
            select! {
                n = client.read(&mut client_buf[..]).fuse() => {
                    let data = &client_buf[..n?];
                    server.write_all(data).await?;
                }
                n = server.read(&mut server_buf[..]).fuse() => {
                    let data = &server_buf[..n?];
                    client.write_all(data).await?;
                }
            }
        }
    }

    async fn listener_task(destination: SocketAddr, listener: TcpListener) -> anyhow::Result<()> {
        // put client tasks in a JoinSet so that they are aborted if dropped
        let mut client_tasks = JoinSet::new();
        loop {
            let (stream, from) = listener.accept().await?;
            log::trace!("{from} connected to TCP proxy");
            client_tasks.spawn(async move {
                if let Err(e) = client_task(destination, stream).await {
                    log::warn!("disconnecting TCP proxy client {from} because of error: {e:#}");
                }
            });
        }
    }

    let task = task::spawn(async move {
        if let Err(e) = listener_task(destination, socket).await {
            log::error!("UDP proxy task exited with error: {e:#}");
        } else {
            log::debug!("UDP proxy task exited gracefully");
        }
    });

    Ok(AbortOnDrop::new(task))
}

/// Spawn a UPD socket that forwards packets between `destination` and anyone that connects to it.
///
/// NOTE: Doesn't work with multiple concurrent clients.
///
/// Returns a handle that will stop the proxy when dropped.
async fn spawn_udp_proxy(destination: SocketAddr, port: u16) -> anyhow::Result<AbortOnDrop<()>> {
    let socket = UdpSocket::bind((TEST_SUBNET.ip(), port)).await?;
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
