use mullvad_management_interface::MullvadProxyClient;
use std::str;
use test_macro::test_function;
use test_rpc::{ExecResult, ServiceClient};

use super::{helpers, TestContext};

const AM_I_MULLVAD: &str = "https://am.i.mullvad.net/connected";

#[test_function]
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub async fn test_split_tunnel(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    let mut errored = false;
    let parse_am_i_mullvad = |result: ExecResult| {
        let stdout = str::from_utf8(&result.stdout).expect("curl output is UTF-8");

        Ok(if stdout.contains("You are connected") {
            true
        } else if stdout.contains("You are not connected") {
            false
        } else {
            anyhow::bail!("Unexpected output from `curl {AM_I_MULLVAD}`: {stdout}")
        })
    };

    helpers::connect_and_wait(&mut mullvad_client).await?;

    let i_am_mullvad = parse_am_i_mullvad(rpc.exec("curl", [AM_I_MULLVAD]).await?)?;
    if !i_am_mullvad {
        log::error!("We should be connected, but `am.i.mullvad` reported that it was not connected to Mullvad.");
        errored = true;
    }

    let i_am_mullvad_while_split =
        parse_am_i_mullvad(rpc.exec("mullvad-exclude", ["curl", AM_I_MULLVAD]).await?)?;
    if i_am_mullvad_while_split {
        log::error!(
            "`mullvad-exclude curl {AM_I_MULLVAD}` reported that it was connected to Mullvad."
        );
        log::error!("`am-i-mullvad` does not appear to have been split correctly.");
        errored = true;
    }

    helpers::disconnect_and_wait(&mut mullvad_client).await?;

    let i_am_mullvad_while_disconnected =
        parse_am_i_mullvad(rpc.exec("curl", [AM_I_MULLVAD]).await?)?;
    if i_am_mullvad_while_disconnected {
        log::error!("We should be disconnected, but `curl {AM_I_MULLVAD}` reported that it was connected to Mullvad.");
        log::error!("Host machine is probably connected to Mullvad. This may affect test results.");
        errored = true;
    }

    if errored {
        anyhow::bail!("test_split_tunnel failed, see log output for details.");
    }

    Ok(())
}
