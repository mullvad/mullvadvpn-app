use super::{relay::resolve_location_constraint, relay_constraints::LocationArgs};
use anyhow::{anyhow, bail, Result};
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    relay_constraints::{Constraint, GeographicLocationConstraint},
    relay_list::RelayList,
};

#[derive(Subcommand, Debug)]
pub enum CustomList {
    /// Create a new custom list
    New {
        /// A name for the new custom list
        name: String,
    },

    /// Show all custom lists or retrieve a specific custom list
    List {
        // TODO: Would be cool to provide dynamic auto-completion:
        // https://github.com/clap-rs/clap/issues/1232
        /// A custom list. If omitted, all custom lists are shown
        name: Option<String>,
    },

    /// Edit a custom list
    #[clap(subcommand)]
    Edit(EditCommand),

    /// Delete a custom list
    Delete {
        /// A custom list
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum EditCommand {
    /// Add a location to some custom list
    Add {
        /// A custom list
        name: String,
        #[command(flatten)]
        location: LocationArgs,
    },

    /// Remove a location from some custom list
    Remove {
        /// A custom list
        name: String,
        #[command(flatten)]
        location: LocationArgs,
    },

    /// Rename a custom list
    Rename {
        /// Current name of the custom list
        name: String,
        /// A new name for the custom list
        new_name: String,
    },
}

impl CustomList {
    pub async fn handle(self) -> Result<()> {
        match self {
            CustomList::List { name: None } => Self::list().await,
            CustomList::List { name: Some(name) } => Self::get(name).await,
            CustomList::New { name } => Self::create_list(name).await,
            CustomList::Delete { name } => Self::delete_list(name).await,
            CustomList::Edit(cmd) => match cmd {
                EditCommand::Add { name, location } => Self::add_location(name, location).await,
                EditCommand::Rename { name, new_name } => Self::rename_list(name, new_name).await,
                EditCommand::Remove { name, location } => {
                    Self::remove_location(name, location).await
                }
            },
        }
    }

    /// Print all custom lists.
    async fn list() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let cache = rpc.get_relay_locations().await?;
        for custom_list in rpc.get_settings().await?.custom_lists {
            Self::print_custom_list(&custom_list, &cache)
        }
        Ok(())
    }

    /// Print a specific custom list (if it exists).
    /// If the list does not exist, print an error.
    async fn get(name: String) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let custom_list = find_list_by_name(&mut rpc, &name).await?;
        let cache = rpc.get_relay_locations().await?;
        Self::print_custom_list_content(&custom_list, &cache);
        Ok(())
    }

    async fn create_list(name: String) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.create_custom_list(name).await?;
        Ok(())
    }

    async fn add_location(name: String, location_args: LocationArgs) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;

        // Don't filter out any hosts, i.e. allow adding even inactive ones
        let relay_filter = |_: &_| true;
        let location_constraint =
            resolve_location_constraint(&mut rpc, location_args, relay_filter).await?;

        match location_constraint {
            Constraint::Any => bail!("\"any\" is not a valid location"),
            Constraint::Only(location) => {
                let mut list = find_list_by_name(&mut rpc, &name).await?;
                if list.locations.insert(location) {
                    rpc.update_custom_list(list).await?;
                    println!("Location added to custom-list")
                } else {
                    bail!("Provided location is already present in custom-list")
                };
            }
        }

        Ok(())
    }

    async fn remove_location(name: String, location_args: LocationArgs) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;

        // Don't filter out any hosts, i.e. allow adding even inactive ones
        let relay_filter = |_: &_| true;
        let location_constraint =
            resolve_location_constraint(&mut rpc, location_args, relay_filter).await?;

        match location_constraint {
            Constraint::Any => bail!("\"any\" is not a valid location"),
            Constraint::Only(location) => {
                let mut list = find_list_by_name(&mut rpc, &name).await?;
                if list.locations.remove(&location) {
                    rpc.update_custom_list(list).await?;
                    println!("Location removed from custom-list")
                } else {
                    bail!("Provided location was not present in custom-list")
                };
            }
        }

        Ok(())
    }

    async fn delete_list(name: String) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let list = find_list_by_name(&mut rpc, &name).await?;
        rpc.delete_custom_list(list.id.to_string()).await?;
        Ok(())
    }

    async fn rename_list(name: String, new_name: String) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;

        let mut list = find_list_by_name(&mut rpc, &name).await?;
        list.name = new_name;
        rpc.update_custom_list(list).await?;

        Ok(())
    }

    fn print_custom_list(custom_list: &mullvad_types::custom_list::CustomList, cache: &RelayList) {
        println!("{}", custom_list.name);
        Self::print_custom_list_content(custom_list, cache);
    }

    fn print_custom_list_content(
        custom_list: &mullvad_types::custom_list::CustomList,
        cache: &RelayList,
    ) {
        for location in &custom_list.locations {
            println!(
                "\t{}",
                GeographicLocationConstraintFormatter::from_constraint(location, cache)
            );
        }
    }
}

/// Struct used for pretty printing [`GeographicLocationConstraint`] with
/// human-readable names for countries and cities.
pub struct GeographicLocationConstraintFormatter<'a> {
    constraint: &'a GeographicLocationConstraint,
    country: Option<String>,
    city: Option<String>,
}

impl<'a> GeographicLocationConstraintFormatter<'a> {
    fn from_constraint(constraint: &'a GeographicLocationConstraint, cache: &RelayList) -> Self {
        use GeographicLocationConstraint::*;
        let (country_code, city_code) = match constraint {
            Country(country) => (Some(country), None),
            City(country, city) | Hostname(country, city, _) => (Some(country), Some(city)),
        };

        let country =
            country_code.and_then(|country_code| cache.lookup_country(country_code.to_string()));
        let city = city_code.and_then(|city_code| {
            country.and_then(|country| country.lookup_city(city_code.to_string()))
        });

        Self {
            constraint,
            country: country.map(|x| x.name.clone()),
            city: city.map(|x| x.name.clone()),
        }
    }
}

impl<'a> std::fmt::Display for GeographicLocationConstraintFormatter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let unwrap_country = |country: Option<String>, constraint: &str| {
            country.unwrap_or(format!("{constraint} <invalid country>"))
        };

        let unwrap_city = |city: Option<String>, constraint: &str| {
            city.unwrap_or(format!("{constraint} <invalid city>"))
        };

        match &self.constraint {
            GeographicLocationConstraint::Country(country) => {
                let rich_country = unwrap_country(self.country.clone(), country);
                write!(f, "{rich_country} ({country})")
            }
            GeographicLocationConstraint::City(country, city) => {
                let rich_country = unwrap_country(self.country.clone(), country);
                let rich_city = unwrap_city(self.city.clone(), city);
                write!(f, "{rich_city}, {rich_country} ({city}, {country})")
            }
            GeographicLocationConstraint::Hostname(country, city, hostname) => {
                let rich_country = unwrap_country(self.country.clone(), country);
                let rich_city = unwrap_city(self.city.clone(), city);
                write!(
                    f,
                    "{hostname} in {rich_city}, {rich_country} ({city}, {country})"
                )
            }
        }
    }
}

pub async fn find_list_by_name(
    rpc: &mut MullvadProxyClient,
    name: &str,
) -> Result<mullvad_types::custom_list::CustomList> {
    rpc.get_settings()
        .await?
        .custom_lists
        .into_iter()
        .find(|list| list.name == name)
        .ok_or(anyhow!("List not found"))
}
