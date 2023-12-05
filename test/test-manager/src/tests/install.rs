use super::helpers::{connect_and_wait, get_package_desc};
use super::{Error, TestContext};

use super::config::TEST_CONFIG;
use crate::tests::helpers::{wait_for_tunnel_state, Pinger};
use mullvad_management_interface::{types, ManagementServiceClient};
use std::{collections::HashMap, net::ToSocketAddrs, time::Duration};
use test_macro::test_function;
use test_rpc::meta::Os;
use test_rpc::{mullvad_daemon::ServiceStatus, ServiceClient};

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

    // TODO: Update this to `rpc.exec("mullvad", ["debug", "block-connection"])` when 2023.6 is released.
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
    let pinger = Pinger::start(&rpc).await;

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
    let guest_ip = pinger.guest_ip;
    let monitor_result = pinger.stop().await.unwrap();
    assert_eq!(
        monitor_result.packets.len(),
        0,
        "observed unexpected packets from {guest_ip}"
    );

    // NOTE: Need to create a new `mullvad_client` here after the restart
    // otherwise we *probably* can't communicate with the daemon.
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

/// Install the multiple times starting from a connected state with auto-connect
/// disabled, failing if the app starts in a disconnected state.
///
/// This test is supposed to guard against regressions to this fix included in
/// the 2021.3-beta1 release:
/// https://github.com/mullvad/mullvadvpn-app/blob/main/CHANGELOG.md#security-10
#[test_function(priority = -150)]
pub async fn test_installation_idempotency(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    // Connect to any relay
    connect_and_wait(&mut mullvad_client).await?;
    // Disable auto-connect
    mullvad_client
        .set_auto_connect(false)
        .await
        .expect("failed to enable auto-connect");
    // Start a tunnel monitor. No traffic should be observed going outside of
    // the tunnel during either installation process.
    let pinger = Pinger::start(&rpc).await;
    for _ in 1..=2 {
        // install package
        log::debug!("Installing new app");
        rpc.install_app(get_package_desc(&TEST_CONFIG.current_app_filename)?)
            .await?;
        // verify that daemon is running
        wait_for_tunnel_state(mullvad_client.clone(), |state| state.is_connected())
            .await
            .map_err(|err| {
                log::error!(
                    "App did not start in the expected `Connected` state after the installation process."
                );
                err
            })?;
        // Wait for an arbitrary amount of time. The point is that the pinger
        // should be able to ping while the newly installed app is running.
        if let Some(delay) = pinger.period().checked_mul(3) {
            tokio::time::sleep(delay).await;
        }
    }

    // Make sure that no traffic leak occured during any installation process.
    let guest_ip = pinger.guest_ip;
    let monitor_result = pinger.stop().await.unwrap();
    assert_eq!(
        monitor_result.packets.len(),
        0,
        "observed unexpected packets from {guest_ip}"
    );

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
