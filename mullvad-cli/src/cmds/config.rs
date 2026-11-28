use anyhow::{Context, Result, bail};
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::wireguard::WireguardIdentityInfo;
use std::io::Read;

#[derive(Subcommand, Debug)]
pub enum Config {
    /// Import a WireGuard config file generated at mullvad.net.
    ///
    /// Only the private key and the tunnel addresses (the `Address` line) are used. The relay,
    /// DNS servers and so on are chosen by the app, just like in the regular Mullvad app. Any
    /// previously imported config is replaced.
    Import {
        /// Path to the config file, or "-" to read it from standard input
        file: String,
    },

    /// Set the config from a private key and tunnel addresses given directly.
    ///
    /// The tunnel addresses are found in the `Address` line of a config file generated at
    /// mullvad.net. They are assigned by Mullvad to your key and cannot be derived from it.
    Set {
        /// The base64 encoded WireGuard private key
        #[arg(long)]
        private_key: String,

        /// A tunnel address, for example 10.66.1.2/32. Repeat the option to also give the IPv6
        /// address.
        #[arg(long = "address", required = true)]
        addresses: Vec<String>,
    },

    /// Show the public key and tunnel addresses of the imported config
    Show,

    /// Forget the imported config. This also disconnects the VPN.
    Clear,
}

impl Config {
    pub async fn handle(self) -> Result<()> {
        match self {
            Config::Import { file } => {
                let config = if file == "-" {
                    let mut buffer = String::new();
                    std::io::stdin()
                        .read_to_string(&mut buffer)
                        .context("Failed to read the config from standard input")?;
                    buffer
                } else {
                    tokio::fs::read_to_string(&file)
                        .await
                        .with_context(|| format!("Failed to read {file}"))?
                };
                import(config).await
            }
            Config::Set {
                private_key,
                addresses,
            } => {
                if private_key.contains(['\n', '\r']) || addresses.iter().any(|a| a.contains(['\n', '\r'])) {
                    bail!("The private key and addresses must not contain line breaks");
                }
                let config = format!(
                    "[Interface]\nPrivateKey = {private_key}\nAddress = {}\n",
                    addresses.join(",")
                );
                import(config).await
            }
            Config::Show => {
                let mut rpc = MullvadProxyClient::new().await?;
                match rpc.get_wireguard_identity().await? {
                    Some(identity) => print_identity(&identity),
                    None => println!("No WireGuard config has been imported"),
                }
                Ok(())
            }
            Config::Clear => {
                let mut rpc = MullvadProxyClient::new().await?;
                rpc.clear_wireguard_identity().await?;
                println!("Removed the WireGuard config");
                Ok(())
            }
        }
    }
}

async fn import(config: String) -> Result<()> {
    let mut rpc = MullvadProxyClient::new().await?;
    let identity = rpc.import_wireguard_config(config).await?;
    println!("Imported WireGuard config");
    print_identity(&identity);
    Ok(())
}

fn print_identity(identity: &WireguardIdentityInfo) {
    println!("Public key:   {}", identity.public_key.key);
    println!("IPv4 address: {}", identity.ipv4_address);
    match identity.ipv6_address {
        Some(address) => println!("IPv6 address: {address}"),
        None => println!("IPv6 address: none"),
    }
}
