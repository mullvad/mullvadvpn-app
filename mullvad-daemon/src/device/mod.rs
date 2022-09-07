use chrono::{DateTime, Utc};
use futures::{
    channel::{mpsc, oneshot},
    stream::StreamExt,
};

use mullvad_api::rest;
use mullvad_types::{
    account::{AccountToken, VoucherSubmission},
    device::{
        AccountAndDevice, Device, DeviceEvent, DeviceEventCause, DeviceId, DeviceName, DevicePort,
        DeviceState,
    },
    wireguard::{self, RotationInterval, WireguardData},
};
use std::{
    future::Future,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};
use talpid_core::mpsc::Sender;
use talpid_types::{net::TunnelType, tunnel::TunnelStateTransition, ErrorExt};
use tokio::{
    fs,
    io::{self, AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};

mod api;
mod service;
pub(crate) use service::{AccountService, DeviceService};

/// File that used to store account and device data.
const DEVICE_CACHE_FILENAME: &str = "device.json";

/// How long to keep the known status for [AccountManagerHandle::validate_device].
const VALIDITY_CACHE_TIMEOUT: Duration = Duration::from_secs(10);

/// How long to wait on logout (device removal) before letting it continue as a background task.
const LOGOUT_TIMEOUT: Duration = Duration::from_secs(2);

/// Validate the current device once for every `WG_DEVICE_CHECK_THRESHOLD` failed attempts
/// to set up a WireGuard tunnel.
const WG_DEVICE_CHECK_THRESHOLD: usize = 2;

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
    #[error(display = "Invalid voucher code")]
    InvalidVoucher,
    #[error(display = "The voucher has already been used")]
    UsedVoucher,
    #[error(display = "Failed to read or write device cache")]
    DeviceIoError(#[error(source)] io::Error),
    #[error(display = "Failed parse device cache")]
    ParseDeviceCache(#[error(source)] serde_json::Error),
    #[error(display = "Unexpected HTTP request error")]
    OtherRestError(#[error(source)] rest::Error),
    #[error(display = "The device update task is not running")]
    Cancelled,
    /// Intended to be broadcast to requesters
    #[error(display = "Broadcast error")]
    ResponseFailure(#[error(source)] Arc<Error>),
    #[error(display = "Account changed during operation")]
    AccountChange,
    #[error(display = "The account manager is down")]
    AccountManagerDown,
}

/// Contains the current device state.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrivateDeviceState {
    LoggedIn(PrivateAccountAndDevice),
    LoggedOut,
    Revoked,
}

impl PrivateDeviceState {
    /// Returns whether the device is in the logged in state.
    pub fn logged_in(&self) -> bool {
        matches!(self, PrivateDeviceState::LoggedIn(_))
    }

    /// Returns whether the state is logged out, as opposed to
    /// logged in or revoked.
    pub fn logged_out(&self) -> bool {
        matches!(self, PrivateDeviceState::LoggedOut)
    }

    /// Returns the logged in device config.
    pub fn device(&self) -> Option<&PrivateAccountAndDevice> {
        match self {
            PrivateDeviceState::LoggedIn(device) => Some(device),
            _ => None,
        }
    }

    /// Returns the logged in device config.
    pub fn into_device(self) -> Option<PrivateAccountAndDevice> {
        match self {
            PrivateDeviceState::LoggedIn(device) => Some(device),
            _ => None,
        }
    }

    /// Sets the state to `Revoked`.
    fn revoke(&mut self) {
        *self = PrivateDeviceState::Revoked;
    }

    /// Sets the state to `LoggedOut` and returns the logged-in device, if one exists.
    fn logout(&mut self) -> Option<PrivateAccountAndDevice> {
        match std::mem::replace(self, PrivateDeviceState::LoggedOut) {
            PrivateDeviceState::LoggedIn(data) => Some(data),
            _ => None,
        }
    }
}

impl From<PrivateDeviceState> for DeviceState {
    fn from(state: PrivateDeviceState) -> Self {
        match state {
            PrivateDeviceState::LoggedIn(dev) => DeviceState::LoggedIn(AccountAndDevice::from(dev)),
            PrivateDeviceState::LoggedOut => DeviceState::LoggedOut,
            PrivateDeviceState::Revoked => DeviceState::Revoked,
        }
    }
}

/// Same as [PrivateDevice] but also contains the associated account token.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PrivateAccountAndDevice {
    pub account_token: AccountToken,
    pub device: PrivateDevice,
}

impl From<PrivateAccountAndDevice> for AccountAndDevice {
    fn from(config: PrivateAccountAndDevice) -> Self {
        AccountAndDevice {
            account_token: config.account_token,
            device: Device::from(config.device),
        }
    }
}

/// Device type that contains private data.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PrivateDevice {
    pub id: DeviceId,
    pub name: DeviceName,
    pub wg_data: wireguard::WireguardData,
    pub ports: Vec<DevicePort>,
    // FIXME: Reasonable default to avoid migration code for the field,
    // as it was previously missing.
    // This attribute may be removed once upgrades from `2022.2-beta1`
    // no longer need to be supported.
    #[serde(default)]
    pub hijack_dns: bool,
    // FIXME: Incorrect but reasonable default to avoid migration code
    // for the field, as it was previously missing.
    // The value is corrected when the device is validated or updated.
    // This attribute may be removed once upgrades from `2022.2-beta1`
    // no longer need to be supported.
    #[serde(default = "Utc::now")]
    pub created: DateTime<Utc>,
}

impl PrivateDevice {
    /// Construct a private device from a `WireguardData` and a `Device`. Fails if the pubkey of
    /// `device` does not match that of `wg_data`.
    pub fn try_from_device(
        device: Device,
        wg_data: wireguard::WireguardData,
    ) -> Result<Self, Error> {
        if device.pubkey != wg_data.private_key.public_key() {
            return Err(Error::InvalidDevice);
        }
        Ok(Self {
            id: device.id,
            name: device.name,
            wg_data,
            ports: device.ports,
            hijack_dns: device.hijack_dns,
            created: device.created,
        })
    }

    /// Update all device details that are present in both types. Fails if the pubkey of `device`
    /// does not match that of `wg_data`.
    fn update(&mut self, device: Device) -> Result<(), Error> {
        if device.pubkey != self.wg_data.private_key.public_key() {
            return Err(Error::InvalidDevice);
        }
        self.id = device.id;
        self.ports = device.ports;
        self.name = device.name;
        self.hijack_dns = device.hijack_dns;
        self.created = device.created;
        Ok(())
    }
}

impl From<PrivateDevice> for Device {
    fn from(device: PrivateDevice) -> Self {
        Device {
            id: device.id,
            ports: device.ports,
            pubkey: device.wg_data.private_key.public_key(),
            name: device.name,
            hijack_dns: device.hijack_dns,
            created: device.created,
        }
    }
}

#[derive(Clone)]
pub(crate) enum AccountEvent {
    /// Emitted when the device state changes.
    Device(PrivateDeviceEvent),
    /// Emitted when the account expiry is fetched.
    Expiry(DateTime<Utc>),
}

#[derive(Clone)]
pub(crate) enum PrivateDeviceEvent {
    /// Logged in on a new device.
    Login(PrivateAccountAndDevice),
    /// The device was removed due to user (or daemon) action.
    Logout,
    /// Device was removed because it was not found remotely.
    Revoked,
    /// The device was updated remotely, but not its key.
    Updated(PrivateAccountAndDevice),
    /// The key was rotated.
    RotatedKey(PrivateAccountAndDevice),
}

impl From<PrivateDeviceEvent> for DeviceEvent {
    fn from(event: PrivateDeviceEvent) -> DeviceEvent {
        let cause = match event {
            PrivateDeviceEvent::Login(_) => DeviceEventCause::LoggedIn,
            PrivateDeviceEvent::Logout => DeviceEventCause::LoggedOut,
            PrivateDeviceEvent::Revoked => DeviceEventCause::Revoked,
            PrivateDeviceEvent::Updated(_) => DeviceEventCause::Updated,
            PrivateDeviceEvent::RotatedKey(_) => DeviceEventCause::RotatedKey,
        };
        let new_state = DeviceState::from(event.state());
        DeviceEvent { cause, new_state }
    }
}

impl PrivateDeviceEvent {
    pub fn state(self) -> PrivateDeviceState {
        match self {
            PrivateDeviceEvent::Login(config) => PrivateDeviceState::LoggedIn(config),
            PrivateDeviceEvent::Updated(config) => PrivateDeviceState::LoggedIn(config),
            PrivateDeviceEvent::RotatedKey(config) => PrivateDeviceState::LoggedIn(config),
            PrivateDeviceEvent::Logout => PrivateDeviceState::LoggedOut,
            PrivateDeviceEvent::Revoked => PrivateDeviceState::Revoked,
        }
    }
}

impl Error {
    pub fn is_network_error(&self) -> bool {
        if let Error::OtherRestError(error) = self.unpack() {
            error.is_network_error()
        } else {
            false
        }
    }

    pub fn is_aborted(&self) -> bool {
        if let Error::OtherRestError(error) = self.unpack() {
            error.is_aborted()
        } else {
            false
        }
    }

    pub fn unpack(&self) -> &Error {
        if let Error::ResponseFailure(ref inner) = self {
            inner
        } else {
            self
        }
    }
}

type ResponseTx<T> = oneshot::Sender<Result<T, Error>>;

enum AccountManagerCommand {
    Login(AccountToken, ResponseTx<()>),
    Logout(ResponseTx<()>),
    SetData(PrivateAccountAndDevice, ResponseTx<()>),
    GetData(ResponseTx<PrivateDeviceState>),
    GetDataAfterLogin(ResponseTx<PrivateDeviceState>),
    RotateKey(ResponseTx<()>),
    SetRotationInterval(RotationInterval, ResponseTx<()>),
    ValidateDevice(ResponseTx<()>),
    SubmitVoucher(String, ResponseTx<VoucherSubmission>),
    CheckExpiry(ResponseTx<DateTime<Utc>>),
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
        self.send_command(AccountManagerCommand::Logout).await
    }

    pub async fn set(&self, data: PrivateAccountAndDevice) -> Result<(), Error> {
        self.send_command(|tx| AccountManagerCommand::SetData(data, tx))
            .await
    }

    pub async fn data(&self) -> Result<PrivateDeviceState, Error> {
        self.send_command(AccountManagerCommand::GetData).await
    }

    pub async fn data_after_login(&self) -> Result<PrivateDeviceState, Error> {
        self.send_command(AccountManagerCommand::GetDataAfterLogin)
            .await
    }

    pub async fn rotate_key(&self) -> Result<(), Error> {
        self.send_command(AccountManagerCommand::RotateKey).await
    }

    pub async fn set_rotation_interval(&self, interval: RotationInterval) -> Result<(), Error> {
        self.send_command(|tx| AccountManagerCommand::SetRotationInterval(interval, tx))
            .await
    }

    pub async fn validate_device(&self) -> Result<(), Error> {
        self.send_command(AccountManagerCommand::ValidateDevice)
            .await
    }

    pub async fn submit_voucher(&self, voucher: String) -> Result<VoucherSubmission, Error> {
        self.send_command(move |tx| AccountManagerCommand::SubmitVoucher(voucher, tx))
            .await
    }

    pub async fn check_expiry(&self) -> Result<DateTime<Utc>, Error> {
        self.send_command(AccountManagerCommand::CheckExpiry).await
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
    account_service: AccountService,
    device_service: DeviceService,
    data: PrivateDeviceState,
    rotation_interval: RotationInterval,
    listeners: Vec<Box<dyn Sender<AccountEvent> + Send>>,
    last_validation: Option<SystemTime>,
    validation_requests: Vec<ResponseTx<()>>,
    expiry_requests: Vec<ResponseTx<DateTime<Utc>>>,
    rotation_requests: Vec<ResponseTx<()>>,
    data_requests: Vec<ResponseTx<PrivateDeviceState>>,
}

impl AccountManager {
    /// Starts the account manager actor and returns a handle to it as well as the
    /// current device.
    pub async fn spawn(
        rest_handle: rest::MullvadRestHandle,
        settings_dir: &Path,
        initial_rotation_interval: RotationInterval,
        listener_tx: impl Sender<AccountEvent> + Send + 'static,
    ) -> Result<(AccountManagerHandle, PrivateDeviceState), Error> {
        let (cacher, data) = DeviceCacher::new(settings_dir).await?;
        let token = data.device().map(|state| state.account_token.clone());
        let api_availability = rest_handle.availability.clone();
        let account_service =
            service::spawn_account_service(rest_handle.clone(), token, api_availability.clone());

        let (cmd_tx, cmd_rx) = mpsc::unbounded();

        let device_service = DeviceService::new(rest_handle, api_availability);
        let manager = AccountManager {
            cacher,
            account_service: account_service.clone(),
            device_service: device_service.clone(),
            data: data.clone(),
            rotation_interval: initial_rotation_interval,
            listeners: vec![Box::new(listener_tx)],
            last_validation: None,
            validation_requests: vec![],
            expiry_requests: vec![],
            rotation_requests: vec![],
            data_requests: vec![],
        };

        tokio::spawn(manager.run(cmd_rx));
        let handle = AccountManagerHandle {
            cmd_tx,
            account_service,
            device_service,
        };
        Ok((handle, data))
    }

    async fn run(mut self, mut cmd_rx: mpsc::UnboundedReceiver<AccountManagerCommand>) {
        let mut shutdown_tx = None;
        let mut current_api_call = api::CurrentApiCall::new();

        loop {
            futures::select! {
                api_result = current_api_call => {
                    self.consume_api_result(api_result, &mut current_api_call).await;
                }

                cmd = cmd_rx.next() => {
                    match cmd {
                        Some(AccountManagerCommand::Shutdown(tx)) => {
                            shutdown_tx = Some(tx);
                            break;
                        }
                        Some(AccountManagerCommand::Login(token, tx)) => {
                            let job = self.device_service
                                .generate_for_account(token);
                            current_api_call.set_login(Box::pin(job), tx);
                        }
                        Some(AccountManagerCommand::Logout(tx)) => {
                            current_api_call.clear();
                            self.logout(tx).await;
                        }
                        Some(AccountManagerCommand::SetData(data, tx)) => {
                            let _ = tx.send(self.set(PrivateDeviceEvent::Login(data)).await);
                        }
                        Some(AccountManagerCommand::GetData(tx)) => {
                            let _ = tx.send(Ok(self.data.clone()));
                        }
                        Some(AccountManagerCommand::GetDataAfterLogin(tx)) => {
                            if current_api_call.is_logging_in() {
                                self.data_requests.push(tx);
                            } else {
                                let _ = tx.send(Ok(self.data.clone()));
                            }
                        }
                        Some(AccountManagerCommand::RotateKey(tx)) => {
                            if current_api_call.is_logging_in() {
                                let _ = tx.send(Err(Error::AccountChange));
                                continue
                            }
                            if current_api_call.is_validating() {
                                self.rotation_requests.push(tx);
                                continue
                            }
                            match self.initiate_key_rotation() {
                                Ok(api_call) => {
                                    current_api_call.set_oneshot_rotation(Box::pin(api_call));
                                    self.rotation_requests.push(tx);
                                },
                                Err(err) =>  {
                                    let _ = tx.send(Err(err));
                                }
                            }
                        }
                        Some(AccountManagerCommand::SetRotationInterval(interval, tx)) => {
                            self.rotation_interval = interval;
                            if current_api_call.is_running_timed_totation() {
                                current_api_call.clear();
                            }
                            let _ = tx.send(Ok(()));
                        }
                        Some(AccountManagerCommand::ValidateDevice(tx)) => {
                            self.handle_validation_request(tx, &mut current_api_call);
                        },
                        Some(AccountManagerCommand::SubmitVoucher(voucher, tx)) => {
                            self.handle_voucher_submission(tx, voucher, &mut current_api_call);
                        },
                        Some(AccountManagerCommand::CheckExpiry(tx)) => {
                            self.handle_expiry_request(tx, &mut current_api_call);
                        },

                        None => {
                            break;
                        }
                    }
                }
            }

            if current_api_call.is_idle() {
                if let Some(timed_rotation) = self.spawn_timed_key_rotation() {
                    current_api_call.set_timed_rotation(Box::pin(timed_rotation))
                }
            }
        }
        self.shutdown().await;
        if let Some(tx) = shutdown_tx {
            let _ = tx.send(());
        }
        log::debug!("Account manager has stopped");
    }

    fn handle_validation_request(
        &mut self,
        tx: ResponseTx<()>,
        current_api_call: &mut api::CurrentApiCall,
    ) {
        if current_api_call.is_logging_in() {
            let _ = tx.send(Err(Error::AccountChange));
            return;
        }
        if current_api_call.is_validating() {
            self.validation_requests.push(tx);
            return;
        }
        if !self.needs_validation() {
            let _ = tx.send(Ok(()));
            return;
        }

        match self.validation_call() {
            Ok(call) => {
                current_api_call.set_validation(Box::pin(call));
                self.validation_requests.push(tx);
            }
            Err(err) => {
                let _ = tx.send(Err(err));
            }
        }
    }

    fn handle_voucher_submission(
        &mut self,
        tx: ResponseTx<VoucherSubmission>,
        voucher: String,
        current_api_call: &mut api::CurrentApiCall,
    ) {
        if current_api_call.is_logging_in() {
            let _ = tx.send(Err(Error::AccountChange));
            return;
        }

        let create_submission = move || {
            let old_config = self.data.device().ok_or(Error::NoDevice)?;
            let account_token = old_config.account_token.clone();
            let account_service = self.account_service.clone();
            Ok(async move { account_service.submit_voucher(account_token, voucher).await })
        };

        match create_submission() {
            Ok(call) => {
                current_api_call.set_voucher_submission(Box::pin(call), tx);
            }
            Err(err) => {
                let _ = tx.send(Err(err));
            }
        }
    }

    fn handle_expiry_request(
        &mut self,
        tx: ResponseTx<DateTime<Utc>>,
        current_api_call: &mut api::CurrentApiCall,
    ) {
        if current_api_call.is_logging_in() {
            let _ = tx.send(Err(Error::AccountChange));
            return;
        }
        if current_api_call.is_checking_expiry() {
            self.expiry_requests.push(tx);
            return;
        }

        match self.expiry_call() {
            Ok(call) => {
                current_api_call.set_expiry_check(Box::pin(call));
                self.expiry_requests.push(tx);
            }
            Err(err) => {
                let _ = tx.send(Err(err));
            }
        }
    }

    async fn consume_api_result(
        &mut self,
        result: api::ApiResult,
        api_call: &mut api::CurrentApiCall,
    ) {
        use api::ApiResult::*;
        match result {
            Login(data, tx) => self.consume_login(data, tx).await,
            Rotation(rotation_response) => self.consume_rotation_result(rotation_response).await,
            Validation(data_response) => self.consume_validation(data_response, api_call).await,
            VoucherSubmission(data_response, tx) => {
                self.consume_voucher_result(data_response, tx).await
            }
            ExpiryCheck(data_response) => self.consume_expiry_result(data_response).await,
        }
    }

    async fn consume_login(
        &mut self,
        device_response: Result<PrivateAccountAndDevice, Error>,
        tx: ResponseTx<()>,
    ) {
        let _ =
            tx.send(async { self.set(PrivateDeviceEvent::Login(device_response?)).await }.await);
        let data = self.data.clone();
        Self::drain_requests(&mut self.data_requests, || Ok(data.clone()));
    }

    async fn consume_voucher_result(
        &mut self,
        response: Result<VoucherSubmission, Error>,
        tx: ResponseTx<VoucherSubmission>,
    ) {
        match &response {
            Ok(submission) => {
                // Send expiry update event
                let event = AccountEvent::Expiry(submission.new_expiry);
                self.listeners
                    .retain(|listener| listener.send(event.clone()).is_ok());
            }
            Err(Error::InvalidAccount) => {
                self.revoke_device(|| Error::InvalidAccount).await;
            }
            Err(Error::InvalidDevice) => {
                self.revoke_device(|| Error::InvalidDevice).await;
            }
            Err(err) => log::error!("Failed to submit voucher: {}", err),
        }
        let _ = tx.send(response);
    }

    async fn consume_expiry_result(&mut self, response: Result<DateTime<Utc>, Error>) {
        match response {
            Ok(expiry) => {
                if expiry > chrono::Utc::now() {
                    log::debug!("Account has time left");
                } else {
                    log::debug!("Account has no time left");
                }

                // Send expiry update event
                let event = AccountEvent::Expiry(expiry);
                self.listeners
                    .retain(|listener| listener.send(event.clone()).is_ok());

                Self::drain_requests(&mut self.expiry_requests, || Ok(expiry));
            }
            Err(Error::InvalidAccount) => {
                self.revoke_device(|| Error::InvalidAccount).await;
            }
            Err(Error::InvalidDevice) => {
                self.revoke_device(|| Error::InvalidDevice).await;
            }
            Err(err) => {
                log::error!("Failed to check account expiry: {}", err);
                let cloneable_err = Arc::new(err);
                Self::drain_requests(&mut self.expiry_requests, || {
                    Err(Error::ResponseFailure(cloneable_err.clone()))
                });
            }
        }
    }

    async fn consume_validation(
        &mut self,
        response: Result<Device, Error>,
        api_call: &mut api::CurrentApiCall,
    ) {
        let current_config = self
            .data
            .device()
            .expect("Received a validation response whilst having no device data");

        match response {
            Ok(new_device) => {
                let current_pubkey = current_config.device.wg_data.private_key.public_key();
                if new_device.pubkey == current_pubkey {
                    let mut new_data = current_config.clone();
                    new_data
                        .device
                        .update(new_device)
                        .expect("pubkey must match privkey");

                    if Some(&new_data) != self.data.device() {
                        log::debug!("Updating data for the current device");
                    } else {
                        log::debug!("The current device is still valid");
                    }

                    match self.set(PrivateDeviceEvent::Updated(new_data)).await {
                        Ok(_) => {
                            Self::drain_requests(&mut self.validation_requests, || Ok(()));
                        }
                        Err(err) => {
                            log::error!("Failed to save device data to disk");
                            let cloneable_err = Arc::new(err);
                            Self::drain_requests(&mut self.validation_requests, || {
                                Err(Error::ResponseFailure(cloneable_err.clone()))
                            });
                        }
                    }
                } else {
                    log::debug!("Rotating invalid WireGuard key for device");
                }
            }
            Err(Error::InvalidAccount) => {
                self.revoke_device(|| Error::InvalidAccount).await;
            }
            Err(Error::InvalidDevice) => {
                self.revoke_device(|| Error::InvalidDevice).await;
            }
            Err(err) => {
                log::error!("Failed to validate device: {}", err);
                let cloneable_err = Arc::new(err);
                Self::drain_requests(&mut self.validation_requests, || {
                    Err(Error::ResponseFailure(cloneable_err.clone()))
                });
            }
        }

        if !self.rotation_requests.is_empty() || !self.validation_requests.is_empty() {
            if let Some(updated_config) = self.data.device() {
                let device_service = self.device_service.clone();
                let token = updated_config.account_token.clone();
                let device_id = updated_config.device.id.clone();
                api_call.set_oneshot_rotation(Box::pin(async move {
                    device_service.rotate_key(token, device_id).await
                }));
            }
        }
    }

    async fn consume_rotation_result(&mut self, api_result: Result<WireguardData, Error>) {
        let mut config = self
            .data
            .device()
            .cloned()
            .expect("Received a key rotation result whilst having no data");

        match api_result {
            Ok(wg_data) => {
                log::debug!("Replacing WireGuard key");
                config.device.wg_data = wg_data;
                match self.set(PrivateDeviceEvent::RotatedKey(config)).await {
                    Ok(_) => {
                        Self::drain_requests(&mut self.rotation_requests, || Ok(()));
                        Self::drain_requests(&mut self.validation_requests, || Ok(()));
                    }
                    Err(err) => {
                        self.drain_device_requests_with_err(err);
                    }
                }
            }
            Err(Error::InvalidAccount) => {
                self.revoke_device(|| Error::InvalidAccount).await;
            }
            Err(Error::InvalidDevice) => {
                self.revoke_device(|| Error::InvalidDevice).await;
            }
            Err(err) => {
                self.drain_device_requests_with_err(err);
            }
        }
    }

    fn drain_device_requests_with_err(&mut self, err: Error) {
        let cloneable_err = Arc::new(err);
        Self::drain_requests(&mut self.rotation_requests, || {
            Err(Error::ResponseFailure(cloneable_err.clone()))
        });
        Self::drain_requests(&mut self.validation_requests, || {
            Err(Error::ResponseFailure(cloneable_err.clone()))
        });
    }

    fn drain_requests<T>(requests: &mut Vec<ResponseTx<T>>, result: impl Fn() -> Result<T, Error>) {
        for req in requests.drain(0..) {
            let _ = req.send(result());
        }
    }

    fn spawn_timed_key_rotation(
        &self,
    ) -> Option<impl Future<Output = Result<WireguardData, Error>> + Send + 'static> {
        let config = self.data.device()?;
        let key_rotation_timer = self.key_rotation_timer(config.device.wg_data.created);

        let device_service = self.device_service.clone();
        let account_token = config.account_token.clone();
        let device_id = config.device.id.clone();

        Some(async move {
            key_rotation_timer.await;
            device_service
                .rotate_key_with_backoff(account_token, device_id)
                .await
        })
    }

    async fn revoke_device(&mut self, err_constructor: impl Fn() -> Error) {
        log::debug!("Invalidating the current device");

        if let Err(err) = self.cacher.write(&PrivateDeviceState::Revoked).await {
            log::error!(
                "{}",
                err.display_chain_with_msg("Failed to save device data to disk")
            );
        }
        self.data.revoke();

        Self::drain_requests(&mut self.validation_requests, || Err(err_constructor()));
        Self::drain_requests(&mut self.rotation_requests, || Err(err_constructor()));
        Self::drain_requests(&mut self.expiry_requests, || Err(err_constructor()));

        self.listeners.retain(|listener| {
            listener
                .send(AccountEvent::Device(PrivateDeviceEvent::Revoked))
                .is_ok()
        });
    }

    async fn logout(&mut self, tx: ResponseTx<()>) {
        Self::drain_requests(&mut self.data_requests, || Err(Error::AccountChange));
        if self.data.logged_out() {
            let _ = tx.send(Ok(()));
            return;
        }
        if let Err(err) = self.cacher.write(&PrivateDeviceState::LoggedOut).await {
            let _ = tx.send(Err(err));
            return;
        }

        let old_config = self.data.logout();

        self.listeners.retain(|listener| {
            listener
                .send(AccountEvent::Device(PrivateDeviceEvent::Logout))
                .is_ok()
        });

        if let Some(old_config) = old_config {
            let logout_call = tokio::spawn(Box::pin(self.logout_api_call(old_config)));

            tokio::spawn(async move {
                let _response = tokio::time::timeout(LOGOUT_TIMEOUT, logout_call).await;
                let _ = tx.send(Ok(()));
            });
        } else {
            // The state was `revoked`.
            let _ = tx.send(Ok(()));
        }
    }

    fn logout_api_call(&self, data: PrivateAccountAndDevice) -> impl Future<Output = ()> + 'static {
        let service = self.device_service.clone();

        async move {
            if let Err(error) = service
                .remove_device_with_backoff(data.account_token, data.device.id)
                .await
            {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to logout device")
                );
            }
        }
    }

    async fn set(&mut self, event: PrivateDeviceEvent) -> Result<(), Error> {
        let device_state = event.clone().state();
        if device_state == self.data {
            return Ok(());
        }

        self.cacher.write(&device_state).await?;
        self.last_validation = None;

        if let Some(old_config) = self.data.logout() {
            if device_state.device().map(|d| &d.device.id) != Some(&old_config.device.id) {
                tokio::spawn(self.logout_api_call(old_config));
            }
        }

        self.data = device_state;

        let event = AccountEvent::Device(event);
        self.listeners
            .retain(|listener| listener.send(event.clone()).is_ok());

        Ok(())
    }

    fn initiate_key_rotation(
        &self,
    ) -> Result<impl Future<Output = Result<WireguardData, Error>>, Error> {
        let data = self.data.device().cloned().ok_or(Error::NoDevice)?;
        let device_service = self.device_service.clone();
        Ok(async move {
            device_service
                .rotate_key(data.account_token, data.device.id)
                .await
        })
    }

    fn key_rotation_timer(&self, key_created: DateTime<Utc>) -> impl Future<Output = ()> + 'static {
        let rotation_interval = self.rotation_interval;

        async move {
            let key_age = Duration::from_secs(
                chrono::Utc::now()
                        .signed_duration_since(key_created)
                        .num_seconds()
                        .try_into()
                        // This would only fail if the key was created in the future, in which case
                        // the duration would be negative. In this case, I think it's safe to
                        // assume the daemon should wait one whole key rotation interval.
                        .unwrap_or(0u64),
            );
            let time_until_next_rotation = std::cmp::max(
                rotation_interval.as_duration().saturating_sub(key_age),
                Duration::from_secs(60),
            );

            log::trace!(
                "{} seconds to wait until next rotation",
                time_until_next_rotation.as_secs(),
            );
            talpid_time::sleep(time_until_next_rotation).await
        }
    }

    fn fetch_device_config(
        &self,
        old_config: &PrivateAccountAndDevice,
    ) -> impl Future<Output = Result<Device, Error>> {
        let device_service = self.device_service.clone();
        let account_token = old_config.account_token.clone();
        let device_id = old_config.device.id.clone();
        async move { device_service.get(account_token, device_id).await }
    }

    fn validation_call(&self) -> Result<impl Future<Output = Result<Device, Error>>, Error> {
        let old_config = self.data.device().ok_or(Error::NoDevice)?;
        Ok(self.fetch_device_config(old_config))
    }

    fn expiry_call(&self) -> Result<impl Future<Output = Result<DateTime<Utc>, Error>>, Error> {
        let old_config = self.data.device().ok_or(Error::NoDevice)?;
        let account_token = old_config.account_token.clone();
        let account_service = self.account_service.clone();
        Ok(async move { account_service.check_expiry_2(account_token).await })
    }

    fn needs_validation(&mut self) -> bool {
        if !self.data.logged_in() {
            return true;
        }

        let now = SystemTime::now();

        let elapsed = self
            .last_validation
            .and_then(|last_check| now.duration_since(last_check).ok())
            .unwrap_or(VALIDITY_CACHE_TIMEOUT);

        if elapsed >= VALIDITY_CACHE_TIMEOUT {
            self.last_validation = Some(now);
            return true;
        }

        false
    }

    async fn shutdown(self) {
        self.cacher.finalize().await;
    }
}
pub struct DeviceCacher {
    file: io::BufWriter<fs::File>,
    path: std::path::PathBuf,
}

