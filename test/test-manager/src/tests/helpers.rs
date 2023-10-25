use super::{config::TEST_CONFIG, Error, PING_TIMEOUT, WAIT_FOR_TUNNEL_STATE_TIMEOUT};
use crate::network_monitor::{start_packet_monitor, MonitorOptions};
use futures::StreamExt;
use mullvad_management_interface::{types, ManagementServiceClient};
use mullvad_types::{
    relay_constraints::{
        BridgeState, Constraint, GeographicLocationConstraint, LocationConstraint,
        ObfuscationSettings, OpenVpnConstraints, RelayConstraintsUpdate, RelaySettingsUpdate,
        WireguardConstraints,
    },
    states::TunnelState,
};
use pnet_packet::ip::IpNextHeaderProtocols;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
    time::Duration,
};
use talpid_types::net::wireguard::{PeerConfig, PrivateKey, TunnelConfig};
use test_rpc::{package::Package, AmIMullvad, Interface, ServiceClient};
use tokio::time::timeout;

#[macro_export]
macro_rules! assert_tunnel_state {
    ($mullvad_client:expr, $pattern:pat) => {{
        let state = get_tunnel_state($mullvad_client).await;
        assert!(matches!(state, $pattern), "state: {:?}", state);
    }};
}

pub fn get_package_desc(name: &str) -> Result<Package, Error> {
    Ok(Package {
        path: Path::new(&TEST_CONFIG.artifacts_dir).join(name),
    })
}

/// Reboot the guest virtual machine.
///
/// # macOS
/// The tunnel must be reconfigured after the virtual machine is up,
/// or macOS refuses to assign an IP. The reasons for this are poorly understood.
pub async fn reboot(rpc: &mut ServiceClient) -> Result<(), Error> {
    rpc.reboot().await?;

    #[cfg(target_os = "macos")]
    crate::vm::network::macos::configure_tunnel()
        .await
        .map_err(|error| Error::Other(format!("Failed to recreate custom wg tun: {error}")))?;

    Ok(())
}

#[derive(Debug, Default)]
pub struct ProbeResult {
    tcp: usize,
    udp: usize,
    icmp: usize,
}

impl ProbeResult {
    pub fn all(&self) -> bool {
        self.tcp > 0 && self.udp > 0 && self.icmp > 0
    }

    pub fn none(&self) -> bool {
        !self.any()
    }

    pub fn any(&self) -> bool {
        self.tcp > 0 || self.udp > 0 || self.icmp > 0
    }
}

/// Return whether the guest exit IP is a Mullvad relay
pub async fn using_mullvad_exit(rpc: &ServiceClient) -> bool {
    log::info!("Test whether exit IP is a mullvad relay");
    geoip_lookup_with_retries(rpc)
        .await
        .unwrap()
        .mullvad_exit_ip
}

/// Sends a number of probes and returns the number of observed packets (UDP, TCP, or ICMP)
pub async fn send_guest_probes(
    rpc: ServiceClient,
    interface: Option<Interface>,
    destination: SocketAddr,
) -> Result<ProbeResult, Error> {
    let pktmon = start_packet_monitor(
        move |packet| packet.destination.ip() == destination.ip(),
        MonitorOptions {
            direction: Some(crate::network_monitor::Direction::In),
            timeout: Some(Duration::from_secs(3)),
            ..Default::default()
        },
    )
    .await;

    let send_handle = tokio::spawn(send_guest_probes_without_monitor(
        rpc,
        interface,
        destination,
    ));

    let monitor_result = pktmon.wait().await.unwrap();

    send_handle.abort();
    let _ = send_handle.await;

    let mut result = ProbeResult::default();

    for pkt in monitor_result.packets {
        match pkt.protocol {
            IpNextHeaderProtocols::Tcp => {
                result.tcp = result.tcp.saturating_add(1);
            }
            IpNextHeaderProtocols::Udp => {
                result.udp = result.udp.saturating_add(1);
            }
            IpNextHeaderProtocols::Icmp => {
                result.icmp = result.icmp.saturating_add(1);
            }
            _ => (),
        }
    }

    Ok(result)
}

