use super::{config::TEST_CONFIG, Error, TestContext, WAIT_FOR_TUNNEL_STATE_TIMEOUT};
use crate::{
    mullvad_daemon::RpcClientProvider,
    network_monitor::{
        self, start_packet_monitor, MonitorOptions, MonitorUnexpectedlyStopped, PacketMonitor,
    },
    tests::{
        account::{clear_devices, new_device_client},
        helpers,
    },
};
use anyhow::{anyhow, bail, ensure, Context};
use futures::StreamExt;
use mullvad_management_interface::{client::DaemonEvent, MullvadProxyClient};
use mullvad_relay_selector::{
    query::{OpenVpnRelayQuery, RelayQuery, WireguardRelayQuery},
    GetRelay, RelaySelector, SelectorConfig, WireguardConfig,
};
use mullvad_types::{
    constraints::Constraint,
    custom_list::CustomList,
    relay_constraints::{
        GeographicLocationConstraint, LocationConstraint, RelayConstraints, RelaySettings,
    },
    relay_list::Relay,
    states::TunnelState,
};
use pcap::Direction;
use pnet_packet::ip::IpNextHeaderProtocols;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
    time::{Duration, Instant},
};
use talpid_types::net::wireguard::{PeerConfig, PrivateKey, TunnelConfig};
use test_rpc::{
    meta::Os, mullvad_daemon::ServiceStatus, package::Package, AmIMullvad, ServiceClient, SpawnOpts,
};
use tokio::time::sleep;

pub const THROTTLE_RETRY_DELAY: Duration = Duration::from_secs(120);

const CHECKER_FILENAME_WINDOWS: &str = "connection-checker.exe";
const CHECKER_FILENAME_UNIX: &str = "connection-checker";

const AM_I_MULLVAD_TIMEOUT_S: u64 = 10;
const LEAK_TIMEOUT_S: u64 = 1;

/// Timeout of [ConnCheckerHandle::check_connection].
const CONN_CHECKER_TIMEOUT: Duration = Duration::from_secs(
    AM_I_MULLVAD_TIMEOUT_S // https://am.i.mullvad.net timeout
    + LEAK_TIMEOUT_S // leak-tcp timeout
    + LEAK_TIMEOUT_S // leak-icmp timeout
    + 1, // plus some extra grace time
);

#[macro_export]
macro_rules! assert_tunnel_state {
    ($mullvad_client:expr, $pattern:pat) => {{
        let state = $mullvad_client.get_tunnel_state().await?;
        assert!(matches!(state, $pattern), "state: {:?}", state);
    }};
}

/// Install the app cleanly, failing if the installer doesn't succeed
/// or if the VPN service is not running afterwards.
pub async fn install_app(
    rpc: &ServiceClient,
    app_filename: &str,
    rpc_provider: &RpcClientProvider,
) -> anyhow::Result<MullvadProxyClient> {
    // install package
    log::info!("Installing app '{}'", app_filename);
    rpc.install_app(get_package_desc(app_filename)).await?;

    // verify that daemon is running
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::Running {
        bail!(Error::DaemonNotRunning);
    }

    // Set the log level to trace
    rpc.set_daemon_log_level(test_rpc::mullvad_daemon::Verbosity::Trace)
        .await?;

    replace_openvpn_certificate(rpc).await?;

    // Override env vars
    rpc.set_daemon_environment(get_app_env().await?).await?;

    // Wait for the relay list to be updated
    let mut mullvad_client = rpc_provider.new_client().await;
    helpers::ensure_updated_relay_list(&mut mullvad_client)
        .await
        .context("Failed to update relay list")?;
    Ok(mullvad_client)
}

/// Replace the OpenVPN CA certificate which is currently used by the installed Mullvad App.
/// This needs to be invoked after reach (re)installation to use the custom OpenVPN certificate.
async fn replace_openvpn_certificate(rpc: &ServiceClient) -> Result<(), Error> {
    const DEST_CERT_FILENAME: &str = "ca.crt";

    let dest_dir = match TEST_CONFIG.os {
        Os::Windows => "C:\\Program Files\\Mullvad VPN\\resources",
        Os::Linux => "/opt/Mullvad VPN/resources",
        Os::Macos => "/Applications/Mullvad VPN.app/Contents/Resources",
    };

    let dest = Path::new(dest_dir)
        .join(DEST_CERT_FILENAME)
        .as_os_str()
        .to_string_lossy()
        .into_owned();
    rpc.write_file(dest, TEST_CONFIG.openvpn_certificate.to_vec())
        .await
        .map_err(Error::Rpc)
}

