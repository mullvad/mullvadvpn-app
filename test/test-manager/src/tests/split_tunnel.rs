use anyhow::{anyhow, bail, ensure, Context};
use mullvad_management_interface::MullvadProxyClient;
use pcap::Direction;
use pnet_packet::ip::IpNextHeaderProtocols;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str,
    time::Duration,
};
use test_macro::test_function;
use test_rpc::{meta::Os, ServiceClient, SpawnOpts};
use tokio::time::{sleep, timeout};

use crate::network_monitor::{start_packet_monitor, MonitorOptions};

use super::{config::TEST_CONFIG, helpers, TestContext};

const CHECKER_FILENAME_WINDOWS: &str = "connection-checker.exe";
const CHECKER_FILENAME_UNIX: &str = "connection-checker";
const LEAK_DESTINATION: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 1337);

const AM_I_MULLVAD_TIMEOUT_MS: u64 = 10000;
const LEAK_TIMEOUT_MS: u64 = 500;

/// Timeout of [ConnCheckerHandle::check_connection].
const CONN_CHECKER_TIMEOUT: Duration = Duration::from_millis(
    AM_I_MULLVAD_TIMEOUT_MS // https://am.i.mullvad.net timeout
    + LEAK_TIMEOUT_MS // leak-tcp timeout
    + LEAK_TIMEOUT_MS // leak-icmp timeout
    + 1000, // plus some extra grace time
);

/// Test that split tunneling works by asserting the following:
/// - Splitting a process shouldn't do anything if tunnel is not connected.
/// - A split process should never push traffic through the tunnel.
/// - Splitting/unsplitting should work regardless if process is running.
#[test_function]
pub async fn test_split_tunnel(
    _ctx: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    let mut checker = ConnChecker::new(rpc.clone(), mullvad_client.clone());

    // Test that program is behaving when we are disconnected
    (checker.spawn().await?.assert_insecure().await)
        .with_context(|| "Test disconnected and unsplit")?;
    checker.split().await?;
    (checker.spawn().await?.assert_insecure().await)
        .with_context(|| "Test disconnected and split")?;
    checker.unsplit().await?;

    // Test that program is behaving being split/unsplit while running and we are disconnected
    let mut handle = checker.spawn().await?;
    handle.split().await?;
    (handle.assert_insecure().await)
        .with_context(|| "Test disconnected and being split while running")?;
    handle.unsplit().await?;
    (handle.assert_insecure().await)
        .with_context(|| "Test disconnected and being unsplit while running")?;
    drop(handle);

    helpers::connect_and_wait(&mut mullvad_client).await?;

    // Test running an unsplit program
    checker
        .spawn()
        .await?
        .assert_secure()
        .await
        .with_context(|| "Test connected and unsplit")?;

    // Test running a split program
    checker.split().await?;
    checker
        .spawn()
        .await?
        .assert_insecure()
        .await
        .with_context(|| "Test connected and split")?;

    checker.unsplit().await?;

    // Test splitting and unsplitting a program while it's running
    let mut handle = checker.spawn().await?;
    (handle.assert_secure().await).with_context(|| "Test connected and unsplit (again)")?;
    handle.split().await?;
    (handle.assert_insecure().await)
        .with_context(|| "Test connected and being split while running")?;
    handle.unsplit().await?;
    (handle.assert_secure().await)
        .with_context(|| "Test connected and being unsplit while running")?;

    Ok(())
}

/// This helper spawns a seperate process which checks if we are connected to Mullvad, and tries to
/// leak traffic outside the tunnel by sending TCP, UDP, and ICMP packets to [LEAK_DESTINATION].
struct ConnChecker {
    rpc: ServiceClient,
    mullvad_client: MullvadProxyClient,

    /// Path to the process binary.
    executable_path: String,

    /// Whether the process should be split when spawned. Needed on Linux.
    split: bool,
}

struct ConnCheckerHandle<'a> {
    checker: &'a mut ConnChecker,

    /// ID of the spawned process.
    pid: u32,
}

