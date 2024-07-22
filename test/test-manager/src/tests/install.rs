use anyhow::{bail, Context};
use std::time::Duration;

use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{constraints::Constraint, relay_constraints};
use test_macro::test_function;
use test_rpc::{meta::Os, mullvad_daemon::ServiceStatus, ServiceClient};

use super::{
    config::TEST_CONFIG,
    helpers::{connect_and_wait, get_app_env, get_package_desc, wait_for_tunnel_state, Pinger},
    Error, TestContext,
};

/// Install the last stable version of the app and verify that it is running.
#[test_function(priority = -200)]
pub async fn test_install_previous_app(_: TestContext, rpc: ServiceClient) -> anyhow::Result<()> {
    let Some(previous_app) = TEST_CONFIG.app_package_to_upgrade_from_filename.as_ref() else {
        bail!("Missing previous app version");
    };

    // verify that daemon is not already running
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::NotRunning {
        bail!(Error::DaemonRunning);
    }

    // install package
    log::debug!("Installing old app");
    rpc.install_app(get_package_desc(previous_app)?).await?;

    // verify that daemon is running
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::Running {
        bail!(Error::DaemonNotRunning);
    }

    replace_openvpn_cert(&rpc).await?;

    // Override env vars
    rpc.set_daemon_environment(get_app_env().await?).await?;

    Ok(())
}

