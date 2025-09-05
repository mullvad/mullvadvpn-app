//! macOS-specific tests.

use anyhow::{Context, bail, ensure};
use mullvad_management_interface::MullvadProxyClient;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use test_macro::test_function;
use test_rpc::ServiceClient;

use crate::tests::helpers::connect_and_wait;

use super::TestContext;

/// Test that the local resolver alias is readded if removed.
#[test_function(target_os = "macos")]
async fn test_app_ifconfig_alias(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    // Connect to enable the local resolver
    connect_and_wait(&mut mullvad_client).await?;

    let current_resolver = get_first_dns_resolver(&rpc).await?;
    log::debug!("Current DNS resolver: {current_resolver}");

    let current_resolver = match current_resolver {
        IpAddr::V4(ip) => ip,
        IpAddr::V6(ip) => bail!("Expected IPv4 resolver, got {ip}"),
    };

    ensure!(
        current_resolver.is_loopback() && current_resolver != Ipv4Addr::LOCALHOST,
        "Current resolver should be a loopback address (and not 127.0.0.1), got {current_resolver}"
    );

    // Remove all alias and assert that one is readded.
    rpc.ifconfig_alias_remove("lo0", current_resolver).await?;

    ensure!(
        !alias_exists(&rpc, "lo0", current_resolver).await?,
        "Aliases should have been removed"
    );

    for _attempt in 0..5 {
        if alias_exists(&rpc, "lo0", current_resolver).await? {
            log::debug!("Alias was readded!");
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    bail!("lo0 alias was not readded after removal");
}

/// Test that we can add and remove IP "aliases" to network interfaces.
///
/// This is effectively testing that macOS behaves as expected, and that future versions of it
/// don't break this functionality.
#[test_function(target_os = "macos")]
async fn test_ifconfig_add_alias(
    _: TestContext,
    rpc: ServiceClient,
    _: MullvadProxyClient,
) -> anyhow::Result<()> {
    let alias = Ipv4Addr::new(127, 123, 123, 123);
    let interface = "lo0";

    log::info!("Will try to assign alias {alias} to interface {interface}");

    // Sanity-check that alias does not exist before we add it.
    ensure!(
        !alias_exists(&rpc, interface, alias).await?,
        "Alias shouldn't exist before it's created. Was it left over from a previous test?"
    );

    // Add alias and assert that it exists.
    rpc.ifconfig_alias_add(interface, alias).await?;
    ensure!(
        alias_exists(&rpc, interface, alias).await?,
        "Alias should have been created!"
    );

    // Ensure that we clean up the alias after the test, even if it fails
    let rpc2 = rpc.clone();
    let _cleanup_guard = scopeguard::guard((), |()| {
        log::info!("Cleaning up after test_ifconfig_add_alias");

        let Ok(runtime_handle) = tokio::runtime::Handle::try_current() else {
            log::error!("Missing tokio runtime");
            return;
        };

        runtime_handle.spawn(async move {
            // Ensure that the alias is removed even if the test fails.
            if let Err(e) = rpc2.ifconfig_alias_remove(interface, alias).await {
                log::error!("Failed to remove alias {alias} from interface {interface}: {e}");
            }
        });
    });

    // Assert that we can bind to the alias.
    rpc.send_udp(
        None,
        SocketAddr::from((alias, 0)),
        SocketAddr::from((Ipv4Addr::LOCALHOST, 1234)),
    )
    .await
    .context("Failed to bind to alias")?;

    // Remove alias and assert that it doesn't exist.
    rpc.ifconfig_alias_remove(interface, alias).await?;
    ensure!(
        !alias_exists(&rpc, interface, alias).await?,
        "Alias should have been removed!"
    );

    Ok(())
}

/// Check if an IP alias exists for `interface`.
async fn alias_exists(
    rpc: &ServiceClient,
    interface: &str,
    alias: Ipv4Addr,
) -> anyhow::Result<bool> {
    let alias = alias.to_string();
    let result = rpc.exec("ifconfig", [interface]).await?;

    let stdout = String::from_utf8(result.stdout)?;
    let stderr = String::from_utf8(result.stderr)?;

    if result.code != Some(0) {
        log::error!("ifconfig stdout:\n{stdout}");
        log::error!("ifconfig stderr:\n{stderr}");
        bail!("`ifconfig` exited with code {:?}", result.code);
    }

    Ok(stdout.contains(&alias))
}

/// Get first DNS resolver from `scutil --dns`
async fn get_first_dns_resolver(rpc: &ServiceClient) -> anyhow::Result<IpAddr> {
    let result = rpc.exec("scutil", ["--dns"]).await?;

    let stdout = String::from_utf8(result.stdout)?;
    let stderr = String::from_utf8(result.stderr)?;

    if result.code != Some(0) {
        log::error!("scutil stdout:\n{stdout}");
        log::error!("scutil stderr:\n{stderr}");
        bail!("`scutil` exited with code {:?}", result.code);
    }

    parse_scutil_dns_first_resolver(&stdout).context("No resolver found")
}

fn parse_scutil_dns_first_resolver(output: &str) -> Option<IpAddr> {
    output
        .lines()
        .map(str::trim)
        // nameserver[0] : 127.230.79.91
        .flat_map(|line| line.strip_prefix("nameserver[0]"))
        .flat_map(|server| server.split_whitespace().last())
        .find_map(|addr| addr.parse().ok())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_scutil_dns_first_resolver() {
        let out = r#"resolver #1
  nameserver[0] : 127.230.79.91
  if_index : 11 (en0)
  flags    : Scoped, Request A records
  reach    : 0x00000000 (Not Reachable)"#;

        let aliases = parse_scutil_dns_first_resolver(out);
        assert_eq!(aliases, Some("127.230.79.91".parse::<IpAddr>().unwrap()),);
    }
}
