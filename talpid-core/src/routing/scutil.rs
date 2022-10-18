use std::collections::HashMap;

use tokio::process::Command;

use super::{Error, Result};

fn get_default_interface() -> () {
    ()
}

/// Expected to produce output like:
/// Network information
/// IPv4 network interface information
///   utun3 : flags      : 0x5 (IPv4,DNS)
///           address    : 10.113.48.185
///           VPN server : 127.0.0.1
///           reach      : 0x00000003 (Reachable,Transient Connection)
///     en0 : flags      : 0x5 (IPv4,DNS)
///           address    : 192.168.102.106
///           reach      : 0x00000002 (Reachable)
///
///   REACH : flags 0x00000003 (Reachable,Transient Connection)
///
/// IPv6 network interface information
///   No IPv6 states found
///
///
///   REACH : flags 0x00000007 (Reachable,Transient Connection,Connection Required)
///
/// Network interfaces: utun3 en0

///
async fn obtain_output() -> Result<Vec<u8>> {
    let mut cmd = Command::new("scutil");
    cmd.arg("--nwi");

    Ok(cmd.output().await.map_err(|_| Error::ScUtilCommand)?.stdout)
}

fn parse_scutil_output(output: &[u8]) -> Result<Vec<HashMap<&str, &[u8]>>> {
    for line in output.split(|c| *c == b'\n') {}
    Ok(vec![])
}