/// Upgrade to the "version under test". This test fails if:
///
/// * Leaks (TCP/UDP/ICMP) to a single public IP address are successfully produced during the
///   upgrade.
/// * The installer does not successfully complete.
/// * The VPN service is not running after the upgrade.
#[test_function(priority = -190)]
pub async fn test_upgrade_app(ctx: TestContext, rpc: ServiceClient) -> anyhow::Result<()> {
    anyhow::ensure!(
        TEST_CONFIG.app_package_to_upgrade_from_filename.is_some(),
        "Missing previous app version"
    );

    // Verify that daemon is running
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::Running {
        bail!(Error::DaemonNotRunning);
    }

    let device_client = super::account::new_device_client()
        .await
        .context("Failed to create device client")?;
    super::account::clear_devices(&device_client)
        .await
        .context("failed to clear devices")?;

    // Login to test preservation of device/account
    // TODO: Cannot do this now because overriding the API is impossible for releases
    // mullvad_client
    //    .login_account(TEST_CONFIG.account_number.clone())
    //    .await
    //    .expect("login failed");

    // Start blocking
    //
    log::debug!("Entering blocking error state");

    rpc.exec("mullvad", ["debug", "block-connection"])
        .await
        .context("Failed to set relay location")?;
    rpc.exec("mullvad", ["connect"])
        .await
        .context("Failed to begin connecting")?;

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
    .map_err(|_error| Error::Daemon(String::from("Failed to enter blocking error state")))?;

    // Begin monitoring outgoing traffic and pinging
    //
    let pinger = Pinger::start(&rpc).await;

    // install new package
    log::debug!("Installing new app");
    rpc.install_app(get_package_desc(&TEST_CONFIG.app_package_filename)?)
        .await?;

    // Give it some time to start
    tokio::time::sleep(Duration::from_secs(3)).await;

    // verify that daemon is running
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::Running {
        bail!(Error::DaemonNotRunning);
    }

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
        .get_settings()
        .await
        .context("failed to obtain settings")?;

    const EXPECTED_COUNTRY: &str = "xx";

    let relay_location_was_preserved = match &settings.relay_settings {
        relay_constraints::RelaySettings::Normal(relay_constraints::RelayConstraints {
            location:
                Constraint::Only(relay_constraints::LocationConstraint::Location(
                    relay_constraints::GeographicLocationConstraint::Country(country),
                )),
            ..
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
    // let history = mullvad_client
    // .get_account_history(())
    // .await
    // .expect("failed to obtain account history");
    // assert_eq!(
    // history.into_inner().token,
    // Some(TEST_CONFIG.account_number.clone()),
    // "lost account history"
    // );

    Ok(())
}

/// Install the app cleanly, failing if the installer doesn't succeed
/// or if the VPN service is not running afterwards.
#[test_function(always_run = true, must_succeed = true, priority = -170)]
pub async fn test_install_new_app(_: TestContext, rpc: ServiceClient) -> anyhow::Result<()> {
    // verify that daemon is not already running
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::NotRunning {
        rpc.stop_mullvad_daemon().await.unwrap();
    }

    // install package
    log::debug!("Installing new app");
    rpc.install_app(get_package_desc(&TEST_CONFIG.app_package_filename)?)
        .await?;

    // verify that daemon is running
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::Running {
        bail!(Error::DaemonNotRunning);
    }

    // Set the log level to trace
    rpc.set_daemon_log_level(test_rpc::mullvad_daemon::Verbosity::Trace)
        .await?;

    replace_openvpn_cert(&rpc).await?;

    // Override env vars
    rpc.set_daemon_environment(get_app_env().await?).await?;

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
#[test_function(priority = -160, cleanup = false)]
pub async fn test_uninstall_app(
    _ctx: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::Running {
        bail!(Error::DaemonNotRunning);
    }

    // Clear devices before logging in
    let device_client = super::account::new_device_client()
        .await
        .context("Failed to create device client")?;
    super::account::clear_devices(&device_client)
        .await
        .context("failed to clear devices")?;

    // Login to test preservation of device/account
    // TODO: Remove once we can login before upgrade above
    mullvad_client
        .login_account(TEST_CONFIG.account_number.clone())
        .await
        .context("login failed")?;

    // save device to verify that uninstalling removes the device
    // we should still be logged in after upgrading
    let uninstalled_device = mullvad_client
        .get_device()
        .await
        .context("failed to get device data")?
        .into_device()
        .context("failed to get device")?
        .device
        .id;

    log::debug!("Uninstalling app");
    rpc.uninstall_app(get_app_env().await?).await?;

    let app_traces = rpc
        .find_mullvad_app_traces()
        .await
        .expect("failed to obtain remaining Mullvad files");
    assert!(
        app_traces.is_empty(),
        "found files after uninstall: {app_traces:?}"
    );

    if rpc.mullvad_daemon_get_status().await? != ServiceStatus::NotRunning {
        bail!(Error::DaemonRunning);
    }

    // verify that device was removed
    let device_client = super::account::new_device_client()
        .await
        .context("Failed to create device client")?;
    let devices = super::account::list_devices_with_retries(&device_client)
        .await
        .expect("failed to list devices");

    assert!(
        !devices.iter().any(|device| device.id == uninstalled_device),
        "device id {} still exists after uninstall",
        uninstalled_device,
    );

    // Re-install the app to ensure that the next test can run
    test_install_new_app(_ctx, rpc).await?;

    Ok(())
}

/// Install the multiple times starting from a connected state with auto-connect
/// disabled, failing if the app starts in a disconnected state.
///
/// This test is supposed to guard against regressions to this fix included in
/// the 2021.3-beta1 release:
/// https://github.com/mullvad/mullvadvpn-app/blob/2021.3-beta1/CHANGELOG.md#security
#[test_function(priority = -150)]
pub async fn test_installation_idempotency(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), anyhow::Error> {
    // Connect to any relay. This forces the daemon to enter a secured target state
    connect_and_wait(&mut mullvad_client)
        .await
        .map(|_| ()) // Discard the new tunnel state
        .or_else(|error| match error {
            Error::UnexpectedErrorState(_) => Ok(()),
            err => Err(err),
        })?;
    // Disable auto-connect
    mullvad_client
        .set_auto_connect(false)
        .await
        .context("Failed to enable auto-connect")?;

    // Check for traffic leaks during the installation processes.
    //
    // Start continously pinging while monitoring the network traffic. No
    // traffic should be observed going outside of the tunnel during either
    // installation process.
    let pinger = Pinger::start(&rpc).await;
    for _ in 0..2 {
        // Install the app
        log::info!("Installing new app");
        let app_package = get_package_desc(&TEST_CONFIG.app_package_filename)?;
        rpc.install_app(app_package).await?;
        log::info!("App was successfully installed!");

        // Verify that the daemon starts in a blocking state.
        // I.e., fail if the daemon starts in the disconnected state.
        const STATE_TRANSITION_TIMEOUT: Duration = Duration::from_secs(60);
        tokio::time::timeout(
            STATE_TRANSITION_TIMEOUT,
            wait_for_tunnel_state(mullvad_client.clone(), |state| !state.is_disconnected()),
        )
        .await
        .context("Timeout while waiting for tunnel state")?
        .context(
            "App did not start in the expected `Connected` state after the installation process.",
        )?;

        // Wait for an arbitrary amount of time. The point is that the pinger
        // should be able to ping while the newly installed app is running.
        if let Some(delay) = pinger.period().checked_mul(3) {
            tokio::time::sleep(delay).await;
        }
    }
    // Make sure that no network leak occured during any installation process.
    let guest_ip = pinger.guest_ip;
    let monitor_result = pinger.stop().await.unwrap();
    assert_eq!(
        monitor_result.packets.len(),
        0,
        "observed unexpected packets from {guest_ip}"
    );

    Ok(())
}

async fn replace_openvpn_cert(rpc: &ServiceClient) -> Result<(), Error> {
    use std::path::Path;

    const SOURCE_CERT_FILENAME: &str = "openvpn.ca.crt";
    const DEST_CERT_FILENAME: &str = "ca.crt";

    let dest_dir = match TEST_CONFIG.os {
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
