use super::config::TEST_CONFIG;
use super::helpers;
use super::{Error, TestContext};
use mullvad_management_interface::{types, ManagementServiceClient};
use mullvad_types::relay_constraints::{RelayConstraintsUpdate, RelaySettingsUpdate};
use std::{
    collections::BTreeMap,
    fmt::Debug,
    path::{Path, PathBuf},
};
use test_macro::test_function;
use test_rpc::{meta::Os, ExecResult, ServiceClient};

pub async fn run_test<T: AsRef<str> + Debug>(
    rpc: &ServiceClient,
    params: &[T],
) -> Result<ExecResult, Error> {
    let env: [(&str, T); 0] = [];
    run_test_env(rpc, params, env).await
}

pub async fn run_test_env<
    I: IntoIterator<Item = (K, T)> + Debug,
    K: AsRef<str> + Debug,
    T: AsRef<str> + Debug,
>(
    rpc: &ServiceClient,
    params: &[T],
    env: I,
) -> Result<ExecResult, Error> {
    let new_params: Vec<String>;
    let bin_path;

    match rpc.get_os().await? {
        Os::Linux => {
            bin_path = PathBuf::from("/usr/bin/xvfb-run");

            let ui_runner_path =
                Path::new(&TEST_CONFIG.artifacts_dir).join(&TEST_CONFIG.ui_e2e_tests_filename);
            new_params = std::iter::once(ui_runner_path.to_string_lossy().into_owned())
                .chain(params.iter().map(|param| param.as_ref().to_owned()))
                .collect();
        }
        _ => {
            bin_path =
                Path::new(&TEST_CONFIG.artifacts_dir).join(&TEST_CONFIG.ui_e2e_tests_filename);
            new_params = params
                .iter()
                .map(|param| param.as_ref().to_owned())
                .collect();
        }
    }

    let env: BTreeMap<String, String> = env
        .into_iter()
        .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
        .collect();

    // env may contain sensitive info
    //log::info!("Running UI tests: {params:?}, env: {env:?}");
    log::info!("Running UI tests: {params:?}");

    let result = rpc
        .exec_env(
            bin_path.to_string_lossy().into_owned(),
            new_params.into_iter(),
            env,
        )
        .await?;

    if !result.success() {
        let stdout = std::str::from_utf8(&result.stdout).unwrap_or("invalid utf8");
        let stderr = std::str::from_utf8(&result.stderr).unwrap_or("invalid utf8");

        log::debug!("UI test failed:\n\nstdout:\n\n{stdout}\n\n{stderr}\n");
    }

    Ok(result)
}

/// Test how various tunnel settings are handled and displayed by the GUI
#[test_function]
pub async fn test_ui_tunnel_settings(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    // tunnel-state.spec precondition: a single WireGuard relay should be selected
    log::info!("Select WireGuard relay");
    let entry = helpers::filter_relays(&mut mullvad_client, |relay: &types::Relay| {
        relay.active && relay.endpoint_type == i32::from(types::relay::RelayType::Wireguard)
    })
    .await?
    .pop()
    .unwrap();

    // The test expects us to be disconnected and logged in but to have a specific relay selected
    let relay_settings = RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
        location: helpers::into_constraint(&entry),
        ..Default::default()
    });

    helpers::set_relay_settings(&mut mullvad_client, relay_settings)
        .await
        .expect("failed to update relay settings");

    let ui_result = run_test_env(
        &rpc,
        &["tunnel-state.spec"],
        [
            ("HOSTNAME", entry.hostname.as_str()),
            ("IN_IP", entry.ipv4_addr_in.as_str()),
            (
                "CONNECTION_CHECK_URL",
                &format!("https://am.i.{}", TEST_CONFIG.mullvad_host),
            ),
        ],
    )
    .await
    .unwrap();
    assert!(ui_result.success());

    Ok(())
}

/// Test whether logging in and logging out work in the GUI
#[test_function(priority = 500)]
pub async fn test_ui_login(_: TestContext, rpc: ServiceClient) -> Result<(), Error> {
    let ui_result = run_test_env(
        &rpc,
        &["login.spec"],
        [("ACCOUNT_NUMBER", &*TEST_CONFIG.account_number)],
    )
    .await
    .unwrap();
    assert!(ui_result.success());

    Ok(())
}
