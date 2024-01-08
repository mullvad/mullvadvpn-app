//! Tests of interoperability with other software

use super::{helpers, Error, TestContext};
use mullvad_management_interface::MullvadProxyClient;
use test_macro::test_function;
use test_rpc::{ExecResult, ServiceClient};

/// This test fails if there is no connectivity, or name resolution fails, when connected to a VPN.
#[test_function(target_os = "linux")]
pub async fn test_containers(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    let result = probe_container_connectivity(&rpc).await?;
    assert!(
        result.success(),
        "containers should have connectivity when disconnected from the VPN"
    );

    helpers::connect_and_wait(&mut mullvad_client).await?;

    let result = probe_container_connectivity(&rpc).await?;
    assert!(
        result.success(),
        "containers should have connectivity when connected to the VPN"
    );

    Ok(())
}

/// This function executes curl inside podman or docker in the guest/test runner.
async fn probe_container_connectivity(rpc: &ServiceClient) -> Result<ExecResult, Error> {
    let has_podman = rpc.exec("bash", ["-c", "which podman"]).await?.success();

    let result = if has_podman {
        // podman run docker.io/curlimages/curl:latest https://am.i.mullvad.net/connected
        rpc.exec(
            "podman",
            [
                "run",
                "docker.io/curlimages/curl:latest",
                "https://am.i.mullvad.net/connected",
            ],
        )
        .await?
    } else {
        // sudo docker run docker.io/curlimages/curl:latest https://am.i.mullvad.net/connected
        rpc.exec(
            "sudo",
            [
                "docker",
                "run",
                "docker.io/curlimages/curl:latest",
                "https://am.i.mullvad.net/connected",
            ],
        )
        .await?
    };

    if !result.success() {
        let stdout = std::str::from_utf8(&result.stdout).unwrap_or("invalid utf8");
        let stderr = std::str::from_utf8(&result.stderr).unwrap_or("invalid utf8");
        log::error!("probe_container_connectivity failed:\n\nstdout:\n\n{stdout}\n\n{stderr}\n");
    }
    Ok(result)
}
