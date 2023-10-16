use super::config::TEST_CONFIG;
use super::{helpers, ui, Error, TestContext};
use mullvad_api::DevicesProxy;
use mullvad_management_interface::{types, Code, ManagementServiceClient};
use mullvad_types::device::Device;
use mullvad_types::states::TunnelState;
use std::net::ToSocketAddrs;
use std::time::Duration;
use talpid_types::net::wireguard;
use test_macro::test_function;
use test_rpc::ServiceClient;

const THROTTLE_RETRY_DELAY: Duration = Duration::from_secs(120);

/// Log in and create a new device for the account.
#[test_function(always_run = true, must_succeed = true, priority = -100)]
pub async fn test_login(
    _: TestContext,
    _rpc: ServiceClient,
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    //
    // Instruct daemon to log in
    //

    clear_devices(&new_device_client().await)
        .await
        .expect("failed to clear devices");

    log::info!("Logging in/generating device");
    login_with_retries(&mut mullvad_client)
        .await
        .expect("login failed");

    // Wait for the relay list to be updated
    helpers::ensure_updated_relay_list(&mut mullvad_client).await;

    Ok(())
}

/// Log out and remove the current device
/// from the account.
#[test_function(priority = 100)]
pub async fn test_logout(
    _: TestContext,
    _rpc: ServiceClient,
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    log::info!("Removing device");

    mullvad_client
        .logout_account(())
        .await
        .expect("logout failed");

    Ok(())
}

/// Try to log in when there are too many devices. Make sure it fails as expected.
#[test_function(priority = -151)]
pub async fn test_too_many_devices(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    log::info!("Using up all devices");

    let device_client = new_device_client().await;

    const MAX_ATTEMPTS: usize = 15;

    for _ in 0..MAX_ATTEMPTS {
        let pubkey = wireguard::PrivateKey::new_from_random().public_key();

        match device_client
            .create(TEST_CONFIG.account_number.clone(), pubkey)
            .await
        {
            Ok(_) => (),
            Err(mullvad_api::rest::Error::ApiError(_status, ref code))
                if code == mullvad_api::MAX_DEVICES_REACHED =>
            {
                break;
            }
            Err(error) => {
                log::error!(
                    "Failed to generate device: {error:?}. Retrying after {} seconds",
                    THROTTLE_RETRY_DELAY.as_secs()
                );
                // Sleep for an overly long time.
                // TODO: Only sleep for this long if the error is caused by throttling.
                tokio::time::sleep(THROTTLE_RETRY_DELAY).await;
            }
        }
    }

    log::info!("Log in with too many devices");
    let login_result = login_with_retries(&mut mullvad_client).await;

    assert!(matches!(login_result, Err(status) if status.code() == Code::ResourceExhausted));

    // Run UI test
    let ui_result = ui::run_test_env(
        &rpc,
        &["too-many-devices.spec"],
        [("ACCOUNT_NUMBER", &*TEST_CONFIG.account_number)],
    )
    .await
    .unwrap();
    assert!(ui_result.success());

    if let Err(error) = clear_devices(&device_client).await {
        log::error!("Failed to clear devices: {error}");
    }

    Ok(())
}

/// Test whether the daemon can detect that the current device has been revoked, and enters the
/// error state in that case.
///
/// # Limitations
///
/// Currently, this test does not check whether the daemon automatically detects that the device has
/// been revoked while reconnecting.
#[test_function(priority = -150)]
pub async fn test_revoked_device(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    log::info!("Logging in/generating device");
    login_with_retries(&mut mullvad_client)
        .await
        .expect("login failed");

    let device_id = mullvad_client
        .get_device(())
        .await
        .expect("failed to get device data")
        .into_inner()
        .device
        .unwrap()
        .device
        .unwrap()
        .id;

    helpers::connect_and_wait(&mut mullvad_client).await?;

    log::debug!("Removing current device");

    let device_client = new_device_client().await;
    retry_if_throttled(|| {
        device_client.remove(TEST_CONFIG.account_number.clone(), device_id.clone())
    })
    .await
    .expect("failed to revoke device");

    // Sleep for a while: the device state is only updated if sufficiently old,
    // so `update_device` might be a no-op if called too often.
    const PRE_UPDATE_SLEEP: Duration = Duration::from_secs(12);
    tokio::time::sleep(PRE_UPDATE_SLEEP).await;

    // Begin listening to tunnel state changes first, so that we catch changes due to
    // `update_device`.
    let events = mullvad_client
        .events_listen(())
        .await
        .expect("failed to begin listening for state changes")
        .into_inner();
    let next_state =
        helpers::find_next_tunnel_state(events, |state| matches!(state, TunnelState::Error(..),));

    log::debug!("Update device state");

    let _update_status = mullvad_client.update_device(()).await;

    // Ensure that the tunnel state transitions to "error". Fail if it transitions to some other
    // state.
    let new_state = next_state.await?;
    assert!(
        matches!(&new_state, TunnelState::Error(error_state) if error_state.is_blocking()),
        "expected blocking error state, got {new_state:?}"
    );

    // Verify that the device state is `Revoked`.
    let device_state = mullvad_client
        .get_device(())
        .await
        .expect("failed to get device data");
    assert_eq!(
        device_state.into_inner().state,
        i32::from(types::device_state::State::Revoked),
        "expected device to be revoked"
    );

    // Run UI test
    let ui_result = ui::run_test(&rpc, &["device-revoked.spec"]).await.unwrap();
    assert!(ui_result.success());

    Ok(())
}

