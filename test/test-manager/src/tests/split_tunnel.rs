use mullvad_management_interface::MullvadProxyClient;
use std::str;
use test_macro::test_function;
use test_rpc::{ExecResult, ServiceClient};

use super::{helpers, TestContext};

#[test_function(target_os = "windows")]
pub async fn test_split_tunnel_windows(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    const AM_I_MULLVAD_EXE: &str = "E:\\am-i-mullvad.exe";

    async fn am_i_mullvad(rpc: &ServiceClient) -> anyhow::Result<bool> {
        parse_am_i_mullvad(rpc.exec(AM_I_MULLVAD_EXE, []).await?)
    }

    let mut errored = false;

    helpers::disconnect_and_wait(&mut mullvad_client).await?;

    if am_i_mullvad(&rpc).await? {
        log::error!("We should be disconnected, but `{AM_I_MULLVAD_EXE}` reported that it was connected to Mullvad.");
        log::error!("Host machine is probably connected to Mullvad, this will throw off results");
        errored = true
    }

    helpers::connect_and_wait(&mut mullvad_client).await?;

    if !am_i_mullvad(&rpc).await? {
        log::error!(
            "We should be connected, but `{AM_I_MULLVAD_EXE}` reported no connection to Mullvad."
        );
        errored = true
    }

    mullvad_client
        .add_split_tunnel_app(AM_I_MULLVAD_EXE)
        .await?;
    mullvad_client.set_split_tunnel_state(true).await?;

    if am_i_mullvad(&rpc).await? {
        log::error!(
            "`{AM_I_MULLVAD_EXE}` should have been split, but it reported a connection to Mullvad"
        );
        errored = true
    }

    helpers::disconnect_and_wait(&mut mullvad_client).await?;

    if am_i_mullvad(&rpc).await? {
        log::error!(
            "`{AM_I_MULLVAD_EXE}` reported a connection to Mullvad while split and disconnected"
        );
        errored = true
    }

    mullvad_client.set_split_tunnel_state(false).await?;
    mullvad_client
        .remove_split_tunnel_app(AM_I_MULLVAD_EXE)
        .await?;

    if errored {
        anyhow::bail!("test_split_tunnel failed, see log output for details.");
    }

    Ok(())
}

#[test_function(target_os = "linux")]
pub async fn test_split_tunnel_linux(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    const AM_I_MULLVAD_URL: &str = "https://am.i.mullvad.net/connected";

    async fn am_i_mullvad(rpc: &ServiceClient, split_tunnel: bool) -> anyhow::Result<bool> {
        let result = if split_tunnel {
            rpc.exec("mullvad-exclude", ["curl", AM_I_MULLVAD_URL])
                .await?
        } else {
            rpc.exec("curl", [AM_I_MULLVAD_URL]).await?
        };

        parse_am_i_mullvad(result)
    }

    let mut errored = false;

    helpers::connect_and_wait(&mut mullvad_client).await?;

    if !am_i_mullvad(&rpc, false).await? {
        log::error!("We should be connected, but `am.i.mullvad` reported that it was not connected to Mullvad.");
        errored = true;
    }

    if am_i_mullvad(&rpc, true).await? {
        log::error!(
            "`mullvad-exclude curl {AM_I_MULLVAD_URL}` reported that it was connected to Mullvad."
        );
        log::error!("`curl` does not appear to have been split correctly.");
        errored = true;
    }

    helpers::disconnect_and_wait(&mut mullvad_client).await?;

    if am_i_mullvad(&rpc, false).await? {
        log::error!("We should be disconnected, but `curl {AM_I_MULLVAD_URL}` reported that it was connected to Mullvad.");
        log::error!("Host machine is probably connected to Mullvad. This may affect test results.");
        errored = true;
    }

    if errored {
        anyhow::bail!("test_split_tunnel failed, see log output for details.");
    }

    Ok(())
}

/// Parse output from am-i-mullvad. Returns true if connected to Mullvad.
fn parse_am_i_mullvad(result: ExecResult) -> anyhow::Result<bool> {
    let stdout = str::from_utf8(&result.stdout).expect("curl output is UTF-8");

    Ok(if stdout.contains("You are connected") {
        true
    } else if stdout.contains("You are not connected") {
        false
    } else {
        anyhow::bail!("Unexpected output from am-i-mullvad: {stdout:?}")
    })
}
