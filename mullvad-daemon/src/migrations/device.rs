//! Generates a `device.json` from a WireGuard key and account token by matching them against
//! devices returned by the API and sending the `DeviceMigrationEvent` event to the daemon.
//! The account token and private key may be lost if it fails, but this should not be not
//! critical since the account history also contains the token.
//!
//! This module is allowed to import a number of types, unlike other migration modules, as it
//! does not modify any files directly and may safely fail.

use super::{v5::MigrationData, MigrationComplete};
use crate::{
    device::{self, DeviceService, PrivateAccountAndDevice, PrivateDevice},
    DaemonEventSender, InternalDaemonEvent,
};
use mullvad_types::{account::AccountToken, wireguard::WireguardData};
use std::time::Duration;
use talpid_core::mpsc::Sender;
use talpid_types::ErrorExt;
use tokio::time::timeout;

const TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) fn generate_device(
    migration_data: MigrationData,
    mut migration_complete: MigrationComplete,
    rest_handle: mullvad_api::rest::MullvadRestHandle,
    daemon_tx: DaemonEventSender,
) {
    tokio::spawn(async move {
        let wg_data: Option<WireguardData> = migration_data.wg_data.and_then(|data| {
            serde_json::from_value(data)
                .map(Some)
                .unwrap_or_else(|error| {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to parse WireGuard data")
                    );
                    None
                })
        });

        let api_handle = rest_handle.availability.clone();
        let service = DeviceService::new(rest_handle, api_handle);
        let result = match (migration_data.token, wg_data) {
            (token, Some(wg_data)) => {
                log::info!("Creating a new device cache from previous settings");
                cache_from_wireguard_key(service, token, wg_data).await
            }
            (token, None) => {
                log::info!("Generating a new device for the account");
                cache_from_account(service, token).await
            }
        };
        let _ = daemon_tx.send(InternalDaemonEvent::DeviceMigrationEvent(result));
        migration_complete.set_complete();
    });
}

async fn cache_from_wireguard_key(
    service: DeviceService,
    account_token: AccountToken,
    wg_data: WireguardData,
) -> Result<PrivateAccountAndDevice, device::Error> {
    let devices = timeout(
        TIMEOUT,
        service.list_devices_with_backoff(account_token.clone()),
    )
    .await
    .map_err(|_error| {
        log::error!("Failed to enumerate devices for account: timed out");
        device::Error::Cancelled
    })?
    .map_err(|error| {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to enumerate devices for account")
        );
        error
    })?;

    for device in devices.into_iter() {
        if device.pubkey == wg_data.private_key.public_key() {
            return Ok(PrivateAccountAndDevice {
                account_token,
                device: PrivateDevice::try_from_device(device, wg_data)?,
            });
        }
    }
    log::info!("The existing WireGuard key is not valid");
    Err(device::Error::InvalidDevice)
}

async fn cache_from_account(
    service: DeviceService,
    account_token: AccountToken,
) -> Result<PrivateAccountAndDevice, device::Error> {
    timeout(
        TIMEOUT,
        service.generate_for_account_with_backoff(account_token),
    )
    .await
    .map_err(|_error| {
        log::error!("Failed to generate new device for account: timed out");
        device::Error::Cancelled
    })?
    .map_err(|error| {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to generate new device for account")
        );
        error
    })
}