/// Remove all devices on the current account
pub async fn clear_devices(device_client: &DevicesProxy) -> Result<(), mullvad_api::rest::Error> {
    log::info!("Removing all devices for account");

    for dev in list_devices_with_retries(device_client).await?.into_iter() {
        if let Err(error) = device_client
            .remove(TEST_CONFIG.account_number.clone(), dev.id)
            .await
        {
            log::warn!("Failed to remove device: {error}");
        }
    }
    Ok(())
}

pub async fn new_device_client() -> DevicesProxy {
    let api_endpoint = mullvad_api::ApiEndpoint::from_env_vars();

    let api_host = format!("api.{}", TEST_CONFIG.mullvad_host);
    let api_addr = format!("{api_host}:443")
        .to_socket_addrs()
        .expect("failed to resolve API host")
        .next()
        .unwrap();

    // Override the API endpoint to use the one specified in the test config
    let _ = mullvad_api::API.override_init(mullvad_api::ApiEndpoint {
        host: api_host,
        addr: api_addr,
        ..api_endpoint
    });

    let api = mullvad_api::Runtime::new(tokio::runtime::Handle::current())
        .expect("failed to create api runtime");
    let rest_handle = api
        .mullvad_rest_handle(
            mullvad_api::proxy::ApiConnectionMode::Direct.into_repeat(),
            |_| async { true },
        )
        .await;
    DevicesProxy::new(rest_handle)
}

/// Log in and retry if it fails due to throttling
pub async fn login_with_retries(
    mullvad_client: &mut ManagementServiceClient,
) -> Result<(), mullvad_management_interface::Status> {
    loop {
        let result = mullvad_client
            .login_account(TEST_CONFIG.account_number.clone())
            .await;

        if let Err(error) = result {
            if !error.message().contains("THROTTLED") {
                return Err(error);
            }

            // Work around throttling errors by sleeping

            log::debug!(
                "Login failed due to throttling. Sleeping for {} seconds",
                THROTTLE_RETRY_DELAY.as_secs()
            );

            tokio::time::sleep(THROTTLE_RETRY_DELAY).await;
        } else {
            break Ok(());
        }
    }
}

pub async fn list_devices_with_retries(
    device_client: &DevicesProxy,
) -> Result<Vec<Device>, mullvad_api::rest::Error> {
    retry_if_throttled(|| device_client.list(TEST_CONFIG.account_number.clone())).await
}

pub async fn retry_if_throttled<
    F: std::future::Future<Output = Result<T, mullvad_api::rest::Error>>,
    T,
>(
    new_attempt: impl Fn() -> F,
) -> Result<T, mullvad_api::rest::Error> {
    loop {
        match new_attempt().await {
            Ok(val) => break Ok(val),
            // Work around throttling errors by sleeping
            Err(mullvad_api::rest::Error::ApiError(
                mullvad_api::rest::StatusCode::TOO_MANY_REQUESTS,
                _,
            )) => {
                log::debug!(
                    "Device list fetch failed due to throttling. Sleeping for {} seconds",
                    THROTTLE_RETRY_DELAY.as_secs()
                );

                tokio::time::sleep(THROTTLE_RETRY_DELAY).await;
            }
            Err(error) => break Err(error),
        }
    }
}

#[test_function]
pub async fn test_automatic_wireguard_rotation(
    ctx: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    // Make note of current WG key
    let old_key = mullvad_client
        .get_device(())
        .await
        .expect("Could not get device")
        .into_inner()
        .device
        .unwrap()
        .device
        .unwrap()
        .pubkey;

    // Stop daemon
    rpc.set_mullvad_daemon_service_state(false)
        .await
        .expect("Could not stop system service");

    // Open device.json and change created field to more than 7 days ago
    rpc.make_device_json_old()
        .await
        .expect("Could not change device.json to have an old created timestamp");

    // Start daemon
    rpc.set_mullvad_daemon_service_state(true)
        .await
        .expect("Could not start system service");

    // NOTE: Need to create a new `mullvad_client` here after the restart otherwise we can't
    // communicate with the daemon
    drop(mullvad_client);
    let mut mullvad_client = ctx.rpc_provider.new_client().await;

    // Verify rotation has happened after a minute
    const KEY_ROTATION_TIMEOUT: Duration = Duration::from_secs(100);

    let mut event_stream = mullvad_client.events_listen(()).await.unwrap().into_inner();
    let get_pub_key_event = async {
        loop {
            let message = event_stream.message().await;
            if let Ok(Some(event)) = message {
                match event.event.unwrap() {
                    mullvad_management_interface::types::daemon_event::Event::Device(
                        device_event,
                    ) => {
                        let pubkey = device_event
                            .new_state
                            .unwrap()
                            .device
                            .unwrap()
                            .device
                            .unwrap()
                            .pubkey;
                        return Ok(pubkey);
                    }
                    _ => continue,
                }
            }
            return Err(message);
        }
    };

    let new_key = tokio::time::timeout(KEY_ROTATION_TIMEOUT, get_pub_key_event)
        .await
        .unwrap()
        .unwrap();

    assert_ne!(old_key, new_key);
    Ok(())
}
