use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::{MullvadProxyClient, client::RelaySelectorClient};

use super::BooleanOption;

#[derive(Subcommand, Debug)]
pub enum LockdownMode {
    /// Display the current lockdown mode setting
    Get,
    /// Change the lockdown mode setting
    Set { policy: BooleanOption },
    /// TODO: Remove me
    /// Relay selector through gRPC!!!
    Test,
}

impl LockdownMode {
    pub async fn handle(self) -> Result<()> {
        match self {
            LockdownMode::Get => Self::get().await,
            LockdownMode::Set { policy } => Self::set(policy).await,
            LockdownMode::Test => Self::test().await,
        }
    }

    async fn set(policy: BooleanOption) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_lockdown_mode(*policy).await?;
        println!("Changed lockdown mode setting");
        Ok(())
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let state = BooleanOption::from(rpc.get_settings().await?.lockdown_mode);
        println!("Block traffic when the VPN is disconnected: {state}");
        Ok(())
    }

    // TODO: Remove
    async fn test() -> Result<()> {
        use mullvad_management_interface::types::LocationConstraint;
        //use mullvad_management_interface::types::location_constraints::*;
        use mullvad_management_interface::types::location_constraint::Type;
        use mullvad_management_interface::types::relay_selector::*;
        use mullvad_types::relay_constraints::GeographicLocationConstraint;

        let mut relay_selector = RelaySelectorClient::new().await?;
        println!("Connected to relay selector gRPC client!");
        let predicate = {
            let entry_constraints = EntryConstraints {
                general_constraints: Some(ExitConstraints {
                    location: None,
                    // location: Some(LocationConstraint {
                    //     r#type: Some(Type::Location(
                    //         GeographicLocationConstraint::country("se").into(),
                    //     )),
                    // }),
                    // TODO: Make sure the empty vec maps to Constraint::Any.
                    ..Default::default()
                }),
                // obfuscation_settings: None,
                // daita_settings: None,
                ip_version: 4,
                ..Default::default()
            };
            let context = predicate::Context::Singlehop(entry_constraints);
            Predicate {
                context: Some(context),
            }
        };
        println!("Running query {predicate:#?}");
        let relays = relay_selector.partition_relays(predicate).await?;
        println!("{:#?}", relays.matches);
        Ok(())
    }
}