pub fn get_package_desc(name: &str) -> Package {
    Package {
        path: Path::new(&TEST_CONFIG.artifacts_dir).join(name),
    }
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

/// Get VPN tunnel interface name
pub async fn get_tunnel_interface(client: &mut MullvadProxyClient) -> anyhow::Result<String> {
    match client.get_tunnel_state().await? {
        TunnelState::Connecting { endpoint, .. } | TunnelState::Connected { endpoint, .. } => {
            let Some(tunnel_interface) = endpoint.tunnel_interface else {
                bail!("Unknown tunnel interface");
            };
            Ok(tunnel_interface)
        }
        _ => bail!("Tunnel is not up"),
    }
}

/// Sends a number of probes and returns the number of observed packets (UDP, TCP, or ICMP)
pub async fn send_guest_probes(
    rpc: ServiceClient,
    interface: String,
    destination: SocketAddr,
) -> ProbeResult {
    const MONITOR_DURATION: Duration = Duration::from_secs(8);

    let pktmon = start_packet_monitor(
        move |packet| packet.destination.ip() == destination.ip(),
        MonitorOptions {
            direction: Some(network_monitor::Direction::In),
            timeout: Some(MONITOR_DURATION),
            ..Default::default()
        },
    )
    .await;

    let send_handle = tokio::spawn(send_guest_probes_without_monitor(
        rpc,
        Some(interface),
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

    result
}

/// Send one probe per transport protocol to `destination` without running a packet monitor
pub async fn send_guest_probes_without_monitor(
    rpc: ServiceClient,
    interface: Option<String>,
    destination: SocketAddr,
) {
    let bind_addr = if let Some(ref interface) = interface {
        SocketAddr::new(
            rpc.get_interface_ip(interface.clone())
                .await
                .expect("failed to obtain interface IP"),
            0,
        )
    } else {
        "0.0.0.0:0".parse().unwrap()
    };

    let tcp_rpc = rpc.clone();
    let tcp_interface = interface.clone();
    let tcp_send = async move {
        tcp_rpc
            .send_tcp(tcp_interface, bind_addr, destination)
            .await
    };
    let udp_rpc = rpc.clone();
    let udp_interface = interface.clone();
    let udp_send = async move {
        udp_rpc
            .send_udp(udp_interface, bind_addr, destination)
            .await
    };
    let icmp = async move { ping_with_timeout(&rpc, destination.ip(), interface).await };
    let _ = tokio::join!(tcp_send, udp_send, icmp);
}

pub async fn ping_with_timeout(
    rpc: &ServiceClient,
    dest: IpAddr,
    interface: Option<String>,
) -> Result<(), Error> {
    const DEFAULT_PING_SIZE: usize = 64;

    rpc.send_ping(dest, interface, DEFAULT_PING_SIZE)
        .await
        .map_err(Error::Rpc)
}

pub async fn ping_sized_with_timeout(
    rpc: &ServiceClient,
    dest: IpAddr,
    interface: Option<String>,
    size: usize,
) -> Result<(), Error> {
    rpc.send_ping(dest, interface, size)
        .await
        .map_err(Error::Rpc)
}

/// Return the first address that `host` resolves to
pub async fn resolve_hostname_with_retries(
    host: impl tokio::net::ToSocketAddrs,
) -> std::io::Result<SocketAddr> {
    const MAX_ATTEMPTS: usize = 10;
    const RETRY_DELAY: Duration = Duration::from_secs(5);

    let mut last_error = None;
    let mut attempt = 0;

    loop {
        if attempt >= MAX_ATTEMPTS {
            break Err(last_error.unwrap_or(std::io::Error::other("lookup timed out")));
        }
        attempt += 1;

        match tokio::net::lookup_host(&host).await {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    // done
                    break Ok(addr);
                }
            }
            Err(err) => last_error = Some(err),
        }

        tokio::time::sleep(RETRY_DELAY).await;
    }
}

/// Get the mac address (if any) of a network interface (on the test-manager machine).
#[cfg(target_os = "linux")] // not used on macos
pub fn get_interface_mac(interface: &str) -> anyhow::Result<Option<[u8; 6]>> {
    let addrs = nix::ifaddrs::getifaddrs().map_err(|error| {
        log::error!("Failed to obtain interfaces: {}", error);
        test_rpc::Error::Syscall
    })?;

    let mut interface_exists = false;

    let mac_addr = addrs
        .filter(|addr| addr.interface_name == interface)
        .find_map(|addr| {
            // sadly, the only way of distinguishing between "iface doesn't exist" and
            // "iface has no mac addr" is to check if the interface appears anywhere in the list.
            interface_exists = true;

            let addr = addr.address.as_ref()?;
            let link_addr = addr.as_link_addr()?;
            let mac_addr = link_addr.addr()?;
            Some(mac_addr)
        });

    if interface_exists {
        Ok(mac_addr)
    } else {
        bail!("Interface not found: {interface:?}")
    }
}

/// Get the index of a network interface (on the test-manager machine).
#[cfg(target_os = "linux")] // not used on macos
pub fn get_interface_index(interface: &str) -> anyhow::Result<std::ffi::c_uint> {
    use nix::errno::Errno;
    use std::ffi::CString;

    let interface = CString::new(interface).context(anyhow!(
        "Failed to turn interface name {interface:?} into cstr"
    ))?;

    match unsafe { libc::if_nametoindex(interface.as_ptr()) } {
        0 => {
            let err = Errno::last();
            Err(err).context("Failed to get interface index")
        }
        i => Ok(i),
    }
}

/// Log in and retry if it fails due to throttling
pub async fn login_with_retries(
    mullvad_client: &mut MullvadProxyClient,
) -> Result<(), mullvad_management_interface::Error> {
    log::debug!("Logging in/generating device");
    loop {
        match mullvad_client
            .login_account(TEST_CONFIG.account_number.clone())
            .await
        {
            Err(mullvad_management_interface::Error::Rpc(status))
                if status.message().to_uppercase().contains("THROTTLED") =>
            {
                // Work around throttling errors by sleeping
                log::debug!(
                    "Login failed due to throttling. Sleeping for {} seconds",
                    THROTTLE_RETRY_DELAY.as_secs()
                );

                tokio::time::sleep(THROTTLE_RETRY_DELAY).await;
            }
            Err(err) => break Err(err),
            Ok(_) => break Ok(()),
        }
    }
}

/// Ensure that the test runner is logged in to an account.
///
/// This will first check whether we are logged in. If not, it will also try to login
/// on your behalf. If this function returns without any errors, we are logged in to a valid
/// account.
pub async fn ensure_logged_in(mullvad_client: &mut MullvadProxyClient) -> anyhow::Result<()> {
    if !matches!(
        mullvad_client.update_device().await,
        Err(mullvad_management_interface::Error::DeviceNotFound)
    ) && mullvad_client.get_device().await?.is_logged_in()
    {
        return Ok(());
    }
    log::info!("Current device not logged in. Clearing devices and logging in.");
    // We are apparently not logged in already.. Try to log in.
    clear_devices(
        &new_device_client()
            .await
            .context("Failed to create device client")?,
    )
    .await
    .context("failed to clear devices")?;

    login_with_retries(mullvad_client)
        .await
        .context("Failed to log in")?;
    Ok(())
}

/// Try to connect to a Mullvad Tunnel.
///
/// # Returns
/// - `Result::Ok(new_state)` if the daemon successfully connected to a tunnel
/// - `Result::Err` if:
///     - The daemon failed to even begin connecting. Then [`Error::Rpc`] is returned.
///     - The daemon started to connect but ended up in the [`TunnelState::Error`] state. Then
///       [`Error::UnexpectedErrorState`] is returned
pub async fn connect_and_wait(
    mullvad_client: &mut MullvadProxyClient,
) -> Result<TunnelState, Error> {
    log::info!("Connecting");

    let initial_time = Instant::now();

    mullvad_client.connect_tunnel().await?;
    let new_state = wait_for_tunnel_state(mullvad_client.clone(), |state| {
        matches!(
            state,
            TunnelState::Connected { .. } | TunnelState::Error(..)
        )
    })
    .await?;

    if let TunnelState::Error(error_state) = new_state {
        return Err(Error::UnexpectedErrorState(error_state));
    }

    log::info!(
        "Connected after {} seconds",
        initial_time.elapsed().as_secs()
    );

    Ok(new_state)
}

pub async fn disconnect_and_wait(mullvad_client: &mut MullvadProxyClient) -> Result<(), Error> {
    log::trace!("Disconnecting");
    mullvad_client.disconnect_tunnel().await?;

    wait_for_tunnel_state(mullvad_client.clone(), |state| {
        matches!(state, TunnelState::Disconnected { .. })
    })
    .await?;

    log::trace!("Disconnected");

    Ok(())
}

pub async fn wait_for_tunnel_state(
    mut rpc: MullvadProxyClient,
    accept_state_fn: impl Fn(&mullvad_types::states::TunnelState) -> bool,
) -> Result<mullvad_types::states::TunnelState, Error> {
    let events = rpc
        .events_listen()
        .await
        .map_err(|status| Error::Daemon(format!("Failed to get event stream: {}", status)))?;

    let state = rpc
        .get_tunnel_state()
        .await
        .map_err(|error| Error::Daemon(format!("Failed to get tunnel state: {:?}", error)))?;

    if accept_state_fn(&state) {
        return Ok(state);
    }

    find_next_tunnel_state(events, accept_state_fn).await
}

pub async fn find_next_tunnel_state(
    stream: impl futures::Stream<Item = Result<DaemonEvent, mullvad_management_interface::Error>>
        + Unpin,
    accept_state_fn: impl Fn(&mullvad_types::states::TunnelState) -> bool,
) -> Result<mullvad_types::states::TunnelState, Error> {
    tokio::time::timeout(
        WAIT_FOR_TUNNEL_STATE_TIMEOUT,
        find_daemon_event(stream, |daemon_event| match daemon_event {
            DaemonEvent::TunnelState(state) if accept_state_fn(&state) => Some(state),
            _ => None,
        }),
    )
    .await
    .map_err(|_error| Error::Daemon(String::from("Tunnel event listener timed out")))?
}

pub async fn find_daemon_event<Accept, AcceptedEvent>(
    mut event_stream: impl futures::Stream<Item = Result<DaemonEvent, mullvad_management_interface::Error>>
        + Unpin,
    accept_event: Accept,
) -> Result<AcceptedEvent, Error>
where
    Accept: Fn(DaemonEvent) -> Option<AcceptedEvent>,
{
    loop {
        match event_stream.next().await {
            Some(Ok(daemon_event)) => match accept_event(daemon_event) {
                Some(accepted_event) => break Ok(accepted_event),
                None => continue,
            },
            Some(Err(status)) => {
                break Err(Error::Daemon(format!(
                    "Failed to get next event: {}",
                    status
                )));
            }
            None => break Err(Error::Daemon(String::from("Lost daemon event stream"))),
        }
    }
}

/// Set environment variables specified by `env` and restart the Mullvad daemon.
/// Returns a new [rpc client][`MullvadProxyClient`], since the old client *probably*
/// can't communicate with the new daemon.
///
/// # Note
/// This is just a thin wrapper around [`ServiceClient::set_daemon_environment`] which also
/// invalidates the old [`MullvadProxyClient`].
pub async fn restart_daemon_with<K, V, Env>(
    rpc: &ServiceClient,
    test_context: &TestContext,
    _: MullvadProxyClient, // Just consume the old proxy client
    env: Env,
) -> Result<MullvadProxyClient, Error>
where
    Env: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    rpc.set_daemon_environment(env).await?;
    // Need to create a new `mullvad_client` here after the restart
    // otherwise we *probably* can't communicate with the daemon.
    Ok(test_context.rpc_provider.new_client().await)
}

