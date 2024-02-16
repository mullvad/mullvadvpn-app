use anyhow::{bail, Result};
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    constraints::Constraint,
    relay_constraints::{
        BridgeConstraintsFormatter, BridgeState, BridgeType, LocationConstraint, Ownership,
        Provider, Providers,
    },
    relay_list::RelayEndpointData,
};
use talpid_types::net::proxy::{CustomProxy, Shadowsocks, Socks5Local, Socks5Remote};

use crate::cmds::proxies::pp::CustomProxyFormatter;

use super::{
    proxies::{ProxyEditParams, ShadowsocksAdd, Socks5LocalAdd, Socks5RemoteAdd},
    relay::resolve_location_constraint,
    relay_constraints::LocationArgs,
};

#[derive(Subcommand, Debug)]
pub enum Bridge {
    /// Get current bridge settings
    Get,
    /// Set bridge state and settings, such as provider
    #[clap(subcommand)]
    Set(SetCommands),
    /// List available bridge relays
    List,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SetCommands {
    /// Specify whether to use a bridge
    State { policy: BridgeState },

    /// Set country or city to select relays from.
    /// Use the 'mullvad bridge list' command to show available alternatives.
    #[command(
        override_usage = "mullvad bridge set location <COUNTRY> [CITY] [HOSTNAME] | <HOSTNAME>

  Select bridge using a country:

\tmullvad bridge set location se

  Select bridge using a country and city:

\tmullvad bridge set location se got

  Select bridge using a country, city and hostname:

\tmullvad bridge set location se got se-got-br-001

  Select bridge using only its hostname:

\tmullvad bridge set location se-got-br-001"
    )]
    Location(LocationArgs),

    /// Set custom list to select relays from. Use the 'custom-lists list'
    /// command to show available alternatives.
    CustomList { custom_list_name: String },

    /// Set hosting provider(s) to select relays from. The 'list'
    /// command shows the available relays and their providers.
    Provider {
        /// Either 'any', or provider to select from.
        #[arg(required(true), num_args = 1..)]
        providers: Vec<Provider>,
    },

    /// Filter relays based on ownership. The 'list' command
    /// shows the available relays and whether they're rented.
    Ownership {
        /// Servers to select from: 'any', 'owned', or 'rented'.
        ownership: Constraint<Ownership>,
    },

    /// Configure a SOCKS5 proxy
    #[clap(subcommand)]
    Custom(CustomCommands),
}

#[derive(Subcommand, Debug, Clone)]
pub enum CustomCommands {
    /// Create or update and enable the custom bridge configuration.
    #[clap(subcommand)]
    Set(AddCustomCommands),
    /// Edit an existing custom bridge configuration.
    Edit(ProxyEditParams),
    /// Use an existing custom bridge configuration.
    Use,
    /// Stop using the custom bridge configuration.
    Disable,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AddCustomCommands {
    #[clap(subcommand)]
    Socks5(AddSocks5Commands),
    /// Configure bundled Shadowsocks proxy
    Shadowsocks {
        #[clap(flatten)]
        add: ShadowsocksAdd,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum AddSocks5Commands {
    /// Configure a local SOCKS5 proxy
    #[clap(
        about = "Registers a local SOCKS5 proxy. Will allow all local programs to leak traffic *only* to the remote endpoint."
    )]
    Local {
        #[clap(flatten)]
        add: Socks5LocalAdd,
    },

    /// Configure a remote SOCKS5 proxy
    Remote {
        #[clap(flatten)]
        add: Socks5RemoteAdd,
    },
}

impl Bridge {
    pub async fn handle(self) -> Result<()> {
        match self {
            Bridge::Get => Self::get().await,
            Bridge::List => Self::list().await,
            Bridge::Set(subcmd) => Self::set(subcmd).await,
        }
    }

    async fn set(subcmd: SetCommands) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        match subcmd {
            SetCommands::State { policy } => {
                rpc.set_bridge_state(policy).await?;
                println!("Updated bridge state");
                Ok(())
            }
            SetCommands::Location(location_constraint_args) => {
                let relay_filter = |relay: &mullvad_types::relay_list::Relay| {
                    relay.active && relay.endpoint_data == RelayEndpointData::Bridge
                };
                let location_constraint =
                    resolve_location_constraint(&mut rpc, location_constraint_args, relay_filter)
                        .await?
                        .map(LocationConstraint::from);
                Self::update_bridge_settings(&mut rpc, Some(location_constraint), None, None).await
            }
            SetCommands::CustomList { custom_list_name } => {
                let list =
                    super::custom_list::find_list_by_name(&mut rpc, &custom_list_name).await?;
                let location =
                    Constraint::Only(LocationConstraint::CustomList { list_id: list.id });
                Self::update_bridge_settings(&mut rpc, Some(location), None, None).await
            }
            SetCommands::Ownership { ownership } => {
                Self::update_bridge_settings(&mut rpc, None, None, Some(ownership)).await
            }
            SetCommands::Provider { providers } => {
                let providers = if providers[0].eq_ignore_ascii_case("any") {
                    Constraint::Any
                } else {
                    Constraint::Only(Providers::new(providers.into_iter()).unwrap())
                };
                Self::update_bridge_settings(&mut rpc, None, Some(providers), None).await
            }
            SetCommands::Custom(subcmd) => Self::handle_custom(subcmd).await,
        }
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let settings = rpc.get_settings().await?;
        println!("Bridge state: {}", settings.bridge_state);
        println!(
            "Active bridge type: {}",
            settings.bridge_settings.bridge_type
        );

