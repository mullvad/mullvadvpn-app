use std::{future::Future, time::Duration};

use chrono::{DateTime, Utc};
use futures::future::{abortable, AbortHandle};
#[cfg(target_os = "android")]
use mullvad_types::account::{PlayPurchase, PlayPurchasePaymentToken};
use mullvad_types::{
    account::{AccountToken, VoucherSubmission},
    device::{Device, DeviceId},
    wireguard::WireguardData,
};
use talpid_types::net::wireguard::PrivateKey;

use super::{Error, PrivateAccountAndDevice, PrivateDevice};
use mullvad_api::{
    availability::ApiAvailabilityHandle,
    rest::{self, MullvadRestHandle},
    AccountsProxy, DevicesProxy,
};
use talpid_core::future_retry::{retry_future, ConstantInterval, ExponentialBackoff, Jittered};
/// Retry strategy used for user-initiated actions that require immediate feedback
const RETRY_ACTION_STRATEGY: ConstantInterval = ConstantInterval::new(Duration::ZERO, Some(3));
/// Retry strategy used for background tasks
const RETRY_BACKOFF_STRATEGY: Jittered<ExponentialBackoff> = Jittered::jitter(
    ExponentialBackoff::new(Duration::from_secs(4), 5)
        .max_delay(Some(Duration::from_secs(24 * 60 * 60))),
);

#[derive(Clone)]
pub struct DeviceService {
    api_availability: ApiAvailabilityHandle,
    proxy: DevicesProxy,
}

impl DeviceService {
    pub fn new(handle: rest::MullvadRestHandle, api_availability: ApiAvailabilityHandle) -> Self {
        Self {
            proxy: DevicesProxy::new(handle),
            api_availability,
        }
    }

    /// Generate a new device for a given token
    pub fn generate_for_account(
        &self,
        account_token: AccountToken,
    ) -> impl Future<Output = Result<PrivateAccountAndDevice, Error>> + Send {
        let private_key = PrivateKey::new_from_random();
        let pubkey = private_key.public_key();

        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        let token_copy = account_token.clone();
        async move {
            let (device, addresses) = retry_future(
                move || proxy.create(token_copy.clone(), pubkey.clone()),
                move |result| should_retry(result, &api_handle),
                RETRY_ACTION_STRATEGY,
            )
            .await
            .map_err(map_rest_error)?;

            Ok(PrivateAccountAndDevice {
                account_token,
                device: PrivateDevice::try_from_device(
                    device,
                    WireguardData {
                        private_key,
                        addresses,
                        created: Utc::now(),
                    },
                )?,
            })
        }
    }

    pub async fn generate_for_account_with_backoff(
        &self,
        account_token: AccountToken,
    ) -> Result<PrivateAccountAndDevice, Error> {
        let private_key = PrivateKey::new_from_random();
        let pubkey = private_key.public_key();

        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        let token_copy = account_token.clone();
        let (device, addresses) = retry_future(
            move || api_handle.when_online(proxy.create(token_copy.clone(), pubkey.clone())),
            should_retry_backoff,
            RETRY_BACKOFF_STRATEGY,
        )
        .await
        .map_err(map_rest_error)?;

        Ok(PrivateAccountAndDevice {
            account_token,
            device: PrivateDevice::try_from_device(
                device,
                WireguardData {
                    private_key,
                    addresses,
                    created: Utc::now(),
                },
            )?,
        })
    }

    pub async fn remove_device(
        &self,
        account_token: AccountToken,
        device_id: DeviceId,
    ) -> Result<Vec<Device>, Error> {
        self.remove_device_inner(account_token.clone(), device_id)
            .await?;
        self.list_devices(account_token).await
    }

    async fn remove_device_inner(
        &self,
        token: AccountToken,
        device: DeviceId,
    ) -> Result<(), Error> {
        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        retry_future(
            move || proxy.remove(token.clone(), device.clone()),
            move |result| should_retry(result, &api_handle),
            RETRY_ACTION_STRATEGY,
        )
        .await
        .map_err(map_rest_error)?;
        Ok(())
    }

    pub async fn remove_device_with_backoff(
        &self,
        token: AccountToken,
        device: DeviceId,
    ) -> Result<(), Error> {
        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();

        retry_future(
            // NOTE: Not honoring "paused" state, because the account may have no time on it.
            move || api_handle.when_online(proxy.remove(token.clone(), device.clone())),
            should_retry_backoff,
            // Not setting a maximum interval
            RETRY_BACKOFF_STRATEGY.clone().max_delay(None),
        )
        .await
        .map_err(map_rest_error)?;

        Ok(())
    }

