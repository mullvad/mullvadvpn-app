use chrono::{DateTime, Utc};
use futures::{
    channel::{mpsc, oneshot},
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
    device::{Device, DeviceData, DeviceEvent, DeviceId},
    wireguard::{RotationInterval, WireguardData},
};
use std::{
    future::Future,
    path::Path,
    time::{Duration, SystemTime},
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
const KEY_CHECK_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// File that used to store account and device data.
const DEVICE_CACHE_FILENAME: &str = "device.json";

const RETRY_ACTION_INTERVAL: Duration = Duration::ZERO;
const RETRY_ACTION_MAX_RETRIES: usize = 2;

const RETRY_BACKOFF_INTERVAL_INITIAL: Duration = Duration::from_secs(4);
const RETRY_BACKOFF_INTERVAL_FACTOR: u32 = 5;
const RETRY_BACKOFF_INTERVAL_MAX: Duration = Duration::from_secs(24 * 60 * 60);

const VALIDITY_CACHE_TIMEOUT: Duration = Duration::from_secs(10);

const LOGOUT_TIMEOUT: Duration = Duration::from_secs(2);

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
    #[error(display = "The device update task is not running")]
    DeviceUpdaterCancelled(#[error(source)] oneshot::Canceled),
    #[error(display = "The account manager is down")]
    AccountManagerDown,
}

#[derive(Clone)]
pub(crate) enum InnerDeviceEvent {
    /// The device was removed due to user (or daemon) action.
    Logout,
    /// Logged in to a new device.
    Login(DeviceData),
    /// The device was updated remotely, but not its key.
    Updated(DeviceData),
    /// The key was rotated.
    RotatedKey(DeviceData),
    /// Device was removed because it was not found remotely.
    Revoked,
}

impl From<InnerDeviceEvent> for DeviceEvent {
    fn from(event: InnerDeviceEvent) -> DeviceEvent {
        match event {
            InnerDeviceEvent::Logout => DeviceEvent::revoke(false),
            InnerDeviceEvent::Login(data) => DeviceEvent::from_device(data, false),
            InnerDeviceEvent::Updated(data) => DeviceEvent::from_device(data, true),
            InnerDeviceEvent::RotatedKey(data) => DeviceEvent::from_device(data, false),
            InnerDeviceEvent::Revoked => DeviceEvent::revoke(true),
        }
    }
}

impl InnerDeviceEvent {
    fn data(&self) -> Option<&DeviceData> {
        match self {
            InnerDeviceEvent::Login(data) => Some(&data),
            InnerDeviceEvent::Updated(data) => Some(&data),
            InnerDeviceEvent::RotatedKey(data) => Some(&data),
            InnerDeviceEvent::Logout | InnerDeviceEvent::Revoked => None,
        }
    }

    fn into_data(self) -> Option<DeviceData> {
        match self {
            InnerDeviceEvent::Login(data) => Some(data),
            InnerDeviceEvent::Updated(data) => Some(data),
            InnerDeviceEvent::RotatedKey(data) => Some(data),
            InnerDeviceEvent::Logout | InnerDeviceEvent::Revoked => None,
        }
    }
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
    /// The device was valid but the key was replaced
    RotatedKey,
    /// The device was valid but one or more fields, such as ports, were replaced
    Updated,
    /// The device was not found remotely and was removed from the cache.
    Removed,
}

type ResponseTx<T> = oneshot::Sender<Result<T, Error>>;

enum AccountManagerCommand {
    Login(AccountToken, ResponseTx<()>),
    Logout(ResponseTx<()>),
    SetData(DeviceData, ResponseTx<()>),
    GetData(ResponseTx<Option<DeviceData>>),
    RotateKey(ResponseTx<()>),
    SetRotationInterval(RotationInterval, ResponseTx<()>),
    GetRotationInterval(ResponseTx<RotationInterval>),
    ValidateDevice(ResponseTx<ValidationResult>),
    ReceiveEvents(Box<dyn Sender<InnerDeviceEvent> + Send>, ResponseTx<()>),
    Shutdown(oneshot::Sender<()>),
}

#[derive(Clone)]
pub(crate) struct AccountManagerHandle {
    cmd_tx: mpsc::UnboundedSender<AccountManagerCommand>,
    pub account_service: AccountService,
    pub device_service: DeviceService,
}

impl AccountManagerHandle {
    pub async fn login(&self, token: AccountToken) -> Result<(), Error> {
        self.send_command(|tx| AccountManagerCommand::Login(token, tx))
            .await
    }

    pub async fn logout(&self) -> Result<(), Error> {
        self.send_command(|tx| AccountManagerCommand::Logout(tx))
            .await
    }

    pub async fn set(&self, data: DeviceData) -> Result<(), Error> {
        self.send_command(|tx| AccountManagerCommand::SetData(data, tx))
            .await
    }

    pub async fn data(&self) -> Result<Option<DeviceData>, Error> {
        self.send_command(|tx| AccountManagerCommand::GetData(tx))
            .await
    }

    pub async fn rotate_key(&self) -> Result<(), Error> {
        self.send_command(|tx| AccountManagerCommand::RotateKey(tx))
            .await
    }

    pub async fn set_rotation_interval(&self, interval: RotationInterval) -> Result<(), Error> {
        self.send_command(|tx| AccountManagerCommand::SetRotationInterval(interval, tx))
            .await
    }

    pub async fn rotation_interval(&self) -> Result<RotationInterval, Error> {
        self.send_command(|tx| AccountManagerCommand::GetRotationInterval(tx))
            .await
    }

    pub async fn validate_device(&self) -> Result<ValidationResult, Error> {
        self.send_command(|tx| AccountManagerCommand::ValidateDevice(tx))
            .await
    }

    pub async fn receive_events(
        &self,
        events_tx: impl Sender<InnerDeviceEvent> + Send + 'static,
    ) -> Result<(), Error> {
        self.send_command(|tx| {
            AccountManagerCommand::ReceiveEvents(Box::new(events_tx) as Box<_>, tx)
        })
        .await
    }

    pub async fn shutdown(self) {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .cmd_tx
            .unbounded_send(AccountManagerCommand::Shutdown(tx));
        let _ = rx.await;
    }

    async fn send_command<T>(
        &self,
        make_cmd: impl FnOnce(oneshot::Sender<Result<T, Error>>) -> AccountManagerCommand,
    ) -> Result<T, Error> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .unbounded_send(make_cmd(tx))
            .map_err(|_| Error::AccountManagerDown)?;
        rx.await.map_err(|_| Error::AccountManagerDown)?
    }
}

