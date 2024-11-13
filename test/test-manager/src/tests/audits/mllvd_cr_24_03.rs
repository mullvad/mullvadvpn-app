#![cfg(target_os = "linux")]
//! Test mitigation for mllvd_cr_24_03
//!
//! By sending an ARP request for the in-tunnel IP address to any network interface on the device running Mullvad, it
//! will respond and confirm that it owns this address. This means someone on the LAN or similar can figure out the
//! device's in-tunnel IP, and potentially also make an educated guess that they are using Mullvad at all.
//!
//! # Setup
//!
//! Victim: test-runner
//!
//! Network adjacent attacker: test-manager
//!
//! # Procedure
//! Have test-runner connect to relay. Let test-manager know about the test-runner's private in-tunnel IP (such that
//! we don't have to enumerate all possible private IPs).
//!
//! Have test-manager invoke the `arping` command targeting the bridge network between test-manager <-> test-runner.
//! If `arping` times out without a reply, it will exit with a non-0 exit code. If it got a reply from test-runner, it
//! will exit with code 0.
//!
//! Note that only linux was susceptible to this vulnerability.

use std::ffi::OsStr;
use std::process::Output;

use anyhow::bail;
use mullvad_management_interface::MullvadProxyClient;
use test_macro::test_function;
use test_rpc::ServiceClient;

use crate::tests::helpers::*;
use crate::tests::TestContext;
use crate::vm::network::bridge;

#[test_function(target_os = "linux")]
pub async fn test_mllvd_cr_24_03(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    // Get the bridge network between manager and runner. This will be used when invoking `arping`.
    let bridge = bridge()?;
    // Connect runner to a relay. After this point we will be able to acquire the runner's private in-tunnel IP.
    connect_and_wait(&mut mullvad_client).await?;
    // Get the private ip address
    let in_tunnel_ip = {
        let vpn_interface = get_tunnel_interface(&mut mullvad_client).await.unwrap(); // :cat-scream:
        rpc.get_interface_ip(vpn_interface).await?
    };
    // Invoke arping
    let malicious_arping = arping([
        "-w",
        "5",
        "-i",
        "1",
        "-I",
        &bridge,
        &in_tunnel_ip.to_string(),
    ])
    .await?;
    // If arping exited with code 0, it means the runner replied to the ARP request, implying the runner leaked its
    // private in-tunnel IP!
    if let Some(0) = malicious_arping.status.code() {
        log::error!("{}", String::from_utf8(malicious_arping.stdout)?);
        bail!("ARP leak detected")
    }
    // test runner did not respond to ARP request, leak mitigation seems to work!
    Ok(())
}

/// Invoke `arping` on test-manager.
async fn arping<I, S>(args: I) -> std::io::Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut arping = tokio::process::Command::new("arping");
    arping.args(args);
    arping.output().await
}
