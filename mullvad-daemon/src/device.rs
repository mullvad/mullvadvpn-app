use crate::DaemonEventSender;
use chrono::{DateTime, Utc};
use futures::{
    channel::mpsc,
    future::{abortable, AbortHandle},
    stream::StreamExt,
};
use mullvad_rpc::{
    availability::ApiAvailabilityHandle,
    rest::{self, Error as RestError, MullvadRestHandle},
    AccountsProxy, DevicesProxy,
};
use mullvad_types::{
    account::{AccountToken, VoucherSubmission},
    device::{Device, DeviceData, DeviceId},
    wireguard::{RotationInterval, WireguardData},
};
use std::{
    future::Future,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use talpid_core::{
    future_retry::{constant_interval, retry_future, retry_future_n, ExponentialBackoff, Jittered},
    mpsc::Sender,
};
use talpid_types::{net::wireguard::PrivateKey, ErrorExt};
use tokio::{
    fs,
    io::{self, AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};

/// How often to check whether the key has expired.
/// A short interval is used in case the computer is ever suspended.
const KEY_CHECK_INTERVAL: Duration = Duration::from_secs(60);

/// File that used to store account and device data.
const DEVICE_CACHE_FILENAME: &str = "device.json";

const RETRY_ACTION_INTERVAL: Duration = Duration::ZERO;
const RETRY_ACTION_MAX_RETRIES: usize = 2;

const RETRY_BACKOFF_INTERVAL_INITIAL: Duration = Duration::from_secs(4);
const RETRY_BACKOFF_INTERVAL_FACTOR: u32 = 5;
const RETRY_BACKOFF_INTERVAL_MAX: Duration = Duration::from_secs(24 * 60 * 60);

pub struct DeviceKeyEvent(pub DeviceData);

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "The account already has a maximum number of devices")]
    MaxDevicesReached,
    #[error(display = "No device is set")]
    NoDevice,
    #[error(display = "Device not found")]
    InvalidDevice,
    #[error(display = "Invalid account")]
    InvalidAccount,
    #[error(display = "Failed to read or write device cache")]
    DeviceIoError(#[error(source)] io::Error),
    #[error(display = "Failed parse device cache")]
    ParseDeviceCache(#[error(source)] serde_json::Error),
    #[error(display = "Unexpected HTTP request error")]
    OtherRestError(#[error(source)] rest::Error),
}

impl Error {
    pub fn is_network_error(&self) -> bool {
        if let Error::OtherRestError(error) = self {
            error.is_network_error()
        } else {
            false
        }
    }
}

pub enum ValidationResult {
    /// The device and key were valid.
    Valid,
    /// The device was valid but the key had to be replaced.
    RotatedKey(WireguardData),
    /// The device was not found remotely and was removed from the cache.
    Removed,
}

pub(crate) struct AccountManager {
    runtime: tokio::runtime::Handle,
    account_service: AccountService,
    device_service: DeviceService,
    inner: Arc<Mutex<AccountManagerInner>>,
    cache_update_tx: mpsc::UnboundedSender<Option<DeviceData>>,
    cache_task_join_handle: Option<tokio::task::JoinHandle<()>>,
    key_update_tx: DaemonEventSender<DeviceKeyEvent>,
    rotation_abort_handle: Option<AbortHandle>,
}

struct AccountManagerInner {
    data: Option<DeviceData>,
    rotation_interval: RotationInterval,
}

impl AccountManager {
    pub async fn new(
        runtime: tokio::runtime::Handle,
        rest_handle: rest::MullvadRestHandle,
        api_availability: ApiAvailabilityHandle,
        settings_dir: &Path,
        key_update_tx: DaemonEventSender<DeviceKeyEvent>,
    ) -> Result<AccountManager, Error> {
        let (mut cacher, device_data) = DeviceCacher::new(settings_dir).await?;
        let token = device_data.as_ref().map(|state| state.token.clone());
        let account_service = Account::new(
            runtime.clone(),
            rest_handle.clone(),
            token,
            api_availability.clone(),
        );
        let should_start_rotation = device_data.is_some();
        let inner = Arc::new(Mutex::new(AccountManagerInner {
            data: device_data,
            rotation_interval: RotationInterval::default(),
        }));

        let (cache_update_tx, mut cache_update_rx) = mpsc::unbounded();
        let cache_task_join_handle = runtime.spawn(async move {
            while let Some(new_device) = cache_update_rx.next().await {
                if let Err(error) = cacher.write(new_device).await {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to update device cache")
                    );
                }
            }
        });

        let mut manager = AccountManager {
            runtime,
            account_service,
            device_service: DeviceService::new(rest_handle, api_availability),
            inner,
            cache_update_tx,
            cache_task_join_handle: Some(cache_task_join_handle),
            key_update_tx,
            rotation_abort_handle: None,
        };

        if should_start_rotation {
            manager.start_key_rotation();
        }

        Ok(manager)
    }

    pub fn account_service(&self) -> AccountService {
        self.account_service.clone()
    }

    pub fn device_service(&self) -> DeviceService {
        self.device_service.clone()
    }

    pub async fn login(&mut self, token: AccountToken) -> Result<DeviceData, Error> {
        let data = self.device_service.generate_for_account(token).await?;
        self.logout();
        {
            let mut inner = self.inner.lock().unwrap();
            inner.data.replace(data.clone());
            let _ = self.cache_update_tx.unbounded_send(Some(data.clone()));
        }
        self.start_key_rotation();

        Ok(data)
    }

    pub fn set(&mut self, data: DeviceData) {
        self.logout();
        {
            let mut inner = self.inner.lock().unwrap();
            inner.data.replace(data.clone());
            let _ = self.cache_update_tx.unbounded_send(Some(data));
        }
        self.start_key_rotation();
    }

    /// Log out without waiting for the result.
    pub fn logout(&mut self) {
        let fut = self.logout_inner(true);
        self.runtime.spawn(async move {
            let result = fut.await;
            if let Err(error) = result {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to remove a previous device")
                );
            }
        });
    }

    /// Log out, and wait until the API has removed the device.
    #[cfg(not(target_os = "android"))]
    pub fn logout_wait(&mut self) -> impl Future<Output = Result<(), Error>> {
        self.logout_inner(false)
    }

    fn logout_inner(&mut self, use_backoff: bool) -> impl Future<Output = Result<(), Error>> {
        self.stop_key_rotation();
        let data = {
            let mut inner = self.inner.lock().unwrap();
            let _ = self.cache_update_tx.unbounded_send(None);
            inner.data.take()
        };
        let service = self.device_service.clone();
        async move {
            if let Some(data) = data {
                if use_backoff {
                    return service
                        .remove_device_with_backoff(data.token, data.device.id)
                        .await;
                } else {
                    return service.remove_device(data.token, data.device.id).await;
                }
            }
            Ok(())
        }
    }

    pub async fn rotate_key(&mut self) -> Result<WireguardData, Error> {
        let mut data = {
            let inner = self.inner.lock().unwrap();
            inner.data.as_ref().ok_or(Error::NoDevice)?.clone()
        };
        self.stop_key_rotation();
        let result = self
            .device_service
            .rotate_key(data.token.clone(), data.device.id.clone())
            .await;
        if let Ok(ref wg_data) = result {
            data.wg_data = wg_data.clone();
            data.device.pubkey = wg_data.private_key.public_key();
            let mut inner = self.inner.lock().unwrap();
            inner.data.replace(data.clone());
            let _ = self.cache_update_tx.unbounded_send(Some(data));
        }
        self.start_key_rotation();
        result
    }

    pub fn get(&self) -> Option<DeviceData> {
        self.inner.lock().unwrap().data.clone()
    }

    pub fn is_some(&self) -> bool {
        self.inner.lock().unwrap().data.is_some()
    }

    pub async fn set_rotation_interval(&mut self, interval: RotationInterval) {
        self.stop_key_rotation();
        let restart_rotation = {
            let mut inner = self.inner.lock().unwrap();
            inner.rotation_interval = interval;
            inner.data.is_some()
        };
        if restart_rotation {
            self.start_key_rotation();
        }
    }

    /// Check if the device is valid for the account, and yank it if it no longer exists.
    pub async fn validate_device(&mut self) -> Result<ValidationResult, Error> {
        let data = {
            let inner = self.inner.lock().unwrap();
            inner.data.as_ref().ok_or(Error::NoDevice)?.clone()
        };

        match self.device_service.get(data.token, data.device.id).await {
            Ok(device) => {
                if device.pubkey == data.device.pubkey {
                    log::debug!("The current device is still valid");
                    Ok(ValidationResult::Valid)
                } else {
                    log::debug!("Rotating invalid WireGuard key");
                    Ok(ValidationResult::RotatedKey(self.rotate_key().await?))
                }
            }
            Err(Error::InvalidAccount) | Err(Error::InvalidDevice) => {
                log::debug!("The current device is no longer valid for this account");
                self.stop_key_rotation();
                {
                    self.inner.lock().unwrap().data.take();
                    let _ = self.cache_update_tx.unbounded_send(None);
                }
                Ok(ValidationResult::Removed)
            }
            Err(error) => Err(error),
        }
    }

    fn start_key_rotation(&mut self) {
        self.stop_key_rotation();

        let service = self.device_service.clone();
        let inner = self.inner.clone();
        let cache_update_tx = self.cache_update_tx.clone();
        let key_update_tx = self.key_update_tx.clone();

        let (task, abort_handle) = abortable(async move {
            loop {
                tokio::time::sleep(KEY_CHECK_INTERVAL).await;

                let rotation_interval = { inner.lock().unwrap().rotation_interval.clone() };

                let mut state = {
                    match inner.lock().unwrap().data.as_ref() {
                        Some(device_config) => device_config.clone(),
                        None => continue,
                    }
                };

                if (chrono::Utc::now()
                    .signed_duration_since(state.wg_data.created)
                    .num_seconds() as u64)
                    < rotation_interval.as_duration().as_secs()
                {
                    continue;
                }

                match service
                    .rotate_key_with_backoff(state.token.clone(), state.device.id.clone())
                    .await
                {
                    Ok(wg_data) => {
                        state.device.pubkey = wg_data.private_key.public_key();
                        state.wg_data = wg_data;
                        {
                            let mut inner = inner.lock().unwrap();
                            inner.data.replace(state.clone());
                            let _ = cache_update_tx.unbounded_send(Some(state.clone()));
                        }
                        let _ = key_update_tx.send(DeviceKeyEvent(state));
                    }
                    Err(error) => {
                        log::debug!("{}", error.display_chain_with_msg("Stopping key rotation"));
                    }
                }
            }
        });
        self.runtime.spawn(task);
        self.rotation_abort_handle = Some(abort_handle);
    }

    fn stop_key_rotation(&mut self) {
        if let Some(abort_handle) = self.rotation_abort_handle.take() {
            abort_handle.abort();
        }
    }

    /// Consumes the object and completes when there is nothing left to write to
    /// the cache file.
    pub fn finalize(mut self) -> impl Future<Output = ()> {
        let join_handle = self.cache_task_join_handle.take();
        drop(self);

        async move {
            if let Some(join_handle) = join_handle {
                let _ = join_handle.await;
            }
        }
    }
}

