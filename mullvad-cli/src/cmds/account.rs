use crate::{new_rpc_client, Command, Error, Result};
use itertools::Itertools;
use mullvad_management_interface::{
    types::{self, Timestamp},
    Code, ManagementServiceClient, Status,
};
use mullvad_types::{account::AccountToken, device::Device};
use std::io::{self, Write};

const NOT_LOGGED_IN_MESSAGE: &str = "Not logged in on any account";
const REVOKED_MESSAGE: &str = "The current device has been revoked";
const DEVICE_NOT_FOUND_ERROR: &str = "There is no such device";
const INVALID_ACCOUNT_ERROR: &str = "The account does not exist";
const TOO_MANY_DEVICES_ERROR: &str =
    "There are too many devices on this account. Revoke one to log in";
const ALREADY_LOGGED_IN_ERROR: &str =
    "You are already logged in. Please log out before creating a new account";

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
                    clap::Arg::new("account")
                        .help("The Mullvad account token to configure the client with")
                        .required(false),
                ),
            )
            .subcommand(clap::App::new("logout").about("Log out of the current account"))
            .subcommand(
                clap::App::new("get")
                    .about("Display information about the current account")
                    .arg(
                        clap::Arg::new("verbose")
                            .long("verbose")
                            .short('v')
                            .help("Enables verbose output"),
                    ),
            )
            .subcommand(
                clap::App::new("list-devices")
                    .about("List devices associated with an account")
                    .arg(
                        clap::Arg::new("account")
                            .help("Mullvad account number")
                            .long("account")
                            .takes_value(true),
                    )
                    .arg(
                        clap::Arg::new("verbose")
                            .long("verbose")
                            .short('v')
                            .help("Enables verbose output"),
                    ),
            )
            .subcommand(
                clap::App::new("revoke-device")
                    .about("Revoke a device associated with an account")
                    .arg(
                        clap::Arg::new("account")
                            .help("Mullvad account number")
                            .long("account")
                            .takes_value(true),
                    )
                    .arg(
                        clap::Arg::new("device")
                            .help("Name or ID of the device to revoke")
                            .required(true),
                    ),
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
            self.login(parse_token_else_stdin(set_matches)).await
        } else if let Some(_matches) = matches.subcommand_matches("logout") {
            self.logout().await
        } else if let Some(set_matches) = matches.subcommand_matches("get") {
            let verbose = set_matches.is_present("verbose");
            self.get(verbose).await
        } else if let Some(set_matches) = matches.subcommand_matches("list-devices") {
            self.list_devices(set_matches).await
        } else if let Some(set_matches) = matches.subcommand_matches("revoke-device") {
            self.revoke_device(set_matches).await
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
        rpc.create_new_account(()).await.map_err(map_device_error)?;
        println!("New account created!");
        self.get(false).await
    }

    async fn login(&self, token: AccountToken) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.login_account(token.clone())
            .await
            .map_err(map_device_error)?;
        println!("Mullvad account \"{}\" set", token);
        Ok(())
    }

    async fn logout(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.logout_account(()).await?;
        println!("Removed device from Mullvad account");
        Ok(())
    }

    async fn get(&self, verbose: bool) -> Result<()> {
        let mut rpc = new_rpc_client().await?;

        let _ = rpc.update_device(()).await;

        let state = rpc
            .get_device(())
            .await
            .map_err(map_device_error)?
            .into_inner();

        use types::device_state::State;

        match State::from_i32(state.state).unwrap() {
            State::LoggedIn => {
                let device = state.device.expect("Device must be provided if logged in");
                println!("Mullvad account: {}", device.account_token);
                let inner_device = Device::try_from(device.device.unwrap()).unwrap();
                println!("Device name    : {}", inner_device.pretty_name());
                if verbose {
                    println!("Device id      : {}", inner_device.id);
                    println!("Device pubkey  : {}", inner_device.pubkey);
                    for port in inner_device.ports {
                        println!("Device port    : {}", port);
                    }
                }
                let expiry = rpc
                    .get_account_data(device.account_token)
                    .await
                    .map_err(|error| Error::RpcFailedExt("Failed to fetch account data", error))?
                    .into_inner();
                println!(
                    "Expires at     : {}",
                    Self::format_expiry(&expiry.expiry.unwrap())
                );
            }
            State::LoggedOut => {
                println!("{}", NOT_LOGGED_IN_MESSAGE);
            }
            State::Revoked => {
                println!("{}", REVOKED_MESSAGE);
            }
        }

        Ok(())
    }

    async fn list_devices(&self, matches: &clap::ArgMatches) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let token = self.parse_account_else_current(&mut rpc, matches).await?;
        let device_list = rpc
            .list_devices(token)
            .await
            .map_err(map_device_error)?
            .into_inner();

        let verbose = matches.is_present("verbose");

        println!("Devices on the account:");
        for device in device_list.devices {
            let device = Device::try_from(device.clone()).unwrap();
            if verbose {
                println!();
                println!("Name      : {}", device.pretty_name());
                println!("Id        : {}", device.id);
                println!("Public key: {}", device.pubkey);
                for port in device.ports {
                    println!("Port      : {}", port);
                }
            } else {
                println!("{}", device.pretty_name());
            }
        }

        Ok(())
    }

    async fn revoke_device(&self, matches: &clap::ArgMatches) -> Result<()> {
        let mut rpc = new_rpc_client().await?;

        let token = self.parse_account_else_current(&mut rpc, matches).await?;
        let device_to_revoke = parse_device_name(matches);

        let device_list = rpc
            .list_devices(token.clone())
            .await
            .map_err(map_device_error)?
            .into_inner();
        let device_id = device_list
            .devices
            .into_iter()
            .find(|dev| {
                dev.name.eq_ignore_ascii_case(&device_to_revoke)
                    || dev.id.eq_ignore_ascii_case(&device_to_revoke)
            })
            .map(|dev| dev.id)
            .ok_or_else(|| Error::Other(DEVICE_NOT_FOUND_ERROR))?;

        rpc.remove_device(types::DeviceRemoval {
            account_token: token,
            device_id,
        })
        .await
        .map_err(map_device_error)?;
        println!("Removed device");
        Ok(())
    }

    async fn parse_account_else_current(
        &self,
        rpc: &mut ManagementServiceClient,
        matches: &clap::ArgMatches,
    ) -> Result<String> {
        match matches.value_of("account").map(str::to_string) {
            Some(token) => Ok(token),
            None => {
                let state = rpc
                    .get_device(())
                    .await
                    .map_err(|error| Error::RpcFailedExt("Failed to obtain device", error))?
                    .into_inner();
                if state.state != types::device_state::State::LoggedIn as i32 {
                    return Err(Error::Other("Log in or specify an account"));
                }
                Ok(state.device.unwrap().account_token)
            }
        }
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

fn map_device_error(error: Status) -> Error {
    match error.code() {
        Code::ResourceExhausted => Error::Other(TOO_MANY_DEVICES_ERROR),
        Code::Unauthenticated => Error::Other(INVALID_ACCOUNT_ERROR),
        Code::AlreadyExists => Error::Other(ALREADY_LOGGED_IN_ERROR),
        Code::NotFound => Error::Other(DEVICE_NOT_FOUND_ERROR),
        _other => Error::RpcFailed(error),
    }
}

fn parse_token_else_stdin(matches: &clap::ArgMatches) -> String {
    parse_from_match_else_stdin("Enter account number: ", "account", matches)
        .split_whitespace()
        .join("")
}

fn parse_device_name(matches: &clap::ArgMatches) -> String {
    parse_from_match_else_stdin("Enter device name: ", "device", matches)
        .trim()
        .to_string()
}

fn parse_from_match_else_stdin(
    prompt_str: &'static str,
    key: &'static str,
    matches: &clap::ArgMatches,
) -> String {
    match matches.value_of(key) {
        Some(device) => device.to_string(),
        None => {
            let mut val = String::new();
            io::stdout()
                .write_all(prompt_str.as_bytes())
                .expect("Failed to write to STDOUT");
            let _ = io::stdout().flush();
            io::stdin()
                .read_line(&mut val)
                .expect("Failed to read from STDIN");
            val
        }
    }
}