    pub async fn rotate_key(
        &self,
        token: AccountToken,
        device: DeviceId,
    ) -> Result<WireguardData, Error> {
        let private_key = PrivateKey::new_from_random();

        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        let pubkey = private_key.public_key();
        let addresses = retry_future(
            move || proxy.replace_wg_key(token.clone(), device.clone(), pubkey.clone()),
            move |result| should_retry(result, &api_handle),
            RETRY_ACTION_STRATEGY,
        )
        .await
        .map_err(map_rest_error)?;

        Ok(WireguardData {
            private_key,
            addresses,
            created: Utc::now(),
        })
    }

    pub async fn rotate_key_with_backoff(
        &self,
        token: AccountToken,
        device: DeviceId,
    ) -> Result<WireguardData, Error> {
        let private_key = PrivateKey::new_from_random();

        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        let pubkey = private_key.public_key();

        let rotate_retry_strategy = std::iter::repeat(Duration::from_secs(24 * 60 * 60));

        let addresses = retry_future(
            move || {
                api_handle.when_bg_resumes(proxy.replace_wg_key(
                    token.clone(),
                    device.clone(),
                    pubkey.clone(),
                ))
            },
            should_retry_backoff,
            rotate_retry_strategy,
        )
        .await
        .map_err(map_rest_error)?;

        Ok(WireguardData {
            private_key,
            addresses,
            created: Utc::now(),
        })
    }

    pub async fn list_devices(&self, token: AccountToken) -> Result<Vec<Device>, Error> {
        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        retry_future(
            move || proxy.list(token.clone()),
            move |result| should_retry(result, &api_handle),
            RETRY_ACTION_STRATEGY,
        )
        .await
        .map_err(map_rest_error)
    }

    pub async fn list_devices_with_backoff(
        &self,
        token: AccountToken,
    ) -> Result<Vec<Device>, Error> {
        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();

        retry_future(
            move || api_handle.when_online(proxy.list(token.clone())),
            should_retry_backoff,
            RETRY_BACKOFF_STRATEGY,
        )
        .await
        .map_err(map_rest_error)
    }

    pub async fn get(&self, token: AccountToken, device: DeviceId) -> Result<Device, Error> {
        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        retry_future(
            move || proxy.get(token.clone(), device.clone()),
            move |result| should_retry(result, &api_handle),
            RETRY_ACTION_STRATEGY,
        )
        .await
        .map_err(map_rest_error)
    }
}

#[derive(Clone)]
pub struct AccountService {
    api_availability: ApiAvailabilityHandle,
    initial_check_abort_handle: AbortHandle,
    proxy: AccountsProxy,
}

impl AccountService {
    pub fn create_account(&self) -> impl Future<Output = Result<AccountToken, rest::Error>> {
        let mut proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        retry_future(
            move || proxy.create_account(),
            move |result| should_retry(result, &api_handle),
            RETRY_ACTION_STRATEGY,
        )
    }

    pub fn get_www_auth_token(
        &self,
        account: AccountToken,
    ) -> impl Future<Output = Result<String, rest::Error>> {
        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        retry_future(
            move || proxy.get_www_auth_token(account.clone()),
            move |result| should_retry(result, &api_handle),
            RETRY_ACTION_STRATEGY,
        )
    }

    pub async fn check_expiry(&self, token: AccountToken) -> Result<DateTime<Utc>, rest::Error> {
        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        let result = retry_future(
            move || proxy.get_expiry(token.clone()),
            move |result| should_retry(result, &api_handle),
            RETRY_ACTION_STRATEGY,
        )
        .await;
        if handle_expiry_result_inner(&result, &self.api_availability) {
            self.initial_check_abort_handle.abort();
        }
        result
    }

    pub async fn check_expiry_2(&self, token: AccountToken) -> Result<DateTime<Utc>, Error> {
        self.check_expiry(token).await.map_err(map_rest_error)
    }

    pub async fn submit_voucher(
        &self,
        account_token: AccountToken,
        voucher: String,
    ) -> Result<VoucherSubmission, Error> {
        let mut proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        let result = retry_future(
            move || proxy.submit_voucher(account_token.clone(), voucher.clone()),
            move |result| should_retry(result, &api_handle),
            RETRY_ACTION_STRATEGY,
        )
        .await;
        if result.is_ok() {
            self.initial_check_abort_handle.abort();
            self.api_availability.resume_background();
        }
        result.map_err(map_rest_error)
    }

