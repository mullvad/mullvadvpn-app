use mullvad_management_interface::MullvadProxyClient;
use std::str;
use test_macro::test_function;
use test_rpc::{ExecResult, ServiceClient};

use super::{helpers, Error, TestContext};

#[test_function]
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub async fn test_split_tunnel(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    let mut errored = false;
    let parse_am_i_mullvad = |result: ExecResult| -> bool {
        let stdout = str::from_utf8(&result.stdout).expect("am-i-mullvad output is UTF-8");

        if stdout.contains("You are connected") {
            true
        } else if stdout.contains("You are not connected") {
            false
        } else {
            panic!("Unexpected output from `am-i-mullvad`: {stdout}")
        }
    };

    helpers::connect_and_wait(&mut mullvad_client).await?;

    let i_am_mullvad = parse_am_i_mullvad(rpc.exec("am-i-mullvad", []).await?);
    if !i_am_mullvad {
        log::error!("We should be connected, but `am-i-mullvad` reported that it was not connected to Mullvad.");
        errored = true;
    }

    let i_am_mullvad_while_split =
        parse_am_i_mullvad(rpc.exec("mullvad-exclude", ["am-i-mullvad"]).await?);
    if i_am_mullvad_while_split {
        log::error!("`mullvad-exclude am-i-mullvad` reported that it was connected to Mullvad.");
        log::error!("`am-i-mullvad` does not appear to have been split correctly.");
        errored = true;
    }

    helpers::disconnect_and_wait(&mut mullvad_client).await?;

    let i_am_mullvad_while_disconnected = parse_am_i_mullvad(rpc.exec("am-i-mullvad", []).await?);
    if i_am_mullvad_while_disconnected {
        log::error!("We should be disconnected, but `am-i-mullvad` reported that it was connected to Mullvad.");
        log::error!("Host machine is probably connected to Mullvad. This may affect test results.");
        errored = true;
    }

    if errored {
        panic!("test_split_tunnel failed, see logs for details.");
    }

    Ok(())
}
