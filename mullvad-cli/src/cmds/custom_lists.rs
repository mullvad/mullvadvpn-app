use super::relay_constraints::LocationArgs;
use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    custom_list::CustomListLocationUpdate,
    relay_constraints::{Constraint, GeographicLocationConstraint, RelayConstraintsUpdate, RelaySettingsUpdate, LocationConstraint},
};

#[derive(Subcommand, Debug)]
pub enum CustomList {
    /// Get names of custom lists
    List,

    /// Retrieve a custom list by its name
    Get { name: String },

    /// Create a new custom list
    Create { name: String },

    /// Add a location to the list
    Add {
        name: String,
        #[command(flatten)]
        location: LocationArgs,
    },

    /// Remove a location from the list
    Remove {
        name: String,
        #[command(flatten)]
        location: LocationArgs,
    },

    /// Delete the custom list
    Delete { name: String },

    /// Use a random relay from the custom list
    Select { name: String },
}

impl CustomList {
    pub async fn handle(self) -> Result<()> {
        match self {
            CustomList::List => Self::list().await,
            CustomList::Get { name } => Self::get(name).await,
            CustomList::Create { name } => Self::create_list(name).await,
            CustomList::Add { name, location } => Self::add_location(name, location).await,
            CustomList::Remove { name, location } => Self::remove_location(name, location).await,
            CustomList::Delete { name } => Self::delete_list(name).await,
            CustomList::Select { name } => Self::select_list(name).await,
        }
    }

    async fn list() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        for custom_list in rpc.list_custom_lists().await? {
            Self::print_custom_list(&custom_list);
        }
        Ok(())
    }

    async fn get(name: String) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let custom_list = rpc.get_custom_list(name).await?;
        Self::print_custom_list(&custom_list);
        Ok(())
    }

    async fn create_list(name: String) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.create_custom_list(name).await?;
        Ok(())
    }

    async fn add_location(name: String, location_args: LocationArgs) -> Result<()> {
        let location = Constraint::<GeographicLocationConstraint>::from(location_args);
        let update = CustomListLocationUpdate::Add { name, location };
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.update_custom_list_location(update).await?;
        Ok(())
    }

    async fn remove_location(name: String, location_args: LocationArgs) -> Result<()> {
        let location = Constraint::<GeographicLocationConstraint>::from(location_args);
        let update = CustomListLocationUpdate::Remove { name, location };
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.update_custom_list_location(update).await?;
        Ok(())
    }

    async fn delete_list(name: String) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.delete_custom_list(name).await?;
        Ok(())
    }

    async fn select_list(name: String) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let list_id = rpc.get_custom_list(name).await?.id;
        rpc.update_relay_settings(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            location: Some(Constraint::Only(LocationConstraint::CustomList { list_id, })),
            ..Default::default()
        })).await?;
        Ok(())
    }

    fn print_custom_list(custom_list: &mullvad_types::custom_list::CustomList) {
        println!("{}", custom_list.name);
        for location in &custom_list.locations {
            println!("\t{}", location);
        }
    }
}
