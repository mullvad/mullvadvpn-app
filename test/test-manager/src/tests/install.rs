use super::helpers::{get_package_desc, ping_with_timeout, AbortOnDrop};
use super::{Error, TestContext};

use super::config::TEST_CONFIG;
use crate::network_monitor::{start_packet_monitor, MonitorOptions};
use mullvad_management_interface::types;
use std::{
    collections::HashMap,
    net::{SocketAddr, ToSocketAddrs},
    time::Duration,
};
use test_macro::test_function;
use test_rpc::meta::Os;
use test_rpc::{mullvad_daemon::ServiceStatus, Interface, ServiceClient};

/// Install the last stable version of the app and verify that it is running.
#[test_function(priority = -200)]
pub async fn test_install_previous_app(_: TestContext, rpc: ServiceClient) -> Result<(), Error> {
    // verify that daemon is not already running
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::NotRunning {
        return Err(Error::DaemonRunning);
    }

    // install package
    log::debug!("Installing old app");
    rpc.install_app(get_package_desc(&TEST_CONFIG.previous_app_filename)?)
        .await?;

    // verify that daemon is running
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::Running {
        return Err(Error::DaemonNotRunning);
    }

    replace_openvpn_cert(&rpc).await?;

    // Override env vars
    rpc.set_daemon_environment(get_app_env()).await?;

    Ok(())
}

