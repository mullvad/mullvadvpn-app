use crate::{new_rpc_client, Command, Error, Result};
use itertools::Itertools;
use mullvad_management_interface::{types::Timestamp, Code};
use mullvad_types::account::AccountToken;
use std::io::{self, Write};

pub struct Account;

#[mullvad_management_interface::async_trait]
impl Command for Account {
    fn name(&self) -> &'static str {
        "account"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name())
            .about("Control and display information about your Mullvad account")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(clap::App::new("create").about("Create and log in to a new account"))
            .subcommand(
                clap::App::new("login").about("Log in to an account").arg(
                    clap::Arg::new("token")
                        .help("The Mullvad account token to configure the client with")
                        .required(false),
                ),
            )
            .subcommand(clap::App::new("logout").about("Log out of the current account"))
            .subcommand(
                clap::App::new("get").about("Display information about the current account"),
            )
            .subcommand(
                clap::App::new("redeem").about("Redeems a voucher").arg(
                    clap::Arg::new("voucher")
                        .help("The Mullvad voucher code to be submitted")
                        .required(true),
                ),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(_matches) = matches.subcommand_matches("create") {
            self.create().await
        } else if let Some(set_matches) = matches.subcommand_matches("login") {
            let mut token = match set_matches.value_of("token") {
                Some(token) => token.to_string(),
                None => {
                    let mut token = String::new();
                    io::stdout()
                        .write_all(b"Enter account token: ")
                        .expect("Failed to write to STDOUT");
                    let _ = io::stdout().flush();
                    io::stdin()
                        .read_line(&mut token)
                        .expect("Failed to read from STDIN");
                    token
                }
            };
            token = token.split_whitespace().join("").to_string();
            self.login(token).await
        } else if let Some(_matches) = matches.subcommand_matches("logout") {
            self.logout().await
        } else if let Some(_matches) = matches.subcommand_matches("get") {
            self.get().await
        } else if let Some(matches) = matches.subcommand_matches("redeem") {
            let voucher = matches.value_of_t_or_exit("voucher");
            self.redeem_voucher(voucher).await
        } else {
            unreachable!("No account command given");
        }
    }
}

impl Account {
    async fn create(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.create_new_account(()).await?;
        println!("New account created!");
        self.get().await
    }

    async fn login(&self, token: AccountToken) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.login_account(token.clone()).await?;
        println!("Mullvad account \"{}\" set", token);
        Ok(())
    }

    async fn logout(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.logout_account(()).await?;
        println!("Removed device from Mullvad account");
        Ok(())
    }

    async fn get(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let device = rpc.get_device(()).await?.into_inner();
        if !device.account_token.is_empty() {
            println!("Mullvad account: {}", device.account_token);
            println!("Device name    : {}", device.device.unwrap().name);
            let expiry = rpc
                .get_account_data(device.account_token)
                .await
                .map_err(|error| Error::RpcFailedExt("Failed to fetch account data", error))?
                .into_inner();
            println!(
                "Expires at     : {}",
                Self::format_expiry(&expiry.expiry.unwrap())
            );
        } else {
            println!("No account configured");
        }
        Ok(())
    }

    async fn redeem_voucher(&self, mut voucher: String) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        voucher.retain(|c| c.is_alphanumeric());

        match rpc.submit_voucher(voucher).await {
            Ok(submission) => {
                let submission = submission.into_inner();
                println!(
                    "Added {} to the account",
                    Self::format_duration(submission.seconds_added)
                );
                println!(
                    "New expiry date: {}",
                    Self::format_expiry(&submission.new_expiry.unwrap())
                );
                Ok(())
            }
            Err(err) => {
                match err.code() {
                    Code::NotFound | Code::ResourceExhausted => {
                        eprintln!("Failed to submit voucher: {}", err.message());
                    }
                    _ => return Err(Error::RpcFailed(err)),
                }
                std::process::exit(1);
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

    fn format_expiry(expiry: &Timestamp) -> String {
        let ndt = chrono::NaiveDateTime::from_timestamp(expiry.seconds, expiry.nanos as u32);
        let utc = chrono::DateTime::<chrono::Utc>::from_utc(ndt, chrono::Utc);
        utc.with_timezone(&chrono::Local).to_string()
    }
}