        println!("Normal constraints");
        println!(
            "{:<4}{}",
            "",
            BridgeConstraintsFormatter {
                constraints: &settings.bridge_settings.normal,
                custom_lists: &settings.custom_lists
            }
        );

        if let Some(ref custom_proxy) = settings.bridge_settings.custom {
            println!("Custom proxy");
            println!("{}", CustomProxyFormatter { custom_proxy });
        }

        Ok(())
    }

    async fn list() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let relay_list = rpc.get_relay_locations().await?;

        let mut countries = Vec::new();

        for mut country in relay_list.countries {
            country.cities = country
                .cities
                .into_iter()
                .filter_map(|mut city| {
                    city.relays.retain(|relay| {
                        relay.active && matches!(relay.endpoint_data, RelayEndpointData::Bridge)
                    });
                    if !city.relays.is_empty() {
                        Some(city)
                    } else {
                        None
                    }
                })
                .collect();
            if !country.cities.is_empty() {
                countries.push(country);
            }
        }

        countries.sort_by(|c1, c2| natord::compare_ignore_case(&c1.name, &c2.name));
        for mut country in countries {
            country
                .cities
                .sort_by(|c1, c2| natord::compare_ignore_case(&c1.name, &c2.name));
            println!("{} ({})", country.name, country.code);
            for mut city in country.cities {
                city.relays
                    .sort_by(|r1, r2| natord::compare_ignore_case(&r1.hostname, &r2.hostname));
                println!(
                    "\t{} ({}) @ {:.5}°N, {:.5}°W",
                    city.name, city.code, city.latitude, city.longitude
                );
                for relay in &city.relays {
                    let ownership = if relay.owned {
                        "Mullvad-owned"
                    } else {
                        "rented"
                    };
                    println!(
                        "\t\t{} ({}) - hosted by {} ({ownership})",
                        relay.hostname, relay.ipv4_addr_in, relay.provider
                    );
                }
            }
            println!();
        }
        Ok(())
    }

    async fn update_bridge_settings(
        rpc: &mut MullvadProxyClient,
        location: Option<Constraint<LocationConstraint>>,
        providers: Option<Constraint<Providers>>,
        ownership: Option<Constraint<Ownership>>,
    ) -> Result<()> {
        let mut settings = rpc.get_settings().await?.bridge_settings;
        if let Some(new_location) = location {
            settings.normal.location = new_location;
        }
        if let Some(new_providers) = providers {
            settings.normal.providers = new_providers;
        }
        if let Some(new_ownership) = ownership {
            settings.normal.ownership = new_ownership;
        }

        settings.bridge_type = BridgeType::Normal;

        rpc.set_bridge_settings(settings).await?;

        println!("Updated bridge settings");

        Ok(())
    }

    pub async fn handle_custom(subcmd: CustomCommands) -> Result<()> {
        match subcmd {
            CustomCommands::Set(set_custom_commands) => {
                Self::custom_bridge_set(set_custom_commands).await
            }
            CustomCommands::Edit(edit) => Self::custom_bridge_edit(edit).await,
            CustomCommands::Use => Self::custom_bridge_use().await,
            CustomCommands::Disable => Self::custom_bridge_disable().await,
        }
    }

    async fn custom_bridge_edit(edit: ProxyEditParams) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let mut settings = rpc.get_settings().await?;

        let Some(ref mut custom_bridge) = settings.bridge_settings.custom else {
            bail!("Can not edit as there is no currently saved custom bridge");
        };

        match custom_bridge {
            CustomProxy::Shadowsocks(ss) => *ss = edit.merge_shadowsocks(ss),
            CustomProxy::Socks5Local(local) => *local = edit.merge_socks_local(local),
            CustomProxy::Socks5Remote(remote) => *remote = edit.merge_socks_remote(remote)?,
        };

        rpc.set_bridge_settings(settings.bridge_settings)
            .await
            .map_err(anyhow::Error::from)
    }

    async fn custom_bridge_use() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;

        let mut settings = rpc.get_settings().await?;
        if settings.bridge_settings.custom.is_none() {
            bail!("Cannot enable custom bridge as there are no settings");
        }
        settings.bridge_settings.bridge_type = BridgeType::Custom;
        rpc.set_bridge_settings(settings.bridge_settings).await?;

        Ok(())
    }

    async fn custom_bridge_disable() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let mut settings = rpc.get_settings().await?;

        settings.bridge_settings.bridge_type = BridgeType::Normal;

        rpc.set_bridge_settings(settings.bridge_settings).await?;
        Ok(())
    }

    async fn custom_bridge_set(set_commands: AddCustomCommands) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let mut settings = rpc.get_settings().await?;

        settings.bridge_settings.custom = Some(match set_commands {
            AddCustomCommands::Socks5(AddSocks5Commands::Local { add }) => {
                CustomProxy::Socks5Local(Socks5Local::from(add))
            }
            AddCustomCommands::Socks5(AddSocks5Commands::Remote { add }) => {
                CustomProxy::Socks5Remote(Socks5Remote::try_from(add)?)
            }
            AddCustomCommands::Shadowsocks { add } => {
                CustomProxy::Shadowsocks(Shadowsocks::from(add))
            }
        });

        settings.bridge_settings.bridge_type = BridgeType::Custom;

        rpc.set_bridge_settings(settings.bridge_settings)
            .await
            .map_err(anyhow::Error::from)
    }
}
