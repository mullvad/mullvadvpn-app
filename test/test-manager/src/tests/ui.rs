use super::config::TEST_CONFIG;
use super::helpers;
use super::{Error, TestContext};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::relay_constraints::{RelayConstraints, RelaySettings};
use mullvad_types::relay_list::{Relay, RelayEndpointData};
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

    match TEST_CONFIG.os {
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
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    // tunnel-state.spec precondition: a single WireGuard relay should be selected
    log::info!("Select WireGuard relay");
    let entry = helpers::filter_relays(&mut mullvad_client, |relay: &Relay| {
        relay.active && matches!(relay.endpoint_data, RelayEndpointData::Wireguard(_))
    })
    .await?
    .pop()
    .unwrap();

    // The test expects us to be disconnected and logged in but to have a specific relay selected
    let relay_settings = RelaySettings::Normal(RelayConstraints {
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
            ("IN_IP", &entry.ipv4_addr_in.to_string()),
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

#[test_function]
async fn test_custom_access_methods_gui(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    use mullvad_api::env;
    use mullvad_relay_selector::{RelaySelector, SelectorConfig};
    use talpid_types::net::proxy::CustomProxy;
    // For this test to work, we need to supply the following env-variables:
    //
    // * SHADOWSOCKS_SERVER_IP
    // * SHADOWSOCKS_SERVER_PORT
    // * SHADOWSOCKS_SERVER_CIPHER
    // * SHADOWSOCKS_SERVER_PASSWORD
    //
    // See `gui/test/e2e/installed/state-dependent/api-access-methods.spec.ts`
    // for details. The setup should be the same as in
    // `test_manager::tests::access_methods::test_shadowsocks`.
    //
    // # Note
    //
    // API overrides have to be nullified before proceeding with this test. This
    // is accomplished by setting the env variable
    // `MULLVAD_API_FORCE_DIRECT=false` and restarting the daemon.

    let mut env = helpers::get_app_env();
    env.insert(env::API_FORCE_DIRECT_VAR.to_string(), "0".to_string());

    tokio::time::timeout(
        std::time::Duration::from_secs(60),
        rpc.set_daemon_environment(env),
    )
    .await
    .map_err(|_| Error::DaemonNotRunning)??;

    let gui_test = "api-access-methods.spec";
    let relay_list = mullvad_client.get_relay_locations().await.unwrap();
    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), relay_list);
    let access_method = relay_selector
        .get_bridge_forced()
        .and_then(|proxy| match proxy {
            CustomProxy::Shadowsocks(s) => Some(s),
            _ => None
        })
        .expect("`test_shadowsocks` needs at least one shadowsocks relay to execute. Found none in relay list.");

    let ui_result = run_test_env(
        &rpc,
        &[gui_test],
        [
            (
                "SHADOWSOCKS_SERVER_IP",
                access_method.endpoint.ip().to_string().as_ref(),
            ),
            (
                "SHADOWSOCKS_SERVER_PORT",
                access_method.endpoint.port().to_string().as_ref(),
            ),
            ("SHADOWSOCKS_SERVER_CIPHER", access_method.cipher.as_ref()),
            (
                "SHADOWSOCKS_SERVER_PASSWORD",
                access_method.password.as_ref(),
            ),
        ],
    )
    .await
    .unwrap();

    assert!(ui_result.success());

    // Reset the `api-override` feature.
    tokio::time::timeout(
        std::time::Duration::from_secs(60),
        rpc.set_daemon_environment(helpers::get_app_env()),
    )
    .await
    .map_err(|_| Error::DaemonNotRunning)??;

    Ok(())
}