impl DeviceCacher {
    pub async fn new(settings_dir: &Path) -> Result<(DeviceCacher, PrivateDeviceState), Error> {
        let path = settings_dir.join(DEVICE_CACHE_FILENAME);
        let cache_exists = path.is_file();
        let mut should_save = false;

        let mut file = fs::OpenOptions::from(Self::file_options())
            .write(true)
            .read(true)
            .create(true)
            .open(&path)
            .await?;

        let device: PrivateDeviceState = if cache_exists {
            let mut reader = io::BufReader::new(&mut file);
            let mut buffer = String::new();
            reader.read_to_string(&mut buffer).await?;
            if !buffer.is_empty() {
                serde_json::from_str(&buffer).unwrap_or_else(|error| {
                    should_save = true;
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Wiping device config due to an error")
                    );
                    PrivateDeviceState::LoggedOut
                })
            } else {
                should_save = true;
                PrivateDeviceState::LoggedOut
            }
        } else {
            should_save = true;
            PrivateDeviceState::LoggedOut
        };

        let mut store = DeviceCacher {
            file: io::BufWriter::new(file),
            path,
        };

        if should_save {
            store.write(&device).await?;
        }

        Ok((store, device))
    }

    fn file_options() -> std::fs::OpenOptions {
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
        options
    }

    pub async fn write(&mut self, device: &PrivateDeviceState) -> Result<(), Error> {
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

/// Checks if the current device is valid if a WireGuard tunnel cannot be set up
/// after multiple attempts.
pub(crate) struct TunnelStateChangeHandler {
    manager: AccountManagerHandle,
    check_validity: Arc<AtomicBool>,
    wg_retry_attempt: usize,
}

impl TunnelStateChangeHandler {
    pub fn new(manager: AccountManagerHandle) -> Self {
        Self {
            manager,
            check_validity: Arc::new(AtomicBool::new(true)),
            wg_retry_attempt: 0,
        }
    }

    pub fn handle_state_transition(&mut self, new_state: &TunnelStateTransition) {
        match new_state {
            TunnelStateTransition::Connecting(endpoint) => {
                if endpoint.tunnel_type != TunnelType::Wireguard {
                    return;
                }
                self.wg_retry_attempt = self.wg_retry_attempt.wrapping_add(1);
                if self.wg_retry_attempt % WG_DEVICE_CHECK_THRESHOLD == 0 {
                    let handle = self.manager.clone();
                    let check_validity = self.check_validity.clone();
                    tokio::spawn(async move {
                        if !check_validity.swap(false, Ordering::SeqCst) {
                            return;
                        }
                        if let Err(error) = Self::check_validity(handle).await {
                            log::error!(
                                "{}",
                                error.display_chain_with_msg(
                                    "Failed to check device or account validity"
                                )
                            );
                            if error.is_network_error() || error.is_aborted() {
                                check_validity.store(true, Ordering::SeqCst);
                            }
                        }
                    });
                }
            }
            TunnelStateTransition::Error(_)
            | TunnelStateTransition::Connected(_)
            | TunnelStateTransition::Disconnected => {
                self.check_validity.store(true, Ordering::SeqCst);
                self.wg_retry_attempt = 0;
            }
            _ => (),
        }
    }

    pub async fn check_validity(handle: AccountManagerHandle) -> Result<(), Error> {
        handle.validate_device().await?;
        handle.check_expiry().await.map(|_expiry| ())
    }
}