pub(crate) struct AccountManager {
    cacher: DeviceCacher,
    device_service: DeviceService,
    data: Option<DeviceData>,
    rotation_interval: RotationInterval,
    listeners: Vec<Box<dyn Sender<InnerDeviceEvent> + Send>>,
    last_validation: Option<SystemTime>,
}

impl AccountManager {
    pub async fn spawn(
        rest_handle: rest::MullvadRestHandle,
        api_availability: ApiAvailabilityHandle,
        settings_dir: &Path,
        initial_rotation_interval: RotationInterval,
    ) -> Result<AccountManagerHandle, Error> {
        let (cacher, data) = DeviceCacher::new(settings_dir).await?;
        let token = data.as_ref().map(|state| state.token.clone());
        let account_service =
            spawn_account_service(rest_handle.clone(), token, api_availability.clone());

        let (cmd_tx, cmd_rx) = mpsc::unbounded();

        let device_service = DeviceService::new(rest_handle, api_availability);
        let manager = AccountManager {
            cacher,
            device_service: device_service.clone(),
            data,
            rotation_interval: initial_rotation_interval,
            listeners: vec![],
            last_validation: None,
        };

        tokio::spawn(manager.run(cmd_rx));
        let handle = AccountManagerHandle {
            cmd_tx,
            account_service,
            device_service,
        };
        KeyUpdater::spawn(handle.clone()).await?;
        Ok(handle)
    }

    async fn run(mut self, mut cmd_rx: mpsc::UnboundedReceiver<AccountManagerCommand>) {
        let mut shutdown_tx = None;
        while let Some(cmd) = cmd_rx.next().await {
            match cmd {
                AccountManagerCommand::Shutdown(tx) => {
                    shutdown_tx = Some(tx);
                    break;
                }
                other => self.service_command(other).await,
            }
        }
        self.shutdown().await;
        if let Some(tx) = shutdown_tx {
            let _ = tx.send(());
        }
        log::debug!("Account manager has stopped");
    }

    async fn service_command(&mut self, cmd: AccountManagerCommand) {
        match cmd {
            AccountManagerCommand::Login(token, tx) => {
                let _ = tx.send(self.login(token).await);
            }
            AccountManagerCommand::Logout(tx) => {
                let _ = tx.send(self.logout().await);
            }
            AccountManagerCommand::SetData(data, tx) => {
                let _ = tx.send(self.set(data).await);
            }
            AccountManagerCommand::GetData(tx) => {
                let _ = tx.send(Ok(self.data.clone()));
            }
            AccountManagerCommand::RotateKey(tx) => {
                let _ = tx.send(self.rotate_key().await);
            }
            AccountManagerCommand::SetRotationInterval(interval, tx) => {
                self.rotation_interval = interval;
                let _ = tx.send(Ok(()));
            }
            AccountManagerCommand::GetRotationInterval(tx) => {
                let _ = tx.send(Ok(self.rotation_interval));
            }
            AccountManagerCommand::ValidateDevice(tx) => {
                let _ = tx.send(self.validate_device().await);
            }
            AccountManagerCommand::ReceiveEvents(events_tx, tx) => {
                let _ = tx.send(Ok(self.listeners.push(events_tx)));
            }
            AccountManagerCommand::Shutdown(_) => unreachable!("shutdown is handled earlier"),
        }
    }