/// Send one probe per transport protocol to `destination` without running a packet monitor
pub async fn send_guest_probes_without_monitor(
    rpc: ServiceClient,
    interface: Option<Interface>,
    destination: SocketAddr,
) {
    let bind_addr = if let Some(interface) = interface {
        SocketAddr::new(
            rpc.get_interface_ip(interface)
                .await
                .expect("failed to obtain interface IP"),
            0,
        )
    } else {
        "0.0.0.0:0".parse().unwrap()
    };

    let tcp_rpc = rpc.clone();
    let tcp_send = async move { tcp_rpc.send_tcp(interface, bind_addr, destination).await };
    let udp_rpc = rpc.clone();
    let udp_send = async move { udp_rpc.send_udp(interface, bind_addr, destination).await };
    let icmp = async move { ping_with_timeout(&rpc, destination.ip(), interface).await };
    let _ = tokio::join!(tcp_send, udp_send, icmp);
}

pub async fn ping_with_timeout(
    rpc: &ServiceClient,
    dest: IpAddr,
    interface: Option<Interface>,
) -> Result<(), Error> {
    timeout(PING_TIMEOUT, rpc.send_ping(interface, dest))
        .await
        .map_err(|_| Error::PingTimeout)?
        .map_err(Error::Rpc)
}

/// Try to connect to a Mullvad Tunnel.
///
/// If that fails for whatever reason, the Mullvad daemon ends up in the
/// [`TunnelState::Error`] state & [`Error::DaemonError`] is returned.
pub async fn connect_and_wait(mullvad_client: &mut ManagementServiceClient) -> Result<(), Error> {
    log::info!("Connecting");

    mullvad_client
        .connect_tunnel(())
        .await
        .map_err(|error| Error::DaemonError(format!("failed to begin connecting: {}", error)))?;

    let new_state = wait_for_tunnel_state(mullvad_client.clone(), |state| {
        matches!(
            state,
            TunnelState::Connected { .. } | TunnelState::Error(..)
        )
    })
    .await?;

    if matches!(new_state, TunnelState::Error(..)) {
        return Err(Error::DaemonError("daemon entered error state".to_string()));
    }

    log::info!("Connected");

    Ok(())
}

pub async fn disconnect_and_wait(
    mullvad_client: &mut ManagementServiceClient,
) -> Result<(), Error> {
    log::info!("Disconnecting");

    mullvad_client
        .disconnect_tunnel(())
        .await
        .map_err(|error| Error::DaemonError(format!("failed to begin disconnecting: {}", error)))?;
    wait_for_tunnel_state(mullvad_client.clone(), |state| {
        matches!(state, TunnelState::Disconnected)
    })
    .await?;

    log::info!("Disconnected");

    Ok(())
}

pub async fn wait_for_tunnel_state(
    mut rpc: mullvad_management_interface::ManagementServiceClient,
    accept_state_fn: impl Fn(&mullvad_types::states::TunnelState) -> bool,
) -> Result<mullvad_types::states::TunnelState, Error> {
    let events = rpc
        .events_listen(())
        .await
        .map_err(|status| Error::DaemonError(format!("Failed to get event stream: {}", status)))?;

    let state = mullvad_types::states::TunnelState::try_from(
        rpc.get_tunnel_state(())
            .await
            .map_err(|error| {
                Error::DaemonError(format!("Failed to get tunnel state: {:?}", error))
            })?
            .into_inner(),
    )
    .map_err(|error| Error::DaemonError(format!("Invalid tunnel state: {:?}", error)))?;
    if accept_state_fn(&state) {
        return Ok(state);
    }

    find_next_tunnel_state(events.into_inner(), accept_state_fn).await
}

pub async fn find_next_tunnel_state(
    stream: impl futures::Stream<Item = Result<types::DaemonEvent, tonic::Status>> + Unpin,
    accept_state_fn: impl Fn(&mullvad_types::states::TunnelState) -> bool,
) -> Result<mullvad_types::states::TunnelState, Error> {
    tokio::time::timeout(
        WAIT_FOR_TUNNEL_STATE_TIMEOUT,
        find_next_tunnel_state_inner(stream, accept_state_fn),
    )
    .await
    .map_err(|_error| Error::DaemonError(String::from("Tunnel event listener timed out")))?
}

async fn find_next_tunnel_state_inner(
    mut stream: impl futures::Stream<Item = Result<types::DaemonEvent, tonic::Status>> + Unpin,
    accept_state_fn: impl Fn(&mullvad_types::states::TunnelState) -> bool,
) -> Result<mullvad_types::states::TunnelState, Error> {
    loop {
        match stream.next().await {
            Some(Ok(event)) => match event.event.unwrap() {
                mullvad_management_interface::types::daemon_event::Event::TunnelState(
                    new_state,
                ) => {
                    let state = mullvad_types::states::TunnelState::try_from(new_state).map_err(
                        |error| Error::DaemonError(format!("Invalid tunnel state: {:?}", error)),
                    )?;
                    if accept_state_fn(&state) {
                        return Ok(state);
                    }
                }
                _ => continue,
            },
            Some(Err(status)) => {
                break Err(Error::DaemonError(format!(
                    "Failed to get next event: {}",
                    status
                )))
            }
            None => break Err(Error::DaemonError(String::from("Lost daemon event stream"))),
        }
    }
}