struct ConnectionStatus {
    /// True if <https://am.i.mullvad.net/> reported we are connected.
    am_i_mullvad: bool,

    /// True if we sniffed TCP packets going outside the tunnel.
    leaked_tcp: bool,

    /// True if we sniffed UDP packets going outside the tunnel.
    leaked_udp: bool,

    /// True if we sniffed ICMP packets going outside the tunnel.
    leaked_icmp: bool,
}

impl ConnChecker {
    pub fn new(rpc: ServiceClient, mullvad_client: MullvadProxyClient) -> Self {
        let artifacts_dir = &TEST_CONFIG.artifacts_dir;
        let executable_path = match TEST_CONFIG.os {
            Os::Linux | Os::Macos => format!("{artifacts_dir}/{CHECKER_FILENAME_UNIX}"),
            Os::Windows => format!("{artifacts_dir}\\{CHECKER_FILENAME_WINDOWS}"),
        };

        Self {
            rpc,
            mullvad_client,
            split: false,
            executable_path,
        }
    }

    /// Spawn the connecton checker process and return a handle to it.
    ///
    /// Dropping the handle will stop the process.
    /// **NOTE**: The handle must be dropped from a tokio runtime context.
    pub async fn spawn(&mut self) -> anyhow::Result<ConnCheckerHandle<'_>> {
        log::debug!("spawning connection checker");

        let opts = SpawnOpts {
            attach_stdin: true,
            attach_stdout: true,
            args: [
                "--interactive",
                "--timeout",
                &AM_I_MULLVAD_TIMEOUT_MS.to_string(),
                // try to leak traffic to LEAK_DESTINATION
                "--leak",
                &LEAK_DESTINATION.to_string(),
                "--leak-timeout",
                &LEAK_TIMEOUT_MS.to_string(),
                "--leak-tcp",
                "--leak-udp",
                "--leak-icmp",
            ]
            .map(String::from)
            .to_vec(),
            ..SpawnOpts::new(&self.executable_path)
        };

        let pid = self.rpc.spawn(opts).await?;

        if self.split && TEST_CONFIG.os == Os::Linux {
            self.mullvad_client
                .add_split_tunnel_process(pid as i32)
                .await?;
        }

        Ok(ConnCheckerHandle { pid, checker: self })
    }

    /// Enable split tunneling for the connection checker.
    pub async fn split(&mut self) -> anyhow::Result<()> {
        log::debug!("enable split tunnel");
        self.split = true;

        match TEST_CONFIG.os {
            Os::Linux => { /* linux programs can't be split in the management interface until they are spawned */ }
            Os::Windows | Os::Macos => {
                self.mullvad_client
                    .add_split_tunnel_app(&self.executable_path)
                    .await?;
                self.mullvad_client.set_split_tunnel_state(true).await?;
            }
        }

        Ok(())
    }

    /// Disable split tunneling for the connection checker.
    pub async fn unsplit(&mut self) -> anyhow::Result<()> {
        log::debug!("disable split tunnel");
        self.split = false;

        match TEST_CONFIG.os {
            Os::Linux => {}
            Os::Windows | Os::Macos => {
                self.mullvad_client.set_split_tunnel_state(false).await?;
                self.mullvad_client
                    .remove_split_tunnel_app(&self.executable_path)
                    .await?;
            }
        }

        Ok(())
    }
}