pub async fn geoip_lookup_with_retries(rpc: &ServiceClient) -> Result<AmIMullvad, Error> {
    const MAX_ATTEMPTS: usize = 5;
    const BEFORE_RETRY_DELAY: Duration = Duration::from_secs(2);

    let mut attempt = 0;

    loop {
        let result = rpc
            .geoip_lookup(TEST_CONFIG.mullvad_host.to_owned())
            .await
            .map_err(Error::GeoipLookup);

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
}

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        if let Some(task) = self.0.take() {
            task.abort();
        }
    }
}

/// Applies the given query to the daemon location selection. The query will be intersected with
/// the current location settings, so that the default location custom list is still used.
pub async fn apply_settings_from_relay_query(
    mullvad_client: &mut MullvadProxyClient,
    query: RelayQuery,
) -> Result<(), Error> {
    // To prevent overwriting default custom list location constraint, we make an intersection with
    // a query containing only the current location constraint
    let intersected_relay_query = intersect_with_current_location(mullvad_client, query)
        .await
        .map_err(|error| {
            Error::Other(format!(
                "Failed to join query with current daemon settings: {error}"
            ))
        })?;
    let (constraints, bridge_state, bridge_settings, obfuscation) =
        intersected_relay_query.into_settings();

    mullvad_client
        .set_relay_settings(constraints.into())
        .await
        .map_err(|error| Error::Daemon(format!("Failed to set relay settings: {}", error)))?;
    mullvad_client
        .set_bridge_state(bridge_state)
        .await
        .map_err(|error| Error::Daemon(format!("Failed to set bridge state: {}", error)))?;
    mullvad_client
        .set_bridge_settings(bridge_settings)
        .await
        .map_err(|error| Error::Daemon(format!("Failed to set bridge settings: {}", error)))?;
    mullvad_client
        .set_obfuscation_settings(obfuscation)
        .await
        .map_err(|error| Error::Daemon(format!("Failed to set obfuscation settings: {}", error)))
}