pub async fn geoip_lookup_with_retries(rpc: &ServiceClient) -> Result<AmIMullvad, Error> {
    const MAX_ATTEMPTS: usize = 5;
    const BEFORE_RETRY_DELAY: Duration = Duration::from_secs(2);

    let mut attempt = 0;

    loop {
        let result = rpc
            .geoip_lookup(TEST_CONFIG.mullvad_host.to_owned())
            .await
            .map_err(Error::GeoipError);

        attempt += 1;
        if result.is_ok() || attempt >= MAX_ATTEMPTS {
            return result;
        }

        tokio::time::sleep(BEFORE_RETRY_DELAY).await;
    }
}

pub struct AbortOnDrop<T>(Option<tokio::task::JoinHandle<T>>);

impl<T> AbortOnDrop<T> {
    pub fn new(inner: tokio::task::JoinHandle<T>) -> AbortOnDrop<T> {
        AbortOnDrop(Some(inner))
    }

    pub fn into_inner(mut self) -> tokio::task::JoinHandle<T> {
        self.0.take().unwrap()
    }
}

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        if let Some(task) = self.0.take() {
            task.abort();
        }
    }
}

/// Disconnect and reset all relay, bridge, and obfuscation settings.
///
/// See [`mullvad_types::relay_constraints::RelayConstraintsUpdate`] for details, but in short:
/// * Location constraint is [`Constraint::Any`]
/// * Provider constraint is [`Constraint::Any`]
/// * Ownership constraint is [`Constraint::Any`]
/// * The default tunnel protocol is [`talpid_types::net::TunnelType::Wireguard`]
/// * Wireguard settings are default (i.e. any port is used, no obfuscation ..)
///   see [`mullvad_types::relay_constraints::WireguardConstraints`] for details.
/// * OpenVPN settings are default (i.e. any port is used, no obfuscation ..)
///   see [`mullvad_types::relay_constraints::OpenVpnConstraints`] for details.
pub async fn reset_relay_settings(
    mullvad_client: &mut ManagementServiceClient,
) -> Result<(), Error> {
    disconnect_and_wait(mullvad_client).await?;

    let relay_settings = RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
        location: Some(Constraint::Any),
        tunnel_protocol: Some(Constraint::Any),
        openvpn_constraints: Some(OpenVpnConstraints::default()),
        wireguard_constraints: Some(WireguardConstraints::default()),
        providers: Some(Constraint::Any),
        ownership: Some(Constraint::Any),
    });
    let bridge_state = BridgeState::Auto;
    let obfuscation_settings = ObfuscationSettings::default();

    set_relay_settings(mullvad_client, relay_settings)
        .await
        .map_err(|error| {
            Error::DaemonError(format!("Failed to reset relay settings: {}", error))
        })?;

    mullvad_client
        .set_bridge_state(types::BridgeState::from(bridge_state))
        .await
        .map_err(|error| Error::DaemonError(format!("Failed to reset bridge mode: {}", error)))?;

    mullvad_client
        .set_obfuscation_settings(types::ObfuscationSettings::from(obfuscation_settings))
        .await
        .map_err(|error| Error::DaemonError(format!("Failed to reset obfuscation: {}", error)))?;

    Ok(())
}

pub async fn set_relay_settings(
    mullvad_client: &mut ManagementServiceClient,
    relay_settings_update: RelaySettingsUpdate,
) -> Result<(), Error> {
    let update = types::RelaySettingsUpdate::from(relay_settings_update);

    mullvad_client
        .set_relay_settings(update)
        .await
        .map_err(|error| Error::DaemonError(format!("Failed to set relay settings: {}", error)))?;
    Ok(())
}

pub async fn get_tunnel_state(mullvad_client: &mut ManagementServiceClient) -> TunnelState {
    let state = mullvad_client
        .get_tunnel_state(())
        .await
        .expect("mullvad RPC failed")
        .into_inner();
    TunnelState::try_from(state).unwrap()
}

