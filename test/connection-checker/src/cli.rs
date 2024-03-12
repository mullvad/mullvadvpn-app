use std::net::SocketAddr;

use clap::Parser;

/// CLI tool that queries <https://am.i.mullvad.net> to check if the machine is connected to
/// Mullvad VPN.
#[derive(Parser)]
pub struct Opt {
    /// Interactive mode, press enter to check if you are Mullvad.
    #[clap(short, long)]
    pub interactive: bool,

    /// Timeout for network connections (in millis).
    #[clap(short, long, default_value = "2000")]
    pub timeout: u64,

    /// Try to send some junk data over TCP to <leak>.
    #[clap(long, requires = "leak")]
    pub leak_tcp: bool,

    /// Try to send some junk data over UDP to <leak>.
    #[clap(long, requires = "leak")]
    pub leak_udp: bool,

    /// Try to send ICMP request to <leak>.
    #[clap(long, requires = "leak")]
    pub leak_icmp: bool,

    /// Target of <leak_tcp>, <leak_udp> or <leak_icmp>.
    #[clap(long)]
    pub leak: Option<SocketAddr>,
}