pub async fn set_custom_endpoint(
    mullvad_client: &mut MullvadProxyClient,
    custom_endpoint: mullvad_types::CustomTunnelEndpoint,
) -> Result<(), Error> {
    mullvad_client
        .set_relay_settings(RelaySettings::CustomTunnelEndpoint(custom_endpoint))
        .await
        .map_err(|error| Error::Daemon(format!("Failed to set relay settings: {}", error)))
}

pub async fn update_relay_constraints(
    mullvad_client: &mut MullvadProxyClient,
    fn_mut: impl FnOnce(&mut RelayConstraints),
) -> Result<(), Error> {
    let settings = mullvad_client
        .get_settings()
        .await
        .map_err(|error| Error::Daemon(format!("Failed to set relay settings: {}", error)))?;
    let RelaySettings::Normal(mut relay_constraints) = settings.relay_settings else {
        unimplemented!("Mutating custom endpoint not supported");
    };
    fn_mut(&mut relay_constraints);
    mullvad_client
        .set_relay_settings(RelaySettings::Normal(relay_constraints))
        .await?;
    Ok(())
}

/// Wait for the relay list to be updated, to make sure we have the overridden one.
/// Time out after a while.
pub async fn ensure_updated_relay_list(
    mullvad_client: &mut MullvadProxyClient,
) -> Result<(), mullvad_management_interface::Error> {
    let mut events = mullvad_client.events_listen().await?;
    mullvad_client.update_relay_locations().await?;

    let _ = tokio::time::timeout(std::time::Duration::from_secs(3), async move {
        while let Some(Ok(event)) = events.next().await {
            if matches!(event, DaemonEvent::RelayList(_)) {
                log::debug!("Received new relay list");
                break;
            }
        }
    })
    .await;

    Ok(())
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
            constant_packet_size: false,
        },
        exit_peer: None,
        ipv4_gateway: Ipv4Addr::new(10, 64, 10, 1),
        ipv6_gateway: None,
        #[cfg(target_os = "linux")]
        fwmark: None,
    }
}

/// Return the current `MULLVAD_API_HOST` et al.
///
/// # Note
/// This is independent of the running daemon's environment.
/// It is solely dependant on the current value of [`TEST_CONFIG`].
pub async fn get_app_env() -> anyhow::Result<HashMap<String, String>> {
    use mullvad_api::env;

    let api_host = format!("api.{}", TEST_CONFIG.mullvad_host);
    let api_host_with_port = format!("{api_host}:443");
    let api_addr = resolve_hostname_with_retries(api_host_with_port)
        .await
        .context("failed to resolve API host")?;

    Ok(HashMap::from_iter(vec![
        (env::API_HOST_VAR.to_string(), api_host),
        (env::API_ADDR_VAR.to_string(), api_addr.to_string()),
    ]))
}

