use super::relay_constraints::LocationArgs;
use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    custom_list::CustomListLocationUpdate,
    relay_constraints::{Constraint, GeographicLocationConstraint},
};

#[derive(Subcommand, Debug)]
pub enum CustomList {
    /// List all custom lists or retrieve a custom list by its name
    List {
        /// A custom list. This argument is optional
        name: Option<String>,
    },

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

    /// Rename a custom list to a new name
    Rename { name: String, new_name: String },
}

impl CustomList {
    pub async fn handle(self) -> Result<()> {
        match self {
            CustomList::List { name: None } => Self::list().await,
            CustomList::List { name: Some(name) } => Self::get(name).await,
            CustomList::Create { name } => Self::create_list(name).await,
            CustomList::Add { name, location } => Self::add_location(name, location).await,
            CustomList::Remove { name, location } => Self::remove_location(name, location).await,
            CustomList::Delete { name } => Self::delete_list(name).await,
            CustomList::Rename { name, new_name } => Self::rename_list(name, new_name).await,
        }
    }

    /// Print all custom lists.
    async fn list() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        for custom_list in rpc.list_custom_lists().await? {
            Self::print_custom_list(&custom_list);
        }
        Ok(())
    }

    /// Print a specific custom list (if it exists).
    /// If the list does not exist, print an error.
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

    async fn rename_list(name: String, new_name: String) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.rename_custom_list(name, new_name).await?;
        Ok(())
    }

    fn print_custom_list(custom_list: &mullvad_types::custom_list::CustomList) {
        println!("{}", custom_list.name);
        for location in &custom_list.locations {
            println!("\t{}", location);
        }
    }
}
