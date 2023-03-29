use crate::{Command, Error, MullvadProxyClient, Result};
use itertools::Itertools;
use mullvad_types::{account::AccountToken, device::DeviceState};
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
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.create_new_account().await.map_err(map_device_error)?;
        println!("New account created!");
        self.get(false).await
    }

    async fn login(&self, token: AccountToken) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.login_account(token.clone())
            .await
            .map_err(map_device_error)?;
        println!("Mullvad account \"{token}\" set");
        Ok(())
    }

    async fn logout(&self) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.logout_account().await?;
        println!("Removed device from Mullvad account");
        Ok(())
    }

    async fn get(&self, verbose: bool) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let _ = rpc.update_device().await;

        let state = rpc.get_device().await.map_err(map_device_error)?;

        match state {
            DeviceState::LoggedIn(device) => {
                println!("Mullvad account: {}", device.account_token);
                println!("Device name    : {}", device.device.pretty_name());
                if verbose {
                    println!("Device id      : {}", device.device.id);
                    println!("Device pubkey  : {}", device.device.pubkey);
                    println!(
                        "Device created : {}",
                        device.device.created.with_timezone(&chrono::Local),
                    );
                    for port in device.device.ports {
                        println!("Device port    : {port}");
                    }
                }
                let expiry = rpc.get_account_data(device.account_token).await?;
                println!(
                    "Expires at     : {}",
                    expiry.expiry.with_timezone(&chrono::Local),
                );
            }
            DeviceState::LoggedOut => {
                println!("{NOT_LOGGED_IN_MESSAGE}");
            }
            DeviceState::Revoked => {
                println!("{REVOKED_MESSAGE}");
            }
        }

        Ok(())
    }

    async fn list_devices(&self, matches: &clap::ArgMatches) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let token = self.parse_account_else_current(&mut rpc, matches).await?;
        let mut device_list = rpc.list_devices(token).await.map_err(map_device_error)?;

        let verbose = matches.is_present("verbose");

        println!("Devices on the account:");
        device_list.sort_unstable_by_key(|dev| dev.created.timestamp());
        for device in device_list {
            if verbose {
                println!();
                println!("Name      : {}", device.pretty_name());
                println!("Id        : {}", device.id);
                println!("Public key: {}", device.pubkey);
                println!(
                    "Created   : {}",
                    device.created.with_timezone(&chrono::Local)
                );
                for port in device.ports {
                    println!("Port      : {port}");
                }
            } else {
                println!("{}", device.pretty_name());
            }
        }

        Ok(())
    }

    async fn revoke_device(&self, matches: &clap::ArgMatches) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;

        let token = self.parse_account_else_current(&mut rpc, matches).await?;
        let device_to_revoke = parse_device_name(matches);

        let device_list = rpc
            .list_devices(token.clone())
            .await
            .map_err(map_device_error)?;
        let device_id = device_list
            .into_iter()
            .find(|dev| {
                dev.name.eq_ignore_ascii_case(&device_to_revoke)
                    || dev.id.eq_ignore_ascii_case(&device_to_revoke)
            })
            .map(|dev| dev.id)
            .ok_or(Error::Other(DEVICE_NOT_FOUND_ERROR))?;

        rpc.remove_device(token, device_id)
            .await
            .map_err(map_device_error)?;
        println!("Removed device");
        Ok(())
    }

    async fn parse_account_else_current(
        &self,
        rpc: &mut MullvadProxyClient,
        matches: &clap::ArgMatches,
    ) -> Result<String> {
        match matches.value_of("account").map(str::to_string) {
            Some(token) => Ok(token),
            None => {
                let state = rpc.get_device().await?;
                match state {
                    DeviceState::LoggedIn(account) => Ok(account.account_token),
                    _ => Err(Error::Other("Log in or specify an account")),
                }
            }
        }
    }

    async fn redeem_voucher(&self, mut voucher: String) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        voucher.retain(|c| c.is_alphanumeric());

        let submission = rpc.submit_voucher(voucher).await?;
        println!(
            "Added {} to the account",
            Self::format_duration(submission.time_added)
        );
        println!(
            "New expiry date: {}",
            submission.new_expiry.with_timezone(&chrono::Local),
        );
        Ok(())
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
}

fn map_device_error(error: mullvad_management_interface::Error) -> Error {
    match &error {
        mullvad_management_interface::Error::TooManyDevices => Error::Other(TOO_MANY_DEVICES_ERROR),
        mullvad_management_interface::Error::InvalidAccount => Error::Other(INVALID_ACCOUNT_ERROR),
        mullvad_management_interface::Error::AlreadyLoggedIn => {
            Error::Other(ALREADY_LOGGED_IN_ERROR)
        }
        mullvad_management_interface::Error::DeviceNotFound => Error::Other(DEVICE_NOT_FOUND_ERROR),
        _other => Error::ManagementInterfaceError(error),
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