/// Constrain the daemon to only select the relay compatible with `query` and the current relay
/// settings when establishing all future tunnels (until relay settings are updated, see [`set_relay_settings`]).
/// Returns the selected [`Relay`] for future reference.
///
/// # Note
/// This function does not handle bridges and multihop configurations (currently). There is no
/// particular reason for this other than it not being needed at the time, so feel free to extend
/// this function :).
pub async fn constrain_to_relay(
    mullvad_client: &mut MullvadProxyClient,
    query: RelayQuery,
) -> anyhow::Result<Relay> {
    let intersect_query = intersect_with_current_location(mullvad_client, query).await?;
    let (exit, relay_constraints) =
        get_single_relay_location_contraint(mullvad_client, intersect_query).await?;

    update_relay_constraints(mullvad_client, |current_constraints| {
        *current_constraints = relay_constraints
    })
    .await
    .unwrap();

    Ok(exit)
}

/// Intersects the given query with the current location constraints, to prevent accidentally
/// overwriting the default location custom list
async fn intersect_with_current_location(
    mullvad_client: &mut MullvadProxyClient,
    query: RelayQuery,
) -> anyhow::Result<RelayQuery> {
    let settings = mullvad_client
        .get_settings()
        .await
        .context("Failed to get settings")?;
    let RelaySettings::Normal(constraint) = settings.relay_settings else {
        unimplemented!("Setting locatio for a custom endpoint is not supported");
    };

    // Construct a relay query preverving only the information about the current location
    let current_location_query = RelayQuery::new(
        constraint.location,
        Constraint::Any,
        Constraint::Any,
        Constraint::Any,
        WireguardRelayQuery {
            entry_location: constraint.wireguard_constraints.entry_location,
            ..Default::default()
        },
        OpenVpnRelayQuery {
            bridge_settings: mullvad_relay_selector::query::BridgeQuery::Normal(
                settings.bridge_settings.normal,
            ),
            ..Default::default()
        },
    )?;
    use mullvad_types::Intersection;
    let intersect_query = query
        .intersection(current_location_query)
        .context("Relay query incompatible with default settings")?;
    Ok(intersect_query)
}

/// Get a query representing the current daemon settings
async fn get_query_from_current_settings(
    mullvad_client: &mut MullvadProxyClient,
) -> anyhow::Result<RelayQuery> {
    let settings = mullvad_client
        .get_settings()
        .await
        .context("Failed to get settings")?;
    let current_query =
        RelayQuery::try_from(settings).context("Failed to convert settings to relay query")?;
    Ok(current_query)
}

pub async fn get_all_pickable_relays(
    mullvad_client: &mut MullvadProxyClient,
) -> anyhow::Result<Vec<Relay>> {
    let settings = mullvad_client.get_settings().await?;
    let relay_list = mullvad_client.get_relay_locations().await?;
    let relays = mullvad_relay_selector::filter_matching_relay_list(
        &helpers::get_query_from_current_settings(mullvad_client).await?,
        &relay_list,
        &settings.custom_lists,
    );
    Ok(relays)
}

/// Selects a relay compatible with the given query and relay list from the client, and returns a
/// location constraint for only that relay, along with the relay itself.
///
/// # Note
/// This function does not handle bridges and multihop configurations (currently). There is no
/// particular reason for this other than it not being needed at the time, so feel free to extend
/// this function :).
async fn get_single_relay_location_contraint(
    mullvad_client: &mut MullvadProxyClient,
    query: RelayQuery,
) -> anyhow::Result<(Relay, RelayConstraints)> {
    /// Convert the result of invoking the relay selector to a relay constraint.
    fn convert_to_relay_constraints(
        query: RelayQuery,
        selected_relay: GetRelay,
    ) -> anyhow::Result<(Relay, RelayConstraints)> {
        match selected_relay {
            GetRelay::Wireguard {
                inner: WireguardConfig::Singlehop { exit },
                ..
            }
            | GetRelay::OpenVpn { exit, .. } => {
                let location = into_constraint(&exit);
                let (mut relay_constraints, ..) = query.into_settings();
                relay_constraints.location = location;
                Ok((exit, relay_constraints))
            }
            unsupported => bail!("Can not constrain to a {unsupported:?}"),
        }
    }
    let settings = mullvad_client.get_settings().await?;
    let relay_list = mullvad_client.get_relay_locations().await?;
    let relay_selector = get_daemon_relay_selector(&settings, relay_list);
    let relay = relay_selector.get_relay_by_query(query.clone())?;
    convert_to_relay_constraints(query, relay)
}

/// Get a mirror of the relay selector used by the daemon.
///
/// This can be used to query the relay selector without triggering a tunnel state change in the
/// daemon.
pub fn get_daemon_relay_selector(
    settings: &mullvad_types::settings::Settings,
    relay_list: mullvad_types::relay_list::RelayList,
) -> RelaySelector {
    RelaySelector::from_list(SelectorConfig::from_settings(settings), relay_list)
}