    #[cfg(target_os = "android")]
    pub async fn init_play_purchase(
        &self,
        account_token: AccountToken,
    ) -> Result<PlayPurchasePaymentToken, Error> {
        let mut proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        let result = retry_future(
            move || proxy.init_play_purchase(account_token.clone()),
            move |result| should_retry(result, &api_handle),
            RETRY_ACTION_STRATEGY,
        )
        .await;
        if result.is_ok() {
            self.initial_check_abort_handle.abort();
            self.api_availability.resume_background();
        }
        result.map_err(map_rest_error)
    }

    #[cfg(target_os = "android")]
    pub async fn verify_play_purchase(
        &self,
        account_token: AccountToken,
        play_purchase: PlayPurchase,
    ) -> Result<(), Error> {
        let mut proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        let result = retry_future(
            move || proxy.verify_play_purchase(account_token.clone(), play_purchase.clone()),
            move |result| should_retry(result, &api_handle),
            RETRY_ACTION_STRATEGY,
        )
        .await;
        if result.is_ok() {
            self.initial_check_abort_handle.abort();
            self.api_availability.resume_background();
        }
        result.map_err(map_rest_error)
    }
}

pub fn spawn_account_service(
    api_handle: MullvadRestHandle,
    token: Option<String>,
    api_availability: ApiAvailabilityHandle,
) -> AccountService {
    let accounts_proxy = AccountsProxy::new(api_handle);
    api_availability.pause_background();

    let api_availability_copy = api_availability.clone();
    let accounts_proxy_copy = accounts_proxy.clone();

    let (future, initial_check_abort_handle) = abortable(async move {
        let token = if let Some(token) = token {
            token
        } else {
            api_availability.pause_background();
            return;
        };

        let future_generator = move || {
            let expiry_fut = api_availability.when_online(accounts_proxy.get_expiry(token.clone()));
            let api_availability_copy = api_availability.clone();
            async move { handle_expiry_result_inner(&expiry_fut.await, &api_availability_copy) }
        };
        let should_retry = move |state_was_updated: &bool| -> bool { !*state_was_updated };
        retry_future(future_generator, should_retry, RETRY_BACKOFF_STRATEGY).await;
    });
    tokio::spawn(future);

    AccountService {
        api_availability: api_availability_copy,
        initial_check_abort_handle,
        proxy: accounts_proxy_copy,
    }
}

fn handle_expiry_result_inner(
    result: &Result<chrono::DateTime<chrono::Utc>, rest::Error>,
    api_availability: &ApiAvailabilityHandle,
) -> bool {
    match result {
        Ok(_expiry) if *_expiry >= chrono::Utc::now() => {
            api_availability.resume_background();
            true
        }
        Ok(_expiry) => {
            api_availability.pause_background();
            true
        }
        Err(mullvad_api::rest::Error::ApiError(_status, code)) => {
            if code == mullvad_api::INVALID_ACCOUNT {
                api_availability.pause_background();
                return true;
            }
            false
        }
        Err(_) => false,
    }
}

fn should_retry<T>(result: &Result<T, rest::Error>, api_handle: &ApiAvailabilityHandle) -> bool {
    match result {
        Err(error) if error.is_network_error() => !api_handle.get_state().is_offline(),
        _ => false,
    }
}

fn should_retry_backoff<T>(result: &Result<T, rest::Error>) -> bool {
    match result {
        Ok(_) => false,
        Err(error) => {
            if let rest::Error::ApiError(status, code) = error {
                *status != rest::StatusCode::NOT_FOUND
                    && code != mullvad_api::DEVICE_NOT_FOUND
                    && code != mullvad_api::INVALID_ACCOUNT
                    && code != mullvad_api::MAX_DEVICES_REACHED
                    && code != mullvad_api::PUBKEY_IN_USE
            } else {
                true
            }
        }
    }
}

fn map_rest_error(error: rest::Error) -> Error {
    match error {
        rest::Error::ApiError(_status, ref code) => match code.as_str() {
            // TODO: Implement invalid payment
            mullvad_api::DEVICE_NOT_FOUND => Error::InvalidDevice,
            mullvad_api::INVALID_ACCOUNT => Error::InvalidAccount,
            mullvad_api::MAX_DEVICES_REACHED => Error::MaxDevicesReached,
            mullvad_api::INVALID_VOUCHER => Error::InvalidVoucher,
            mullvad_api::VOUCHER_USED => Error::UsedVoucher,
            _ => Error::OtherRestError(error),
        },
        error => Error::OtherRestError(error),
    }
}
