use std::{net::IpAddr, str};

use anyhow::{anyhow, Context};
use futures::{select, stream::FuturesUnordered, FutureExt, StreamExt};

use tokio::time::sleep;

use crate::{
    traceroute::{TracerouteOpt, DEFAULT_TTL_RANGE, LEAK_TIMEOUT, PROBE_INTERVAL, SEND_TIMEOUT},
    util::{get_interface_ip, Ip},
    LeakInfo, LeakStatus,
};

/// Implementation of traceroute using `ping.exe`
///
/// This monstrosity exists because the Windows firewall is not helpful enough to allow us to
/// permit a process (the daemon) to receive ICMP TimeExceeded packets. We can get around this by
/// using `ping.exe`, which does work for some reason. My best guess is that it has special kernel
/// access to be able to do this.
pub async fn traceroute_using_ping(opt: &TracerouteOpt) -> anyhow::Result<LeakStatus> {
    let ip_version = match opt.destination {
        IpAddr::V4(..) => Ip::v4(),
        IpAddr::V6(..) => Ip::v6(),
    };

    let interface_ip = get_interface_ip(&opt.interface, ip_version)?;

    let mut ping_tasks = FuturesUnordered::new();

    for (i, ttl) in DEFAULT_TTL_RANGE.enumerate() {
        // Don't send all pings at once, wait a bit in between
        // each one to avoid sending more than necessary
        let probe_delay = PROBE_INTERVAL * i as u32;

        ping_tasks.push(async move {
            sleep(probe_delay).await;

            log::debug!("sending probe packet (ttl={ttl})");

            // ping.exe will send ICMP Echo packets to the destination, and since it's running in
            // the kernel it will be able to receive TimeExceeded responses.
            let ping_path = r"C:\Windows\System32\ping.exe";
            let output = tokio::process::Command::new(ping_path)
                .args(["-i", &ttl.to_string()])
                .args(["-n", "1"]) // number of pings
                .args(["-w", &SEND_TIMEOUT.as_millis().to_string()])
                .args(["-S", &interface_ip.to_string()]) // bind to interface IP
                .arg(opt.destination.to_string())
                .kill_on_drop(true)
                .output()
                .await
                .context(anyhow!("Failed to execute {ping_path}"))?;

            let output_err = || anyhow!("Unexpected output from `ping.exe`");

            let stdout = str::from_utf8(&output.stdout).with_context(output_err)?;
            let _stderr = str::from_utf8(&output.stderr).with_context(output_err)?;

            log::trace!("ping stdout: {stdout}");
            log::trace!("ping stderr: {_stderr}");

            // Dumbly parse stdout for a line that looks like this:
            // "Reply from <ip>: TTL expired"

            if !stdout.contains("TTL expired") {
                // No "TTL expired" means we did not receive any TimeExceeded replies.
                return Ok(None);
            }
            let (ip, ..) = stdout
                .split_once("Reply from ")
                .and_then(|(.., s)| s.split_once(": TTL expired"))
                .with_context(output_err)?;

            let ip: IpAddr = ip
                .parse()
                .context("`ping.exe` outputted an invalid IP address")?;

            anyhow::Ok(Some(ip))
        });
    }

    let wait_for_first_leak = async move {
        while let Some(result) = ping_tasks.next().await {
            let Some(ip) = result? else { continue };

            return Ok(LeakStatus::LeakDetected(
                LeakInfo::NodeReachableOnInterface {
                    reachable_nodes: vec![ip],
                    interface: opt.interface.clone(),
                },
            ));
        }

        anyhow::Ok(LeakStatus::NoLeak)
    };

    select! {
        _ = sleep(LEAK_TIMEOUT).fuse() => Ok(LeakStatus::NoLeak),
        result = wait_for_first_leak.fuse() => result,
    }
}
