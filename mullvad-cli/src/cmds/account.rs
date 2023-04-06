use anyhow::{anyhow, Result};
use clap::Subcommand;
use itertools::Itertools;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{account::AccountToken, device::DeviceState};
use std::io::{self, Write};

const NOT_LOGGED_IN_MESSAGE: &str = "Not logged in on any account";
const REVOKED_MESSAGE: &str = "The current device has been revoked";

#[derive(Subcommand, Debug)]
pub enum Account {
    /// Create and log in on a new account
    Create,

    /// Log in on an account
    Login {
        /// The Mullvad account token to configure the client with
        account: Option<String>,
    },

    /// Log out of the current account
    Logout,

    /// Display information about the current account
    Get {
        /// Enable verbose output
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// List devices associated with an account
    ListDevices {
        /// Mullvad account number (current account if not specified)
        #[arg(long, short = 'a')]
        account: Option<String>,

        /// Enable verbose output
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Revoke a device associated with an account
    RevokeDevice {
        /// Name or UID of the device to revoke
        device: String,

        /// Mullvad account number (current account if not specified)
        #[arg(long, short = 'a')]
        account: Option<String>,
    },

    /// Redeem a voucher
    Redeem {
        /// Voucher code to submit
        voucher: String,
    },
}

impl Account {
    pub async fn handle(self) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        match self {
            Account::Create => Self::create(&mut rpc).await,
            Account::Login { account } => {
                Self::login(
                    &mut rpc,
                    account.unwrap_or_else(|| from_stdin("Enter an account number: ")),
                )
                .await
            }
            Account::Logout => Self::logout(&mut rpc).await,
            Account::Get { verbose } => Self::get(&mut rpc, verbose).await,
            Account::ListDevices { account, verbose } => {
                Self::list_devices(&mut rpc, account, verbose).await
            }
            Account::RevokeDevice { device, account } => {
                Self::revoke_device(&mut rpc, device, account).await
            }
            Account::Redeem { voucher } => Self::redeem_voucher(&mut rpc, voucher).await,
        }
    }

    async fn create(rpc: &mut MullvadProxyClient) -> Result<()> {
        rpc.create_new_account().await?;
        println!("New account created!");
        Self::get(rpc, false).await
    }

    async fn login(rpc: &mut MullvadProxyClient, token: AccountToken) -> Result<()> {
        rpc.login_account(token.clone()).await?;
        println!("Mullvad account \"{token}\" set");
        Ok(())
    }

    async fn logout(rpc: &mut MullvadProxyClient) -> Result<()> {
        rpc.logout_account().await?;
        println!("Removed device from Mullvad account");
        Ok(())
    }

    async fn get(rpc: &mut MullvadProxyClient, verbose: bool) -> Result<()> {
        let _ = rpc.update_device().await;

        let state = rpc.get_device().await?;

        match state {
            DeviceState::LoggedIn(device) => {
                println!("Mullvad account: {}", device.account_token);
                println!("Device name    : {}", device.device.pretty_name());
                if verbose {
                    println!("Device id      : {}", device.device.id);
                    println!("Device pubkey  : {}", device.device.pubkey);
                    println!("Device created : {}", device.device.created,);
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

    async fn list_devices(
        rpc: &mut MullvadProxyClient,
        account: Option<String>,
        verbose: bool,
    ) -> Result<()> {
        let token = account_else_current(rpc, account).await?;
        let mut device_list = rpc.list_devices(token).await?;

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

    async fn revoke_device(
        rpc: &mut MullvadProxyClient,
        device: String,
        account: Option<String>,
    ) -> Result<()> {
        let token = account_else_current(rpc, account).await?;

        let device_list = rpc.list_devices(token.clone()).await?;
        let device_id = device_list
            .into_iter()
            .find(|dev| {
                dev.name.eq_ignore_ascii_case(&device) || dev.id.eq_ignore_ascii_case(&device)
            })
            .map(|dev| dev.id)
            .ok_or(mullvad_management_interface::Error::DeviceNotFound)?;

        rpc.remove_device(token, device_id).await?;
        println!("Removed device");
        Ok(())
    }

    async fn redeem_voucher(rpc: &mut MullvadProxyClient, mut voucher: String) -> Result<()> {
        voucher.retain(|c| c.is_alphanumeric());

        let submission = rpc.submit_voucher(voucher).await?;
        println!(
            "Added {} to the account",
            format_duration(submission.time_added)
        );
        println!(
            "New expiry date: {}",
            submission.new_expiry.with_timezone(&chrono::Local),
        );
        Ok(())
    }
}

async fn account_else_current(
    rpc: &mut MullvadProxyClient,
    token: Option<String>,
) -> Result<String> {
    match token {
        Some(account) => Ok(account),
        None => {
            let state = rpc.get_device().await?;
            match state {
                DeviceState::LoggedIn(account) => Ok(account.account_token),
                _ => Err(anyhow!("Log in or specify an account")),
            }
        }
    }
}

fn from_stdin(prompt_str: &'static str) -> String {
    let mut val = String::new();
    io::stdout()
        .write_all(prompt_str.as_bytes())
        .expect("Failed to write to STDOUT");
    let _ = io::stdout().flush();
    io::stdin()
        .read_line(&mut val)
        .expect("Failed to read from STDIN");
    val.split_whitespace().join("")
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
