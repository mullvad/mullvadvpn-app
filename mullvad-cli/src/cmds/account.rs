use crate::{new_rpc_client, Command, Result};
use clap::value_t_or_exit;
use mullvad_types::account::{AccountToken, VoucherError};

pub struct Account;

#[async_trait::async_trait]
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
            .subcommand(
                clap::SubCommand::with_name("clear-history")
                    .about("Clear account history, along with removing all associated keys"),
            )
            .subcommand(
                clap::SubCommand::with_name("create")
                    .about("Creates a new account and sets it as the active one"),
            )
            .subcommand(
                clap::SubCommand::with_name("redeem")
                    .about("Redeems a voucher")
                    .arg(
                        clap::Arg::with_name("voucher")
                            .help("The Mullvad voucher code to be submitted")
                            .required(true),
                    ),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            let token = value_t_or_exit!(set_matches.value_of("token"), String);
            self.set(Some(token))
        } else if let Some(_matches) = matches.subcommand_matches("get") {
            self.get()
        } else if let Some(_matches) = matches.subcommand_matches("unset") {
            self.set(None)
        } else if let Some(_matches) = matches.subcommand_matches("clear-history") {
            self.clear_history()
        } else if let Some(_matches) = matches.subcommand_matches("create") {
            self.create()
        } else if let Some(matches) = matches.subcommand_matches("redeem") {
            let voucher = value_t_or_exit!(matches.value_of("voucher"), String);
            self.redeem_voucher(voucher)
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

    fn create(&self) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        rpc.create_new_account()?;
        println!("New account created!");
        self.get()
    }

    fn redeem_voucher(&self, mut voucher: String) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        voucher.retain(|c| c.is_alphanumeric());

        match rpc.submit_voucher(voucher) {
            Ok(submission) => {
                println!(
                    "Added {} to the account",
                    Self::format_duration(submission.time_added)
                );
                println!("New expiry date: {}", submission.new_expiry);
                Ok(())
            }
            Err(err) => {
                eprintln!(
                    "Failed to submit voucher.\n{}",
                    VoucherError::from_rpc_error_code(Self::get_redeem_rpc_error_code(&err))
                );
                Err(err.into())
            }
        }
    }

    fn format_duration(seconds: u64) -> String {
        let dur = chrono::Duration::seconds(seconds as i64);
        if dur.num_days() > 0 {
            format!("{} days", dur.num_days())
        } else if dur.num_hours() > 0 {
            format!("{} hours", dur.num_hours())
        } else if dur.num_minutes() > 0 {
            format!("{} minutes", dur.num_minutes())
        } else {
            format!("{} seconds", dur.num_seconds())
        }
    }

    fn get_redeem_rpc_error_code(error: &mullvad_ipc_client::Error) -> i64 {
        match error.kind() {
            mullvad_ipc_client::ErrorKind::JsonRpcError(ref rpc_error) => rpc_error.code.code(),
            _ => 0,
        }
    }

    fn clear_history(&self) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        rpc.clear_account_history()?;
        println!("Removed account history and all associated keys");
        Ok(())
    }
}