/// Wait for the relay list to be updated, to make sure we have the overridden one.
/// Time out after a while.
pub async fn ensure_updated_relay_list(mullvad_client: &mut ManagementServiceClient) {
    let mut events = mullvad_client.events_listen(()).await.unwrap().into_inner();
    mullvad_client.update_relay_locations(()).await.unwrap();

    let wait_for_relay_update = async move {
        while let Some(Ok(event)) = events.next().await {
            if matches!(
                event,
                mullvad_management_interface::types::DaemonEvent {
                    event: Some(
                        mullvad_management_interface::types::daemon_event::Event::RelayList { .. }
                    )
                }
            ) {
                log::debug!("Received new relay list");
                break;
            }
        }
    };
    let _ = tokio::time::timeout(std::time::Duration::from_secs(3), wait_for_relay_update).await;
}

pub fn unreachable_wireguard_tunnel() -> talpid_types::net::wireguard::ConnectionConfig {
    talpid_types::net::wireguard::ConnectionConfig {
        tunnel: TunnelConfig {
            private_key: PrivateKey::new_from_random(),
            addresses: vec![IpAddr::V4(Ipv4Addr::new(10, 64, 10, 1))],
        },
        peer: PeerConfig {
            public_key: PrivateKey::new_from_random().public_key(),
            allowed_ips: vec![
                "0.0.0.0/0".parse().expect("Failed to parse ipv6 network"),
                "::0/0".parse().expect("Failed to parse ipv6 network"),
            ],
            endpoint: "1.3.3.7:1234".parse().unwrap(),
            psk: None,
        },
        exit_peer: None,
        ipv4_gateway: Ipv4Addr::new(10, 64, 10, 1),
        ipv6_gateway: None,
        #[cfg(target_os = "linux")]
        fwmark: None,
    }
}

/// Randomly select an entry and exit node from the daemon's relay list.
/// The exit node is distinct from the entry node.
///
/// * `mullvad_client` - An interface to the Mullvad daemon.
/// * `critera` - A function used to determine which relays to include in random selection.
pub async fn random_entry_and_exit<Filter>(
    mullvad_client: &mut ManagementServiceClient,
    criteria: Filter,
) -> Result<(types::Relay, types::Relay), Error>
where
    Filter: Fn(&types::Relay) -> bool,
{
    use itertools::Itertools;
    // Pluck the first 2 relays and return them as a tuple.
    // This will fail if there are less than 2 relays in the relay list.
    filter_relays(mullvad_client, criteria)
        .await?
        .into_iter()
        .next_tuple()
        .ok_or(Error::Other(
            "failed to randomly select two relays from daemon's relay list".to_string(),
        ))
}

/// Return a filtered version of the daemon's relay list.
///
/// * `mullvad_client` - An interface to the Mullvad daemon.
/// * `critera` - A function used to determine which relays to return.
pub async fn filter_relays<Filter>(
    mullvad_client: &mut ManagementServiceClient,
    criteria: Filter,
) -> Result<Vec<types::Relay>, Error>
where
    Filter: Fn(&types::Relay) -> bool,
{
    let relaylist = mullvad_client
        .get_relay_locations(())
        .await
        .map_err(|error| Error::DaemonError(format!("Failed to obtain relay list: {}", error)))?
        .into_inner();

    Ok(flatten_relaylist(relaylist)
        .into_iter()
        .filter(criteria)
        .collect())
}

/// Dig out the [`Relay`]s contained in a [`RelayList`].
pub fn flatten_relaylist(relays: types::RelayList) -> Vec<types::Relay> {
    relays
        .countries
        .iter()
        .flat_map(|country| country.cities.clone())
        .flat_map(|city| city.relays)
        .collect()
}

/// Convenience function for constructing a constraint from a given [`Relay`].
///
/// Returns an [`Option`] because a [`Relay`] is not guaranteed to be poplutaed with a location
/// vaule.
pub fn into_constraint(relay: &types::Relay) -> Option<Constraint<LocationConstraint>> {
    into_locationconstraint(relay).map(Constraint::Only)
}

/// Convenience function for constructing a location constraint from a given [`Relay`].
///
/// Returns an [`Option`] because a [`Relay`] is not guaranteed to be poplutaed with a location
/// vaule.
pub fn into_locationconstraint(relay: &types::Relay) -> Option<LocationConstraint> {
    relay
        .location
        .as_ref()
        .map(
            |types::Location {
                 country_code,
                 city_code,
                 ..
             }| {
                GeographicLocationConstraint::Hostname(
                    country_code.to_string(),
                    city_code.to_string(),
                    relay.hostname.to_string(),
                )
            },
        )
        .map(LocationConstraint::Location)
}