impl Drop for AccountManager {
    fn drop(&mut self) {
        self.stop_key_rotation();
    }
}

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
    pub async fn generate_for_account(&self, token: AccountToken) -> Result<DeviceData, Error> {
        let private_key = PrivateKey::new_from_random();
        let pubkey = private_key.public_key();

        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        let token_copy = token.clone();
        let (device, addresses) = retry_future_n(
            move || proxy.create(token_copy.clone(), pubkey.clone()),
            move |result| should_retry(result, &api_handle),
            constant_interval(RETRY_ACTION_INTERVAL),
            RETRY_ACTION_MAX_RETRIES,
        )
        .await
        .map_err(map_rest_error)?;

        Ok(DeviceData {
            token,
            device,
            wg_data: WireguardData {
                private_key,
                addresses,
                created: Utc::now(),
            },
        })
    }

    pub async fn generate_for_account_with_backoff(
        &self,
        token: AccountToken,
    ) -> Result<DeviceData, Error> {
        let private_key = PrivateKey::new_from_random();
        let pubkey = private_key.public_key();

        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        let token_copy = token.clone();
        let (device, addresses) = retry_future(
            move || api_handle.when_online(proxy.create(token_copy.clone(), pubkey.clone())),
            should_retry_backoff,
            retry_strategy(),
        )
        .await
        .map_err(map_rest_error)?;

        Ok(DeviceData {
            token,
            device,
            wg_data: WireguardData {
                private_key,
                addresses,
                created: Utc::now(),
            },
        })
    }

    pub async fn remove_device(&self, token: AccountToken, device: DeviceId) -> Result<(), Error> {
        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        retry_future_n(
            move || proxy.remove(token.clone(), device.clone()),
            move |result| should_retry(result, &api_handle),
            constant_interval(RETRY_ACTION_INTERVAL),
            RETRY_ACTION_MAX_RETRIES,
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

        let retry_strategy = Jittered::jitter(
            ExponentialBackoff::new(
                RETRY_BACKOFF_INTERVAL_INITIAL,
                RETRY_BACKOFF_INTERVAL_FACTOR,
            ), // Not setting a maximum interval
        );

        retry_future(
            // NOTE: Not honoring "paused" state, because the account may have no time on it.
            move || api_handle.when_online(proxy.remove(token.clone(), device.clone())),
            should_retry_backoff,
            retry_strategy,
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
        let addresses = retry_future_n(
            move || proxy.replace_wg_key(token.clone(), device.clone(), pubkey.clone()),
            move |result| should_retry(result, &api_handle),
            constant_interval(RETRY_ACTION_INTERVAL),
            RETRY_ACTION_MAX_RETRIES,
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

        let addresses = retry_future(
            move || {
                api_handle.when_bg_resumes(proxy.replace_wg_key(
                    token.clone(),
                    device.clone(),
                    pubkey.clone(),
                ))
            },
            should_retry_backoff,
            retry_strategy(),
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
        retry_future_n(
            move || proxy.list(token.clone()),
            move |result| should_retry(result, &api_handle),
            constant_interval(RETRY_ACTION_INTERVAL),
            RETRY_ACTION_MAX_RETRIES,
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
            retry_strategy(),
        )
        .await
        .map_err(map_rest_error)
    }

    pub async fn get(&self, token: AccountToken, device: DeviceId) -> Result<Device, Error> {
        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        retry_future_n(
            move || proxy.get(token.clone(), device.clone()),
            move |result| should_retry(result, &api_handle),
            constant_interval(RETRY_ACTION_INTERVAL),
            RETRY_ACTION_MAX_RETRIES,
        )
        .await
        .map_err(map_rest_error)
    }
}

pub struct DeviceCacher {
    file: io::BufWriter<fs::File>,
}

impl DeviceCacher {
    pub async fn new(settings_dir: &Path) -> Result<(DeviceCacher, Option<DeviceData>), Error> {
        let mut options = std::fs::OpenOptions::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            // exclusive access
            options.share_mode(0);
        }

        let path = settings_dir.join(DEVICE_CACHE_FILENAME);
        let cache_exists = path.is_file();

        let mut file = fs::OpenOptions::from(options)
            .write(true)
            .read(true)
            .create(true)
            .open(path)
            .await?;

        let device: Option<DeviceData> = if cache_exists {
            let mut reader = io::BufReader::new(&mut file);
            let mut buffer = String::new();
            reader.read_to_string(&mut buffer).await?;
            if !buffer.is_empty() {
                serde_json::from_str(&buffer)?
            } else {
                None
            }
        } else {
            None
        };

        Ok((
            DeviceCacher {
                file: io::BufWriter::new(file),
            },
            device,
        ))
    }

    pub async fn write(&mut self, device: Option<DeviceData>) -> Result<(), Error> {
        let data = serde_json::to_vec_pretty(&device).unwrap();

        self.file.get_mut().set_len(0).await?;
        self.file.seek(io::SeekFrom::Start(0)).await?;
        self.file.write_all(&data).await?;
        self.file.flush().await?;
        self.file.get_mut().sync_data().await?;

        Ok(())
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
        retry_future_n(
            move || proxy.create_account(),
            move |result| should_retry(result, &api_handle),
            constant_interval(RETRY_ACTION_INTERVAL),
            RETRY_ACTION_MAX_RETRIES,
        )
    }

    pub fn get_www_auth_token(
        &self,
        account: AccountToken,
    ) -> impl Future<Output = Result<String, rest::Error>> {
        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        retry_future_n(
            move || proxy.get_www_auth_token(account.clone()),
            move |result| should_retry(result, &api_handle),
            constant_interval(RETRY_ACTION_INTERVAL),
            RETRY_ACTION_MAX_RETRIES,
        )
    }

    pub async fn check_expiry(&self, token: AccountToken) -> Result<DateTime<Utc>, rest::Error> {
        let proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        let result = retry_future_n(
            move || proxy.get_expiry(token.clone()),
            move |result| should_retry(result, &api_handle),
            constant_interval(RETRY_ACTION_INTERVAL),
            RETRY_ACTION_MAX_RETRIES,
        )
        .await;
        if handle_expiry_result_inner(&result, &self.api_availability) {
            self.initial_check_abort_handle.abort();
        }
        result
    }

    pub async fn submit_voucher(
        &mut self,
        account_token: AccountToken,
        voucher: String,
    ) -> Result<VoucherSubmission, rest::Error> {
        let mut proxy = self.proxy.clone();
        let api_handle = self.api_availability.clone();
        let result = retry_future_n(
            move || proxy.submit_voucher(account_token.clone(), voucher.clone()),
            move |result| should_retry(result, &api_handle),
            constant_interval(RETRY_ACTION_INTERVAL),
            RETRY_ACTION_MAX_RETRIES,
        )
        .await;
        if result.is_ok() {
            self.initial_check_abort_handle.abort();
            self.api_availability.resume_background();
        }
        result
    }
}

struct Account(());

impl Account {
    pub fn new(
        runtime: tokio::runtime::Handle,
        rpc_handle: MullvadRestHandle,
        token: Option<String>,
        api_availability: ApiAvailabilityHandle,
    ) -> AccountService {
        let accounts_proxy = AccountsProxy::new(rpc_handle);
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
                let expiry_fut =
                    api_availability.when_online(accounts_proxy.get_expiry(token.clone()));
                let api_availability_copy = api_availability.clone();
                async move { handle_expiry_result_inner(&expiry_fut.await, &api_availability_copy) }
            };
            let should_retry = move |state_was_updated: &bool| -> bool { !*state_was_updated };
            retry_future(future_generator, should_retry, retry_strategy()).await;
        });
        runtime.spawn(future);

        AccountService {
            api_availability: api_availability_copy,
            initial_check_abort_handle,
            proxy: accounts_proxy_copy,
        }
    }
}