/// Upgrade to the "version under test". This test fails if:
///
/// * Leaks (TCP/UDP/ICMP) to a single public IP address are successfully produced during the
///   upgrade.
/// * The installer does not successfully complete.
/// * The VPN service is not running after the upgrade.
#[test_function(priority = -190)]
pub async fn test_upgrade_app(ctx: TestContext, rpc: ServiceClient) -> Result<(), Error> {
    let inet_destination: SocketAddr = "1.1.1.1:1337".parse().unwrap();
    let bind_addr: SocketAddr = "0.0.0.0:0".parse().unwrap();

    // Verify that daemon is running
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::Running {
        return Err(Error::DaemonNotRunning);
    }

    super::account::clear_devices(&super::account::new_device_client().await)
        .await
        .expect("failed to clear devices");

    // Login to test preservation of device/account
    // TODO: Cannot do this now because overriding the API is impossible for releases
    //mullvad_client
    //    .login_account(TEST_CONFIG.account_number.clone())
    //    .await
    //    .expect("login failed");

    //
    // Start blocking
    //
    log::debug!("Entering blocking error state");

    rpc.exec("mullvad", ["relay", "set", "location", "xx"])
        .await
        .expect("Failed to set relay location");
    rpc.exec("mullvad", ["connect"])
        .await
        .expect("Failed to begin connecting");

    tokio::time::timeout(super::WAIT_FOR_TUNNEL_STATE_TIMEOUT, async {
        // use polling for sake of simplicity
        loop {
            const FIND_SLICE: &[u8] = b"Blocked:";
            let result = rpc
                .exec("mullvad", ["status"])
                .await
                .expect("Failed to poll tunnel status");
            if result
                .stdout
                .windows(FIND_SLICE.len())
                .any(|subslice| subslice == FIND_SLICE)
            {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
    .await
    .map_err(|_error| Error::DaemonError(String::from("Failed to enter blocking error state")))?;

    //
    // Begin monitoring outgoing traffic and pinging
    //

    let guest_ip = rpc
        .get_interface_ip(Interface::NonTunnel)
        .await
        .expect("failed to obtain tunnel IP");
    log::debug!("Guest IP: {guest_ip}");

    log::debug!("Monitoring outgoing traffic");

    let monitor = start_packet_monitor(
        move |packet| {
            // NOTE: Many packets will likely be observed for API traffic. Rather than filtering all
            // of those specifically, simply fail if our probes are observed.
            packet.source.ip() == guest_ip && packet.destination.ip() == inet_destination.ip()
        },
        MonitorOptions::default(),
    )
    .await;

    let ping_rpc = rpc.clone();
    let abort_on_drop = AbortOnDrop(tokio::spawn(async move {
        loop {
            let _ = ping_rpc.send_tcp(None, bind_addr, inet_destination).await;
            let _ = ping_rpc.send_udp(None, bind_addr, inet_destination).await;
            let _ = ping_with_timeout(&ping_rpc, inet_destination.ip(), None).await;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }));

    // install new package
    log::debug!("Installing new app");
    rpc.install_app(get_package_desc(&TEST_CONFIG.current_app_filename)?)
        .await?;

    // Give it some time to start
    tokio::time::sleep(Duration::from_secs(3)).await;

    // verify that daemon is running
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::Running {
        return Err(Error::DaemonNotRunning);
    }

    //
    // Check if any traffic was observed
    //
    drop(abort_on_drop);
    let monitor_result = monitor.into_result().await.unwrap();
    assert_eq!(
        monitor_result.packets.len(),
        0,
        "observed unexpected packets from {guest_ip}"
    );

    let mut mullvad_client = ctx.rpc_provider.new_client().await;

    // check if settings were (partially) preserved
    log::info!("Sanity checking settings");

    let settings = mullvad_client
        .get_settings(())
        .await
        .expect("failed to obtain settings")
        .into_inner();

    const EXPECTED_COUNTRY: &str = "xx";

    let relay_location_was_preserved = match &settings.relay_settings {
        Some(types::RelaySettings {
            endpoint:
                Some(types::relay_settings::Endpoint::Normal(types::NormalRelaySettings {
                    location:
                        Some(types::LocationConstraint {
                            r#type:
                                Some(types::location_constraint::Type::Location(
                                    types::GeographicLocationConstraint { country, .. },
                                )),
                        }),
                    ..
                })),
        }) => country == EXPECTED_COUNTRY,
        _ => false,
    };

    assert!(
        relay_location_was_preserved,
        "relay location was not preserved after upgrade. new settings: {:?}",
        settings,
    );

    // check if account history was preserved
    // TODO: Cannot check account history because overriding the API is impossible for releases
    /*
    let history = mullvad_client
        .get_account_history(())
        .await
        .expect("failed to obtain account history");
    assert_eq!(
        history.into_inner().token,
        Some(TEST_CONFIG.account_number.clone()),
        "lost account history"
    );
    */

    Ok(())
}

/// Uninstall the app version being tested. This verifies
/// that that the uninstaller works, and also that logs,
/// application files, system services are removed.
/// It also tests whether the device is removed from
/// the account.
///
/// # Limitations
///
/// Files due to Electron, temporary files, registry
/// values/keys, and device drivers are not guaranteed
/// to be deleted.
#[test_function(priority = -170, cleanup = false)]
pub async fn test_uninstall_app(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: mullvad_management_interface::ManagementServiceClient,
) -> Result<(), Error> {
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::Running {
        return Err(Error::DaemonNotRunning);
    }

    // Login to test preservation of device/account
    // TODO: Remove once we can login before upgrade above
    mullvad_client
        .login_account(TEST_CONFIG.account_number.clone())
        .await
        .expect("login failed");

    // save device to verify that uninstalling removes the device
    // we should still be logged in after upgrading
    let uninstalled_device = mullvad_client
        .get_device(())
        .await
        .expect("failed to get device data")
        .into_inner();
    let uninstalled_device = uninstalled_device
        .device
        .expect("missing account/device")
        .device
        .expect("missing device id")
        .id;

    log::debug!("Uninstalling app");
    rpc.uninstall_app(get_app_env()).await?;

    let app_traces = rpc
        .find_mullvad_app_traces()
        .await
        .expect("failed to obtain remaining Mullvad files");
    assert!(
        app_traces.is_empty(),
        "found files after uninstall: {app_traces:?}"
    );

    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::NotRunning {
        return Err(Error::DaemonRunning);
    }

    // verify that device was removed
    let devices =
        super::account::list_devices_with_retries(&super::account::new_device_client().await)
            .await
            .expect("failed to list devices");

    assert!(
        !devices.iter().any(|device| device.id == uninstalled_device),
        "device id {} still exists after uninstall",
        uninstalled_device,
    );

    Ok(())
}

/// Install the app cleanly, failing if the installer doesn't succeed
/// or if the VPN service is not running afterwards.
#[test_function(always_run = true, must_succeed = true, priority = -160)]
pub async fn test_install_new_app(_: TestContext, rpc: ServiceClient) -> Result<(), Error> {
    // verify that daemon is not already running
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::NotRunning {
        return Err(Error::DaemonRunning);
    }

    // install package
    log::debug!("Installing new app");
    rpc.install_app(get_package_desc(&TEST_CONFIG.current_app_filename)?)
        .await?;

    // verify that daemon is running
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::Running {
        return Err(Error::DaemonNotRunning);
    }

    // Set the log level to trace
    rpc.set_daemon_log_level(test_rpc::mullvad_daemon::Verbosity::Trace)
        .await?;

    replace_openvpn_cert(&rpc).await?;

    // Override env vars
    rpc.set_daemon_environment(get_app_env()).await?;

    Ok(())
}

fn get_app_env() -> HashMap<String, String> {
    let mut map = HashMap::new();

    let api_host = format!("api.{}", TEST_CONFIG.mullvad_host);
    let api_addr = format!("{api_host}:443")
        .to_socket_addrs()
        .expect("failed to resolve API host")
        .next()
        .unwrap();

    map.insert("MULLVAD_API_HOST".to_string(), api_host);
    map.insert("MULLVAD_API_ADDR".to_string(), api_addr.to_string());

    map
}

async fn replace_openvpn_cert(rpc: &ServiceClient) -> Result<(), Error> {
    use std::path::Path;

    const SOURCE_CERT_FILENAME: &str = "openvpn.ca.crt";
    const DEST_CERT_FILENAME: &str = "ca.crt";

    let dest_dir = match rpc.get_os().await.expect("failed to get OS") {
        Os::Windows => "C:\\Program Files\\Mullvad VPN\\resources",
        Os::Linux => "/opt/Mullvad VPN/resources",
        Os::Macos => "/Applications/Mullvad VPN.app/Contents/Resources",
    };

    rpc.copy_file(
        Path::new(&TEST_CONFIG.artifacts_dir)
            .join(SOURCE_CERT_FILENAME)
            .as_os_str()
            .to_string_lossy()
            .into_owned(),
        Path::new(dest_dir)
            .join(DEST_CERT_FILENAME)
            .as_os_str()
            .to_string_lossy()
            .into_owned(),
    )
    .await
    .map_err(Error::Rpc)
}
