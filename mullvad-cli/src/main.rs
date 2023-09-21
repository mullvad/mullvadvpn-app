#![deny(rust_2018_idioms)]

#[cfg(all(unix, not(target_os = "android")))]
use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;

mod cmds;
mod format;
use cmds::*;

pub const BIN_NAME: &str = env!("CARGO_BIN_NAME");

#[derive(Debug, Parser)]
#[command(author, version = mullvad_version::VERSION, about, long_about = None)]
#[command(propagate_version = true)]
enum Cli {
    /// Control and display information about your Mullvad account
    #[clap(subcommand)]
    Account(account::Account),

    /// Control the daemon auto-connect setting
    #[clap(subcommand)]
    AutoConnect(auto_connect::AutoConnect),

    /// Receive notifications about beta updates
    #[clap(subcommand)]
    BetaProgram(beta_program::BetaProgram),

    /// Control whether to block network access when disconnected from VPN
    #[clap(subcommand)]
    LockdownMode(lockdown::LockdownMode),

    /// Configure DNS servers to use when connected
    #[clap(subcommand)]
    Dns(dns::Dns),

    /// Control the allow local network sharing setting
    #[clap(subcommand)]
    Lan(lan::Lan),

    /// Connect to a VPN relay
    Connect {
        /// Wait until connected before exiting
        #[arg(long, short = 'w')]
        wait: bool,
    },

    /// Disconnect from the VPN
    Disconnect {
        /// Wait until disconnected before exiting
        #[arg(long, short = 'w')]
        wait: bool,
    },

    /// Reconnect to any matching VPN relay
    Reconnect {
        /// Wait until connected before exiting
        #[arg(long, short = 'w')]
        wait: bool,
    },

    /// Manage use of bridges, socks proxies and Shadowsocks for OpenVPN.
    /// Can make OpenVPN tunnels use Shadowsocks via one of the Mullvad bridge servers.
    /// Can also make OpenVPN connect through any custom SOCKS5 proxy.
    /// These settings also affect how the app reaches the API over Shadowsocks.
    #[clap(subcommand)]
    Bridge(bridge::Bridge),

    /// Manage relay and tunnel constraints
    #[clap(subcommand)]
    Relay(relay::Relay),

    /// Manage use of proxies for reaching the API.
    /// Can make the daemon connect to the the Mullvad API via one of the
    /// Mullvad bridge servers or a custom proxy (SOCKS5 & Shadowsocks).
    #[clap(subcommand)]
    ApiAccess(api_access::ApiAccess),

    /// Manage use of obfuscation protocols for WireGuard.
    /// Can make WireGuard traffic look like something else on the network.
    /// Helps circumvent censorship and to establish a tunnel when on restricted networks
    #[clap(subcommand)]
    Obfuscation(obfuscation::Obfuscation),

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    #[clap(subcommand)]
    SplitTunnel(split_tunnel::SplitTunnel),

    /// Return the state of the VPN tunnel
    Status {
        #[clap(subcommand)]
        cmd: Option<status::Status>,

        #[clap(flatten)]
        args: status::StatusArgs,
    },

    /// Manage tunnel options
    #[clap(subcommand)]
    Tunnel(tunnel::Tunnel),

    /// Show information about the current Mullvad version
    /// and available versions
    Version,

    /// Generate completion scripts for the specified shell
    #[cfg(all(unix, not(target_os = "android")))]
    #[command(hide = true)]
    ShellCompletions {
        /// The shell to generate the script for
        shell: clap_complete::Shell,

        /// Output directory where the shell completions are written
        #[arg(default_value = "./")]
        dir: std::path::PathBuf,
    },

    /// Reset settings, caches, and logs
    FactoryReset,

    /// Manage custom lists
    #[clap(subcommand)]
    CustomList(custom_lists::CustomList),
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    match Cli::parse() {
        Cli::Account(cmd) => cmd.handle().await,
        Cli::Bridge(cmd) => cmd.handle().await,
        Cli::Connect { wait } => tunnel_state::connect(wait).await,
        Cli::Reconnect { wait } => tunnel_state::reconnect(wait).await,
        Cli::Disconnect { wait } => tunnel_state::disconnect(wait).await,
        Cli::AutoConnect(cmd) => cmd.handle().await,
        Cli::BetaProgram(cmd) => cmd.handle().await,
        Cli::LockdownMode(cmd) => cmd.handle().await,
        Cli::Dns(cmd) => cmd.handle().await,
        Cli::Lan(cmd) => cmd.handle().await,
        Cli::Obfuscation(cmd) => cmd.handle().await,
        Cli::ApiAccess(cmd) => cmd.handle().await,
        Cli::Version => version::print().await,
        Cli::FactoryReset => reset::handle().await,
        Cli::Relay(cmd) => cmd.handle().await,
        Cli::Tunnel(cmd) => cmd.handle().await,
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        Cli::SplitTunnel(cmd) => cmd.handle().await,
        Cli::Status { cmd, args } => status::handle(cmd, args).await,
        Cli::CustomList(cmd) => cmd.handle().await,

        #[cfg(all(unix, not(target_os = "android")))]
        Cli::ShellCompletions { shell, dir } => {
            use clap::CommandFactory;

            // FIXME: The shell completions include hidden commands (including "shell-completions")
            println!("Generating shell completions to {}", dir.display());
            clap_complete::generate_to(shell, &mut Cli::command(), BIN_NAME, dir)
                .map_err(|_| anyhow!("Failed to generate shell completions"))?;
            Ok(())
        }
    }
}
