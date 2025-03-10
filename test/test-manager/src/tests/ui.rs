use super::{config::TEST_CONFIG, helpers, Error, TestContext};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_relay_selector::query::builder::RelayQueryBuilder;
use mullvad_types::relay_constraints::RelaySettings;
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

            let ui_runner_path = Path::new(&TEST_CONFIG.artifacts_dir).join(
                TEST_CONFIG
                    .ui_e2e_tests_filename
                    .as_ref()
                    .ok_or(Error::MissingGuiTest)?,
            );
            new_params = std::iter::once(ui_runner_path.to_string_lossy().into_owned())
                .chain(params.iter().map(|param| param.as_ref().to_owned()))
                .collect();
        }
        _ => {
            bin_path = Path::new(&TEST_CONFIG.artifacts_dir).join(
                TEST_CONFIG
                    .ui_e2e_tests_filename
                    .as_ref()
                    .ok_or(Error::MissingGuiTest)?,
            );
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
    // log::info!("Running UI tests: {params:?}, env: {env:?}");
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

        log::error!("UI test failed:\n\nstdout:\n\n{stdout}\n\n{stderr}\n");
    }

    Ok(result)
}

/// Test how various tunnel settings are handled and displayed by the GUI
#[test_function]
pub async fn test_ui_tunnel_settings(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    // tunnel-state.spec precondition: a single WireGuard relay should be selected
    log::info!("Select WireGuard relay");
    let entry =
        helpers::constrain_to_relay(&mut mullvad_client, RelayQueryBuilder::wireguard().build())
            .await?;

    let ui_result = run_test_env(
        &rpc,
        &["state-dependent/tunnel-state.spec"],
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

/// Test how various tunnel settings for OpenVPN are handled and displayed by the GUI
#[test_function]
pub async fn test_ui_openvpn_tunnel_settings(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    // openvpn-tunnel-state.spec precondition: OpenVPN needs to be selected
    let relay_settings = mullvad_client.get_settings().await?.get_relay_settings();
    let RelaySettings::Normal(mut constraints) = relay_settings else {
        unimplemented!()
    };
    constraints.tunnel_protocol = talpid_types::net::TunnelType::OpenVpn;
    mullvad_client
        .set_relay_settings(RelaySettings::Normal(constraints))
        .await?;

    let ui_result = run_test(&rpc, &["openvpn-tunnel-state.spec"]).await?;
    assert!(ui_result.success());
    Ok(())
}

/// Test whether logging in and logging out work in the GUI
#[test_function(priority = 500)]
pub async fn test_ui_login(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    mullvad_client.logout_account().await?;
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

#[test_function(priority = 1000)]
async fn test_custom_access_methods_gui(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    use mullvad_api::env;
    use mullvad_relay_selector::{RelaySelector, SelectorConfig};

    // For this test to work, we need to supply the following env-variables:
    //
    // * SHADOWSOCKS_SERVER_IP
    // * SHADOWSOCKS_SERVER_PORT
    // * SHADOWSOCKS_SERVER_CIPHER
    // * SHADOWSOCKS_SERVER_PASSWORD
    //
    // See
    // `desktop/packages/mullvad-vpn/test/e2e/installed/state-dependent/api-access-methods.spec.ts`
    // for details. The setup should be the same as in
    // `test_manager::tests::access_methods::test_shadowsocks`.
    //
    // # Note
    //
    // API overrides have to be nullified before proceeding with this test. This
    // is accomplished by setting the env variable
    // `MULLVAD_API_FORCE_DIRECT=false` and restarting the daemon.

    let mut env = helpers::get_app_env().await?;
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

    Ok(())
}

#[test_function(priority = 1000)]
async fn test_custom_bridge_gui(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    // For this test to work, we need to supply the following env-variables:
    //
    // * SHADOWSOCKS_SERVER_IP
    // * SHADOWSOCKS_SERVER_PORT
    // * SHADOWSOCKS_SERVER_CIPHER
    // * SHADOWSOCKS_SERVER_PASSWORD
    //
    // See
    // `desktop/packages/mullvad-vpn/test/e2e/installed/state-dependent/custom-bridge.spec.ts`
    // for details. The setup should be the same as in
    // `test_manager::tests::access_methods::test_shadowsocks`.

    let gui_test = "custom-bridge.spec";

    let settings = mullvad_client.get_settings().await.unwrap();
    let relay_list = mullvad_client.get_relay_locations().await.unwrap();
    let relay_selector = helpers::get_daemon_relay_selector(&settings, relay_list);
    let custom_proxy = relay_selector
        .get_bridge_forced()
        .expect("`test_shadowsocks` needs at least one shadowsocks relay to execute. Found none in relay list.");

    let ui_result = run_test_env(
        &rpc,
        &[gui_test],
        [
            (
                "SHADOWSOCKS_SERVER_IP",
                custom_proxy.endpoint.ip().to_string().as_ref(),
            ),
            (
                "SHADOWSOCKS_SERVER_PORT",
                custom_proxy.endpoint.port().to_string().as_ref(),
            ),
            ("SHADOWSOCKS_SERVER_CIPHER", custom_proxy.cipher.as_ref()),
            (
                "SHADOWSOCKS_SERVER_PASSWORD",
                custom_proxy.password.as_ref(),
            ),
        ],
    )
    .await
    .unwrap();

    assert!(ui_result.success());

    Ok(())
}

/// Test settings import / IP overrides in the GUI
#[test_function]
pub async fn test_import_settings_ui(
    _: TestContext,
    rpc: ServiceClient,
    _: MullvadProxyClient,
) -> Result<(), Error> {
    let ui_result = run_test(&rpc, &["settings-import.spec"]).await?;
    assert!(ui_result.success());
    Ok(())
}

/// Test obfuscation settings in the GUI
#[test_function]
pub async fn test_obfuscation_settings_ui(
    _: TestContext,
    rpc: ServiceClient,
    _: MullvadProxyClient,
) -> Result<(), Error> {
    let ui_result = run_test(&rpc, &["obfuscation.spec"]).await?;
    assert!(ui_result.success());
    Ok(())
}

/// Test settings in the GUI
#[test_function]
pub async fn test_settings_ui(
    _: TestContext,
    rpc: ServiceClient,
    _: MullvadProxyClient,
) -> Result<(), Error> {
    let ui_result = run_test(&rpc, &["settings.spec"]).await?;
    assert!(ui_result.success());
    Ok(())
}
