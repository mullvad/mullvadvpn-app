use crate::{new_rpc_client, Command, Result};
use clap::value_t_or_exit;
use mullvad_types::account::AccountToken;

pub struct Account;

impl Command for Account {
    fn name(&self) -> &'static str {
        "account"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Control and display information about your Mullvad account")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("set")
                    .about("Change account")
                    .arg(
                        clap::Arg::with_name("token")
                            .help("The Mullvad account token to configure the client with")
                            .required(true),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("get")
                    .about("Display information about the currently configured account"),
            )
            .subcommand(
                clap::SubCommand::with_name("unset")
                    .about("Removes the account number from the settings"),
            )
    }

    fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            let token = value_t_or_exit!(set_matches.value_of("token"), String);
            self.set(Some(token))
        } else if let Some(_matches) = matches.subcommand_matches("unset") {
            self.set(None)
        } else if let Some(_matches) = matches.subcommand_matches("get") {
            self.get()
        } else {
            unreachable!("No account command given");
        }
    }
}

impl Account {
    fn set(&self, token: Option<AccountToken>) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        rpc.set_account(token.clone())?;
        if let Some(token) = token {
            println!("Mullvad account \"{}\" set", token);
        } else {
            println!("Mullvad account removed");
        }
        Ok(())
    }

    fn get(&self) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        let settings = rpc.get_settings()?;
        if let Some(account_token) = settings.get_account_token() {
            println!("Mullvad account: {}", account_token);
            let expiry = rpc.get_account_data(account_token)?;
            println!("Expires at     : {}", expiry.expiry);
        } else {
            println!("No account configured");
        }
        Ok(())
    }
}