impl ConnCheckerHandle<'_> {
    pub async fn split(&mut self) -> anyhow::Result<()> {
        if TEST_CONFIG.os == Os::Linux {
            self.checker
                .mullvad_client
                .add_split_tunnel_process(self.pid as i32)
                .await?;
        }

        self.checker.split().await
    }

    pub async fn unsplit(&mut self) -> anyhow::Result<()> {
        if TEST_CONFIG.os == Os::Linux {
            self.checker
                .mullvad_client
                .remove_split_tunnel_process(self.pid as i32)
                .await?;
        }

        self.checker.unsplit().await
    }

    /// Assert that traffic is flowing through the Mullvad tunnel and that no packets are leaked.
    pub async fn assert_secure(&mut self) -> anyhow::Result<()> {
        log::info!("checking that connection is secure");
        let status = self.check_connection().await?;
        ensure!(status.am_i_mullvad);
        ensure!(!status.leaked_tcp);
        ensure!(!status.leaked_udp);
        ensure!(!status.leaked_icmp);

        Ok(())
    }

    /// Assert that traffic is NOT flowing through the Mullvad tunnel and that packets ARE leaked.
    pub async fn assert_insecure(&mut self) -> anyhow::Result<()> {
        log::info!("checking that connection is not secure");
        let status = self.check_connection().await?;
        ensure!(!status.am_i_mullvad);
        ensure!(status.leaked_tcp);
        ensure!(status.leaked_udp);
        ensure!(status.leaked_icmp);

        Ok(())
    }

    async fn check_connection(&mut self) -> anyhow::Result<ConnectionStatus> {
        // Monitor all pakets going to LEAK_DESTINATION during the check.
        let monitor = start_packet_monitor(
            |packet| packet.destination.ip() == LEAK_DESTINATION.ip(),
            MonitorOptions {
                direction: Some(Direction::In),
                ..MonitorOptions::default()
            },
        )
        .await;

        // Write a newline to the connection checker to prompt it to perform the check.
        self.checker
            .rpc
            .write_child_stdin(self.pid, "Say the line, Bart!\r\n".into())
            .await?;

        // The checker responds when the check is complete.
        let line = self.read_stdout_line().await?;

        let monitor_result = monitor
            .into_result()
            .await
            .map_err(|_e| anyhow!("Packet monitor unexpectedly stopped"))?;

        Ok(ConnectionStatus {
            am_i_mullvad: parse_am_i_mullvad(line)?,

            leaked_tcp: (monitor_result.packets.iter())
                .any(|pkt| pkt.protocol == IpNextHeaderProtocols::Tcp),

            leaked_udp: (monitor_result.packets.iter())
                .any(|pkt| pkt.protocol == IpNextHeaderProtocols::Udp),

            leaked_icmp: (monitor_result.packets.iter())
                .any(|pkt| pkt.protocol == IpNextHeaderProtocols::Icmp),
        })
    }

    /// Try to a single line of output from the spawned process
    async fn read_stdout_line(&mut self) -> anyhow::Result<String> {
        // Add a timeout to avoid waiting forever.
        timeout(CONN_CHECKER_TIMEOUT, async {
            let mut line = String::new();

            // tarpc doesn't support streams, so we poll the checker process in a loop instead
            loop {
                let Some(output) = self.checker.rpc.read_child_stdout(self.pid).await? else {
                    bail!("got EOF from connection checker process");
                };

                if output.is_empty() {
                    sleep(Duration::from_millis(500)).await;
                    continue;
                }

                line.push_str(&output);

                if line.contains('\n') {
                    log::info!("output from child process: {output:?}");
                    return Ok(line);
                }
            }
        })
        .await
        .with_context(|| "Timeout reading stdout from connection checker")?
    }
}

impl Drop for ConnCheckerHandle<'_> {
    fn drop(&mut self) {
        let rpc = self.checker.rpc.clone();
        let pid = self.pid;

        let Ok(runtime_handle) = tokio::runtime::Handle::try_current() else {
            log::error!("ConnCheckerHandle dropped outside of a tokio runtime.");
            return;
        };

        runtime_handle.spawn(async move {
            // Make sure child process is stopped when this handle is dropped.
            // Closing stdin does the trick.
            let _ = rpc.close_child_stdin(pid).await;
        });
    }
}

/// Parse output from connection-checker. Returns true if connected to Mullvad.
fn parse_am_i_mullvad(result: String) -> anyhow::Result<bool> {
    Ok(if result.contains("You are connected") {
        true
    } else if result.contains("You are not connected") {
        false
    } else {
        bail!("Unexpected output from connection-checker: {result:?}")
    })
}