    async fn login(&mut self, token: AccountToken) -> Result<(), Error> {
        let data = self.device_service.generate_for_account(token).await?;
        self.set(data).await?;
        Ok(())
    }

    async fn logout(&mut self) -> Result<(), Error> {
        if self.data.is_some() {
            self.cacher.write(None).await?;
            let _ = tokio::time::timeout(LOGOUT_TIMEOUT, self.logout_inner()).await;

            let event = InnerDeviceEvent::Logout;
            self.listeners
                .retain(|listener| listener.send(event.clone()).is_ok());
        }
        Ok(())
    }

    async fn logout_inner(&mut self) -> tokio::task::JoinHandle<()> {
        let prev_data = self.data.take();
        let service = self.device_service.clone();

        tokio::spawn(async move {
            if let Some(data) = prev_data {
                if let Err(error) = service
                    .remove_device_with_backoff(data.token, data.device.id)
                    .await
                {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to remove a previous device")
                    );
                }
            }
        })
    }

    #[inline]
    async fn set(&mut self, new_data: DeviceData) -> Result<(), Error> {
        self.set_inner(InnerDeviceEvent::Login(new_data)).await
    }

    async fn set_inner(&mut self, event: InnerDeviceEvent) -> Result<(), Error> {
        let data = event.data();
        if data == self.data.as_ref() {
            return Ok(());
        }

        self.cacher.write(data).await?;

        if self
            .data
            .as_ref()
            .map(|current| data.as_ref().map(|d| &d.device.id) != Some(&current.device.id))
            .unwrap_or(false)
        {
            // Remove the existing device if its ID differs. Otherwise, only update
            // the data.
            self.logout_inner().await;
        }

        self.data = data.cloned();

        self.listeners
            .retain(|listener| listener.send(event.clone()).is_ok());

        Ok(())
    }

    async fn rotate_key(&mut self) -> Result<(), Error> {
        // TODO: Update all data opportunistically?
        let data = self.data.as_ref().ok_or(Error::NoDevice)?;

        let wg_data = self
            .device_service
            .rotate_key(data.token.clone(), data.device.id.clone())
            .await?;

        // Copy the data to keep a predictable state if an error occurs.
        let mut new_data = data.clone();
        new_data.device.pubkey = wg_data.private_key.public_key();
        new_data.wg_data = wg_data;
        self.set_inner(InnerDeviceEvent::RotatedKey(new_data)).await
    }

    /// Check if the device is valid for the account, and yank it if it no longer exists.
    /// This also updates any associated data and returns whether it changed.
    async fn validate_device(&mut self) -> Result<ValidationResult, Error> {
        log::debug!("Checking whether the device is still valid");

        if let Some(result) = self.cached_validation() {
            log::debug!("The current device is still valid");
            return Ok(result);
        }

        let data = self.data.as_ref().ok_or(Error::NoDevice)?;

        match self
            .device_service
            .get(data.token.clone(), data.device.id.clone())
            .await
        {
            Ok(device) => {
                if device.pubkey == data.device.pubkey {
                    if device == data.device {
                        log::debug!("The current device is still valid");
                        Ok(ValidationResult::Valid)
                    } else {
                        log::debug!("Updating data for the current device");
                        // Copy the data to keep a predictable state if an error occurs.
                        let new_data = DeviceData {
                            device,
                            ..data.clone()
                        };
                        self.set_inner(InnerDeviceEvent::Updated(new_data)).await?;
                        Ok(ValidationResult::Updated)
                    }
                } else {
                    log::debug!("Rotating invalid WireGuard key");
                    self.rotate_key().await?;
                    Ok(ValidationResult::RotatedKey)
                }
            }
            Err(Error::InvalidAccount) | Err(Error::InvalidDevice) => {
                log::debug!("The current device is no longer valid for this account");

                self.cacher.write(None).await?;
                self.data = None;

                let event = InnerDeviceEvent::Revoked;
                self.listeners
                    .retain(|listener| listener.send(event.clone()).is_ok());

                Ok(ValidationResult::Removed)
            }
            Err(error) => Err(error),
        }
    }

    fn cached_validation(&mut self) -> Option<ValidationResult> {
        if self.data.is_none() {
            return None;
        }

        let now = SystemTime::now();

        let elapsed = self
            .last_validation
            .and_then(|last_check| now.duration_since(last_check).ok())
            .unwrap_or(VALIDITY_CACHE_TIMEOUT);

        if elapsed >= VALIDITY_CACHE_TIMEOUT {
            self.last_validation = Some(now);
            return None;
        }

        Some(ValidationResult::Valid)
    }

    async fn shutdown(self) {
        self.cacher.finalize().await;
    }
}