/// Convenience function for constructing a constraint from a given [`Relay`].
///
/// # Panics
///
/// The relay must have a location set.
pub fn into_constraint(relay: &Relay) -> Constraint<LocationConstraint> {
    let constraint = GeographicLocationConstraint::hostname(
        relay.location.country_code.clone(),
        relay.location.city_code.clone(),
        &relay.hostname,
    );

    Constraint::Only(LocationConstraint::Location(constraint))
}

/// Ping monitoring made easy!
///
/// Continuously ping some destination while monitoring to detect diverging
/// packets.
///
/// To customize [`Pinger`] before the pinging and network monitoring starts,
/// see [`PingerBuilder`]. Call [`start`](Pinger::start) to start pinging, and
/// [`stop`](Pinger::stop) to get the leak test results.
#[allow(dead_code)]
pub struct Pinger {
    // These values can be configured with [`PingerBuilder`].
    destination: SocketAddr,
    interval: tokio::time::Interval,
    // Run-time specific values
    pub guest_ip: IpAddr,
    ping_task: AbortOnDrop<tokio::task::JoinHandle<()>>,
    monitor: PacketMonitor,
}

impl Pinger {
    /// Create a [`Pinger`] with a default configuration.
    ///
    /// See [`PingerBuilder`] for details.
    pub async fn start(rpc: &test_rpc::ServiceClient) -> Pinger {
        let defaults = PingerBuilder::default();
        Self::start_with(defaults, rpc).await
    }

    /// Create a [`Pinger`] using the configuration of `builder`.
    ///
    /// See [`PingerBuilder`] for details on how to configure a [`Pinger`]
    /// before starting it.
    pub async fn start_with(builder: PingerBuilder, rpc: &test_rpc::ServiceClient) -> Pinger {
        // Get the associated IP address of the test runner on the default, non-tunnel interface.
        let guest_ip = obtain_guest_ip(rpc).await;
        log::debug!("Guest IP: {guest_ip}");

        // Start a network monitor
        log::debug!("Monitoring outgoing traffic");
        let monitor = start_packet_monitor(
            move |packet| {
                // NOTE: Many packets will likely be observed for API traffic. Rather than filtering
                // all of those specifically, simply fail if our probes are
                // observed.
                packet.source.ip() == guest_ip
                    && packet.destination.ip() == builder.destination.ip()
            },
            MonitorOptions::default(),
        )
        .await;

        // Start pinging
        //
        // Create some network activity for the network monitor to sniff.
        let ping_rpc = rpc.clone();
        let mut interval = tokio::time::interval(builder.interval.period());
        #[allow(clippy::async_yields_async)]
        let ping_task = AbortOnDrop::new(tokio::spawn(async move {
            loop {
                send_guest_probes_without_monitor(ping_rpc.clone(), None, builder.destination)
                    .await;
                interval.tick().await;
            }
        }));

        Pinger {
            destination: builder.destination,
            interval: builder.interval,
            guest_ip,
            ping_task,
            monitor,
        }
    }

    /// Stop pinging and extract the result of the network monitor.
    pub async fn stop(self) -> Result<network_monitor::MonitorResult, MonitorUnexpectedlyStopped> {
        // Abort the inner probe sender, which is accomplished by dropping the
        // join handle to the running task.
        drop(self.ping_task);
        self.monitor.into_result().await
    }

    /// Return the time period determining the cadence of pings that are sent.
    pub fn period(&self) -> tokio::time::Duration {
        self.interval.period()
    }
}

/// Returns the [`IpAddr`] of the default non-tunnel interface.
async fn obtain_guest_ip(rpc: &ServiceClient) -> IpAddr {
    let guest_iface = rpc
        .get_default_interface()
        .await
        .expect("failed to obtain default interface");
    rpc.get_interface_ip(guest_iface)
        .await
        .expect("failed to obtain non-tun IP")
}

/// Configure a [`Pinger`] before starting it.
pub struct PingerBuilder {
    destination: SocketAddr,
    interval: tokio::time::Interval,
}

#[allow(dead_code)]
impl PingerBuilder {
    /// Create a default [`PingerBuilder`].
    ///
    /// This is probably good enough for checking network traffic leaks when the
    /// test-runner is supposed to be blocked from sending or receiving *any*
    /// packets outside of localhost.
    pub fn default() -> PingerBuilder {
        PingerBuilder {
            destination: "1.1.1.1:1337".parse().unwrap(),
            interval: tokio::time::interval(Duration::from_secs(1)),
        }
    }

    /// Set the target to ping.
    pub fn destination(mut self, destination: SocketAddr) -> Self {
        self.destination = destination;
        self
    }

    /// How often a ping should be sent.
    pub fn interval(mut self, period: Duration) -> Self {
        self.interval = tokio::time::interval(period);
        self
    }
}

/// This helper spawns a separate process which checks if we are connected to Mullvad, and tries to
/// leak traffic outside the tunnel by sending TCP, UDP, and ICMP packets to [LEAK_DESTINATION].
pub struct ConnChecker {
    rpc: ServiceClient,
    mullvad_client: MullvadProxyClient,
    leak_destination: SocketAddr,

    /// Path to the process binary.
    executable_path: String,