fn handle_expiry_result_inner(
    result: &Result<chrono::DateTime<chrono::Utc>, mullvad_rpc::rest::Error>,
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
        Err(mullvad_rpc::rest::Error::ApiError(_status, code)) => {
            if code == mullvad_rpc::INVALID_ACCOUNT {
                api_availability.pause_background();
                return true;
            }
            false
        }
        Err(_) => false,
    }
}

fn should_retry<T>(result: &Result<T, RestError>, api_handle: &ApiAvailabilityHandle) -> bool {
    match result {
        Err(error) if error.is_network_error() => !api_handle.get_state().is_offline(),
        _ => false,
    }
}

fn should_retry_backoff<T>(result: &Result<T, RestError>) -> bool {
    match result {
        Ok(_) => false,
        Err(error) => {
            if let RestError::ApiError(status, code) = error {
                *status != rest::StatusCode::NOT_FOUND
                    && code != mullvad_rpc::INVALID_ACCOUNT
                    && code != mullvad_rpc::MAX_DEVICES_REACHED
                    && code != mullvad_rpc::PUBKEY_IN_USE
            } else {
                true
            }
        }
    }
}

fn map_rest_error(error: rest::Error) -> Error {
    match error {
        RestError::ApiError(status, ref code) => {
            if status == rest::StatusCode::NOT_FOUND {
                return Error::InvalidDevice;
            }
            match code.as_str() {
                mullvad_rpc::INVALID_ACCOUNT => Error::InvalidAccount,
                mullvad_rpc::MAX_DEVICES_REACHED => Error::MaxDevicesReached,
                _ => Error::OtherRestError(error),
            }
        }
        error => Error::OtherRestError(error),
    }
}

fn retry_strategy() -> Jittered<ExponentialBackoff> {
    Jittered::jitter(
        ExponentialBackoff::new(
            RETRY_BACKOFF_INTERVAL_INITIAL,
            RETRY_BACKOFF_INTERVAL_FACTOR,
        )
        .max_delay(RETRY_BACKOFF_INTERVAL_MAX),
    )
}