struct KeyUpdater {
    handle: AccountManagerHandle,
    rx: mpsc::UnboundedReceiver<InnerDeviceEvent>,
    data: Option<DeviceData>,
}

impl KeyUpdater {
    async fn spawn(handle: AccountManagerHandle) -> Result<(), Error> {
        let (tx, rx) = mpsc::unbounded();
        handle.receive_events(tx).await?;
        let data = handle.data().await?;
        let mut key_rotator = KeyUpdater { handle, rx, data };

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(KEY_CHECK_INTERVAL).await;

                if let Err(error) = key_rotator.check_key_validity().await {
                    if let Error::AccountManagerDown = error {
                        break;
                    }
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Stopping key rotation task due to an error")
                    );
                    break;
                }
            }
            log::debug!("Stopping key updater");
        });

        Ok(())
    }

    async fn check_key_validity(&mut self) -> Result<(), Error> {
        let rotation_interval = self.handle.rotation_interval().await?;
        let data = self.wait_for_data().await?;

        if (chrono::Utc::now()
            .signed_duration_since(data.wg_data.created)
            .num_seconds() as u64)
            < rotation_interval.as_duration().as_secs()
        {
            return Ok(());
        }

        let mut data = data.clone();

        let rotation_fut = self
            .handle
            .device_service
            .rotate_key_with_backoff(data.token.clone(), data.device.id.clone());

        match futures::future::select(Box::pin(rotation_fut), self.rx.next()).await {
            futures::future::Either::Left((Ok(wg_data), _)) => {
                log::debug!("Rotating WireGuard key");
                data.device.pubkey = wg_data.private_key.public_key();
                data.wg_data = wg_data;
                self.handle.set(data).await?;
            }
            futures::future::Either::Left((Err(error), _)) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Stopping key rotation due to an error")
                );

                // Forget the current device. Key rotation will restart when
                // it is updated in any way.
                self.data = None;
            }
            futures::future::Either::Right((event, _)) => {
                // Abort key rotation if the device changed
                if let Some(event) = event {
                    self.data = event.into_data();
                } else {
                    return Err(Error::AccountManagerDown);
                }
            }
        }

        Ok(())
    }

    async fn wait_for_data(&mut self) -> Result<&DeviceData, Error> {
        while let Ok(item) = self.rx.try_next() {
            match item {
                Some(event) => {
                    self.data = event.into_data();
                }
                None => return Err(Error::AccountManagerDown),
            }
        }

        match self.data {
            Some(ref data) => Ok(data),
            None => loop {
                let event = self.rx.next().await;
                match event {
                    Some(event) => {
                        if let Some(data) = event.into_data() {
                            self.data = Some(data);
                            break Ok(self.data.as_ref().unwrap());
                        }
                    }
                    None => break Err(Error::AccountManagerDown),
                }
            },
        }
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
    path: std::path::PathBuf,
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
            .open(&path)
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
                path,
            },
            device,
        ))
    }

    pub async fn write(&mut self, device: Option<&DeviceData>) -> Result<(), Error> {
        let data = serde_json::to_vec_pretty(&device).unwrap();

        self.file.get_mut().set_len(0).await?;
        self.file.seek(io::SeekFrom::Start(0)).await?;
        self.file.write_all(&data).await?;
        self.file.flush().await?;
        self.file.get_mut().sync_data().await?;

        Ok(())
    }

    pub async fn remove(self) -> Result<(), Error> {
        let path = {
            let DeviceCacher { path, file } = self;
            let std_file = file.into_inner().into_std().await;
            let _ = tokio::task::spawn_blocking(move || drop(std_file)).await;
            path
        };
        tokio::fs::remove_file(path).await?;
        Ok(())
    }

    async fn finalize(self) {
        let std_file = self.file.into_inner().into_std().await;
        let _ = tokio::task::spawn_blocking(move || drop(std_file)).await;
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

pub fn spawn_account_service(
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
            let expiry_fut = api_availability.when_online(accounts_proxy.get_expiry(token.clone()));
            let api_availability_copy = api_availability.clone();
            async move { handle_expiry_result_inner(&expiry_fut.await, &api_availability_copy) }
        };
        let should_retry = move |state_was_updated: &bool| -> bool { !*state_was_updated };
        retry_future(future_generator, should_retry, retry_strategy()).await;
    });
    tokio::spawn(future);

    AccountService {
        api_availability: api_availability_copy,
        initial_check_abort_handle,
        proxy: accounts_proxy_copy,
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