    /// Whether the process should be split when spawned. Needed on Linux.
    split: bool,

    /// Some arbitrary payload
    payload: Option<String>,
}

pub struct ConnCheckerHandle<'a> {
    checker: &'a mut ConnChecker,

    /// ID of the spawned process.
    pid: u32,
}

pub struct ConnectionStatus {
    /// True if <https://am.i.mullvad.net/> reported we are connected.
    am_i_mullvad: bool,

    /// True if we sniffed TCP packets going outside the tunnel.
    leaked_tcp: bool,

    /// True if we sniffed UDP packets going outside the tunnel.
    leaked_udp: bool,

    /// True if we sniffed ICMP packets going outside the tunnel.
    leaked_icmp: bool,
}

impl ConnChecker {
    pub fn new(
        rpc: ServiceClient,
        mullvad_client: MullvadProxyClient,
        leak_destination: SocketAddr,
    ) -> Self {
        let artifacts_dir = &TEST_CONFIG.artifacts_dir;
        let executable_path = match TEST_CONFIG.os {
            Os::Linux | Os::Macos => format!("{artifacts_dir}/{CHECKER_FILENAME_UNIX}"),
            Os::Windows => format!("{artifacts_dir}\\{CHECKER_FILENAME_WINDOWS}"),
        };

        Self {
            rpc,
            mullvad_client,
            leak_destination,
            split: false,
            executable_path,
            payload: None,
        }
    }

    /// Set a custom magic payload that the connection checker binary should use when leak-testing.
    pub fn payload(&mut self, payload: impl Into<String>) {
        self.payload = Some(payload.into())
    }

    /// Spawn the connection checker process and return a handle to it.
    ///
    /// Dropping the handle will stop the process.
    /// **NOTE**: The handle must be dropped from a tokio runtime context.
    pub async fn spawn(&mut self) -> anyhow::Result<ConnCheckerHandle<'_>> {
        log::debug!("spawning connection checker");

        let opts = {
            let mut args = [
                "--interactive",
                "--timeout",
                &AM_I_MULLVAD_TIMEOUT_S.to_string(),
                // try to leak traffic to LEAK_DESTINATION
                "--leak",
                &self.leak_destination.to_string(),
                "--leak-timeout",
                &LEAK_TIMEOUT_S.to_string(),
                "--leak-tcp",
                "--leak-udp",
                "--leak-icmp",
                "--url",
                &format!("https://am.i.{}/json", TEST_CONFIG.mullvad_host),
            ]
            .map(String::from)
            .to_vec();

            if let Some(payload) = &self.payload {
                args.push("--payload".to_string());
                args.push(payload.clone());
            };

            SpawnOpts {
                attach_stdin: true,
                attach_stdout: true,
                args,
                ..SpawnOpts::new(&self.executable_path)
            }
        };

        let pid = self.rpc.spawn(opts).await?;

        if self.split && TEST_CONFIG.os == Os::Linux {
            self.mullvad_client
                .add_split_tunnel_process(pid as i32)
                .await?;
        }

        // TODO: The ST process monitor is a bit racy on macOS, such that processes aren't
        //       immediately recognized. This is a workaround until fixed.
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        Ok(ConnCheckerHandle { pid, checker: self })
    }

    /// Enable split tunneling for the connection checker.
    pub async fn split(&mut self) -> anyhow::Result<()> {
        log::debug!("enable split tunnel");
        self.split = true;

        match TEST_CONFIG.os {
            Os::Linux => { /* linux programs can't be split until they are spawned */ }
            Os::Macos | Os::Windows => {
                self.mullvad_client
                    .add_split_tunnel_app(&self.executable_path)
                    .await?;
                self.mullvad_client.set_split_tunnel_state(true).await?;
            }
        }

        Ok(())
    }

    /// Disable split tunneling for the connection checker.
    pub async fn unsplit(&mut self) -> anyhow::Result<()> {
        log::debug!("disable split tunnel");
        self.split = false;

        match TEST_CONFIG.os {
            Os::Linux => {}
            Os::Macos | Os::Windows => {
                self.mullvad_client.set_split_tunnel_state(false).await?;
                self.mullvad_client
                    .remove_split_tunnel_app(&self.executable_path)
                    .await?;
            }
        }

        Ok(())
    }
}

impl ConnCheckerHandle<'_> {
    pub async fn split(&mut self) -> anyhow::Result<()> {
        if TEST_CONFIG.os == Os::Linux {
            self.checker
                .mullvad_client
                .add_split_tunnel_process(self.pid as i32)
                .await?;
        }

        self.checker.split().await
    }

    pub async fn unsplit(&mut self) -> anyhow::Result<()> {
        if TEST_CONFIG.os == Os::Linux {
            self.checker
                .mullvad_client
                .remove_split_tunnel_process(self.pid as i32)
                .await?;
        }

        self.checker.unsplit().await
    }

    /// Assert that traffic is flowing through the Mullvad tunnel and that no packets are leaked.
    pub async fn assert_secure(&mut self) -> anyhow::Result<()> {
        log::info!("checking that connection is secure");
        let status = self.check_connection().await?;
        ensure!(status.am_i_mullvad);
        ensure!(!status.leaked_tcp);
        ensure!(!status.leaked_udp);
        ensure!(!status.leaked_icmp);

        Ok(())
    }

    /// Assert that traffic is NOT flowing through the Mullvad tunnel and that packets ARE leaked.
    pub async fn assert_insecure(&mut self) -> anyhow::Result<()> {
        log::info!("checking that connection is not secure");
        let status = self.check_connection().await?;
        ensure!(!status.am_i_mullvad);
        ensure!(status.leaked_tcp);
        ensure!(status.leaked_udp);
        ensure!(status.leaked_icmp);

        Ok(())
    }

    pub async fn check_connection(&mut self) -> anyhow::Result<ConnectionStatus> {
        // Monitor all packets going to LEAK_DESTINATION during the check.
        let leak_destination = self.checker.leak_destination;
        let monitor = start_packet_monitor(
            move |packet| packet.destination.ip() == leak_destination.ip(),
            MonitorOptions {
                direction: Some(Direction::In),
                ..MonitorOptions::default()
            },
        )
        .await;

        // Write a newline to the connection checker to prompt it to perform the check.
        self.checker
            .rpc
            .write_child_stdin(self.pid, "Say the line, Bart!\r\n".into())
            .await?;

        // The checker responds when the check is complete.
        let line = self.read_stdout_line().await?;

        let monitor_result = monitor
            .into_result()
            .await
            .map_err(|_e| anyhow!("Packet monitor unexpectedly stopped"))?;

        Ok(ConnectionStatus {
            am_i_mullvad: parse_am_i_mullvad(line)?,

            leaked_tcp: (monitor_result.packets.iter())
                .any(|pkt| pkt.protocol == IpNextHeaderProtocols::Tcp),

            leaked_udp: (monitor_result.packets.iter())
                .any(|pkt| pkt.protocol == IpNextHeaderProtocols::Udp),

            leaked_icmp: (monitor_result.packets.iter())
                .any(|pkt| pkt.protocol == IpNextHeaderProtocols::Icmp),
        })
    }

    /// Try to a single line of output from the spawned process
    async fn read_stdout_line(&mut self) -> anyhow::Result<String> {
        // Add a timeout to avoid waiting forever.
        tokio::time::timeout(CONN_CHECKER_TIMEOUT, async {
            let mut line = String::new();

            // tarpc doesn't support streams, so we poll the checker process in a loop instead
            loop {
                let Some(output) = self.checker.rpc.read_child_stdout(self.pid).await? else {
                    bail!("got EOF from connection checker process");
                };

                if output.is_empty() {
                    sleep(Duration::from_millis(500)).await;
                    continue;
                }

                line.push_str(&output);

                if line.contains('\n') {
                    log::info!("output from child process: {output:?}");
                    return Ok(line);
                }
            }
        })
        .await
        .with_context(|| "Timeout reading stdout from connection checker")?
    }
}

impl Drop for ConnCheckerHandle<'_> {
    fn drop(&mut self) {
        let rpc = self.checker.rpc.clone();
        let pid = self.pid;

        let Ok(runtime_handle) = tokio::runtime::Handle::try_current() else {
            log::error!("ConnCheckerHandle dropped outside of a tokio runtime.");
            return;
        };

        runtime_handle.spawn(async move {
            // Make sure child process is stopped when this handle is dropped.
            // Closing stdin does the trick.
            let _ = rpc.close_child_stdin(pid).await;
        });
    }
}

/// Parse output from connection-checker. Returns true if connected to Mullvad.
fn parse_am_i_mullvad(result: String) -> anyhow::Result<bool> {
    Ok(if result.contains("You are connected") {
        true
    } else if result.contains("You are not connected") {
        false
    } else {
        bail!("Unexpected output from connection-checker: {result:?}")
    })
}

/// Set the location to the given [`LocationConstraint`]. The same location constraint will be set
/// for the multihop entry and OpenVPN bridge location as well.
pub async fn set_location(
    mullvad_client: &mut MullvadProxyClient,
    location: impl Into<LocationConstraint>,
) -> anyhow::Result<()> {
    let location_constraint: LocationConstraint = location.into();
    let mut settings = mullvad_client
        .get_settings()
        .await
        .map_err(|error| Error::Daemon(format!("Failed to set relay settings: {}", error)))?;

    settings.bridge_settings.normal.location = Constraint::Only(location_constraint.clone());
    mullvad_client
        .set_bridge_settings(settings.bridge_settings)
        .await?;

    let RelaySettings::Normal(mut constraint) = settings.relay_settings else {
        unimplemented!("Setting locatio for a custom endpoint is not supported");
    };
    constraint.location = Constraint::Only(location_constraint.clone());
    constraint.wireguard_constraints.entry_location = Constraint::Only(location_constraint);
    mullvad_client
        .set_relay_settings(RelaySettings::Normal(constraint))
        .await?;
    Ok(())
}

/// Dig out a custom list from the daemon settings based on the custom list's name.
/// There should be an rpc for this.
pub async fn find_custom_list(
    rpc: &mut MullvadProxyClient,
    name: &str,
) -> anyhow::Result<CustomList> {
    rpc.get_settings()
        .await?
        .custom_lists
        .into_iter()
        .find(|list| list.name == name)
        .ok_or(anyhow!("List '{name}' not found"))
}
