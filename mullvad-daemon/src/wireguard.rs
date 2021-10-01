use crate::{DaemonEventSender, InternalDaemonEvent};
use chrono::offset::Utc;
use mullvad_rpc::{
    availability::ApiAvailabilityHandle,
    rest::{Error as RestError, MullvadRestHandle},
};
use mullvad_types::account::AccountToken;
pub use mullvad_types::wireguard::*;
use std::{future::Future, pin::Pin, time::Duration};

use futures::future::{abortable, AbortHandle};
#[cfg(not(target_os = "android"))]
use talpid_core::future_retry::constant_interval;
use talpid_core::{
    future_retry::{retry_future, retry_future_n, ExponentialBackoff, Jittered},
    mpsc::Sender,
};

pub use talpid_types::net::wireguard::{
    ConnectionConfig, PrivateKey, TunnelConfig, TunnelParameters,
};
use talpid_types::ErrorExt;

/// How long to wait before starting key rotation
const ROTATION_START_DELAY: Duration = Duration::from_secs(60 * 3);

/// How often to check whether the key has expired.
/// A short interval is used in case the computer is ever suspended.
const KEY_CHECK_INTERVAL: Duration = Duration::from_secs(60);

const RETRY_INTERVAL_INITIAL: Duration = Duration::from_secs(4);
const RETRY_INTERVAL_FACTOR: u32 = 5;
const RETRY_INTERVAL_MAX: Duration = Duration::from_secs(24 * 60 * 60);

#[cfg(not(target_os = "android"))]
const SHORT_RETRY_INTERVAL: Duration = Duration::ZERO;

const MAX_KEY_REMOVAL_RETRIES: usize = 2;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Unexpected HTTP request error")]
    RestError(#[error(source)] mullvad_rpc::rest::Error),
    #[error(display = "API availability check was interrupted")]
    ApiCheckError(#[error(source)] mullvad_rpc::availability::Error),
    #[error(display = "Account already has maximum number of keys")]
    TooManyKeys,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct KeyManager {
    daemon_tx: DaemonEventSender,
    availability_handle: ApiAvailabilityHandle,
    http_handle: MullvadRestHandle,
    current_job: Option<AbortHandle>,

    abort_scheduler_tx: Option<AbortHandle>,
    auto_rotation_interval: RotationInterval,
}

impl KeyManager {
    pub(crate) fn new(
        daemon_tx: DaemonEventSender,
        availability_handle: ApiAvailabilityHandle,
        http_handle: MullvadRestHandle,
    ) -> Self {
        Self {
            daemon_tx,
            availability_handle,
            http_handle,
            current_job: None,
            abort_scheduler_tx: None,
            auto_rotation_interval: RotationInterval::default(),
        }
    }

    /// Reset key rotation, cancelling the current one and starting a new one for the specified
    /// account
    pub async fn reset_rotation(&mut self, current_key: PublicKey, account_token: AccountToken) {
        self.run_automatic_rotation(account_token, current_key)
            .await
    }

    /// Update automatic key rotation interval
    /// Passing `None` for the interval will cause the default value to be used.
    pub async fn set_rotation_interval(
        &mut self,
        current_key: PublicKey,
        account_token: AccountToken,
        auto_rotation_interval: Option<RotationInterval>,
    ) {
        self.auto_rotation_interval = auto_rotation_interval.unwrap_or_default();
        self.reset_rotation(current_key, account_token).await;
    }

    /// Stop current key generation
    pub fn reset(&mut self) {
        if let Some(job) = self.current_job.take() {
            job.abort()
        }
    }

    /// Generate a new private key
    pub async fn generate_key_sync(&mut self, account: AccountToken) -> Result<WireguardData> {
        self.reset();
        let private_key = PrivateKey::new_from_random();

        self.push_future_generator(account, private_key, None)()
            .await
            .map_err(Self::map_rpc_error)
    }


    /// Replace a key for an account synchronously
    pub async fn replace_key(
        &mut self,
        account: AccountToken,
        old_key: PublicKey,
    ) -> Result<WireguardData> {
        self.reset();

        let new_key = PrivateKey::new_from_random();
        Self::replace_key_rpc(self.http_handle.clone(), account, old_key, new_key).await
    }

    /// Verifies whether a key is valid or not.
    pub fn verify_wireguard_key(
        &self,
        account: AccountToken,
        key: talpid_types::net::wireguard::PublicKey,
    ) -> impl Future<Output = Result<bool>> {
        let mut rpc = mullvad_rpc::WireguardKeyProxy::new(self.http_handle.clone());
        async move {
            match rpc.get_wireguard_key(account, &key).await {
                Ok(_) => Ok(true),
                Err(mullvad_rpc::rest::Error::ApiError(status, _code))
                    if status == mullvad_rpc::StatusCode::NOT_FOUND =>
                {
                    Ok(false)
                }
                Err(err) => Err(Self::map_rpc_error(err)),
            }
        }
    }

    /// Removes a key from an account
    #[cfg(not(target_os = "android"))]
    pub fn remove_key(
        &self,
        account: AccountToken,
        key: talpid_types::net::wireguard::PublicKey,
    ) -> impl Future<Output = Result<()>> {
        self.remove_key_inner(account, key, constant_interval(SHORT_RETRY_INTERVAL), false)
    }

    /// Removes a key from an account
    pub fn remove_key_with_backoff(
        &self,
        account: AccountToken,
        key: talpid_types::net::wireguard::PublicKey,
    ) -> impl Future<Output = Result<()>> {
        let retry_strategy = Jittered::jitter(
            ExponentialBackoff::new(RETRY_INTERVAL_INITIAL, RETRY_INTERVAL_FACTOR)
                .max_delay(RETRY_INTERVAL_MAX),
        );
        self.remove_key_inner(account, key, retry_strategy, true)
    }

    fn remove_key_inner<D: Iterator<Item = Duration> + 'static>(
        &self,
        account: AccountToken,
        key: talpid_types::net::wireguard::PublicKey,
        retry_strategy: D,
        offline_check: bool,
    ) -> impl Future<Output = Result<()>> {
        let mut rpc = mullvad_rpc::WireguardKeyProxy::new(self.http_handle.clone());
        let api_handle = self.availability_handle.clone();
        let api_handle_2 = api_handle.clone();
        let future = retry_future_n(
            move || {
                let remove_key = rpc.remove_wireguard_key(account.clone(), key.clone());
                let wait_future = api_handle.wait_online();
                async move {
                    if offline_check {
                        let _ = wait_future.await;
                    }
                    remove_key.await
                }
            },
            move |result| match result {
                Ok(_) => false,
                Err(error) => Self::should_retry_removal(error, &api_handle_2),
            },
            retry_strategy,
            MAX_KEY_REMOVAL_RETRIES,
        );
        async move { future.await.map_err(Self::map_rpc_error) }
    }

    fn should_retry_removal(error: &RestError, api_handle: &ApiAvailabilityHandle) -> bool {
        error.is_network_error() && !api_handle.get_state().is_offline()
    }

    fn should_retry(error: &RestError) -> bool {
        if let RestError::ApiError(_status, code) = &error {
            code != mullvad_rpc::INVALID_ACCOUNT && code != mullvad_rpc::KEY_LIMIT_REACHED
        } else {
            true
        }
    }


    /// Generate a new private key asynchronously. The new keys will be sent to the daemon channel.
    pub async fn spawn_key_generation_task(
        &mut self,
        account: AccountToken,
        timeout: Option<Duration>,
    ) {
        self.reset();
        let private_key = PrivateKey::new_from_random();

        let error_tx = self.daemon_tx.clone();
        let error_account = account.clone();

        let mut inner_future_generator =
            self.push_future_generator(account.clone(), private_key, timeout);

        let availability_handle = self.availability_handle.clone();

        let future_generator = move || {
            let wait_available = availability_handle.wait_available();
            let fut = inner_future_generator();
            let error_tx = error_tx.clone();
            let error_account = error_account.clone();
            async move {
                let error_account_copy = error_account.clone();
                wait_available.await.map_err(|error| {
                    let _ = error_tx.send(InternalDaemonEvent::WgKeyEvent((
                        error_account_copy,
                        Err(Error::ApiCheckError(error)),
                    )));
                    false
                })?;
                let response = fut.await;
                match response {
                    Ok(addresses) => Ok(addresses),
                    Err(err) => {
                        let should_retry = Self::should_retry(&err);
                        let _ = error_tx.send(InternalDaemonEvent::WgKeyEvent((
                            error_account,
                            Err(Self::map_rpc_error(err)),
                        )));
                        Err(should_retry)
                    }
                }
            }
        };

        let retry_strategy = Jittered::jitter(
            ExponentialBackoff::new(RETRY_INTERVAL_INITIAL, RETRY_INTERVAL_FACTOR)
                .max_delay(RETRY_INTERVAL_MAX),
        );

        let should_retry = move |result: &std::result::Result<_, bool>| -> bool {
            match result {
                Ok(_) => false,
                Err(should_retry) => *should_retry,
            }
        };

        let upload_future = retry_future(future_generator, should_retry, retry_strategy);


        let (cancellable_upload, abort_handle) = abortable(Box::pin(upload_future));
        let daemon_tx = self.daemon_tx.clone();
        let future = async move {
            match cancellable_upload.await {
                Ok(Ok(wireguard_data)) => {
                    let _ = daemon_tx.send(InternalDaemonEvent::WgKeyEvent((
                        account,
                        Ok(wireguard_data),
                    )));
                }
                Ok(Err(_)) => {}
                Err(_) => {
                    log::error!("Key generation cancelled");
                }
            }
        };


        tokio::spawn(Box::pin(future));
        self.current_job = Some(abort_handle);
    }


    fn push_future_generator(
        &self,
        account: AccountToken,
        private_key: PrivateKey,
        timeout: Option<Duration>,
    ) -> Box<
        dyn FnMut() -> Pin<
                Box<dyn Future<Output = std::result::Result<WireguardData, RestError>> + Send>,
            > + Send,
    > {
        let mut rpc = mullvad_rpc::WireguardKeyProxy::new(self.http_handle.clone());
        let public_key = private_key.public_key();

        let push_future =
            move || -> std::pin::Pin<Box<dyn Future<Output = std::result::Result<WireguardData,  RestError>> + Send >> {
                let key = private_key.clone();
                let address_future = rpc
                    .push_wg_key(account.clone(), public_key.clone(), timeout);
                Box::pin(async move {
                    let addresses = address_future.await?;
                    Ok(WireguardData {
                        private_key: key,
                        addresses,
                        created: Utc::now(),
                    })
                })
            };
        Box::new(push_future)
    }

    async fn replace_key_rpc(
        http_handle: MullvadRestHandle,
        account: AccountToken,
        old_key: PublicKey,
        new_key: PrivateKey,
    ) -> Result<WireguardData> {
        let mut rpc = mullvad_rpc::WireguardKeyProxy::new(http_handle);
        let new_public_key = new_key.public_key();
        let addresses = rpc
            .replace_wg_key(account, old_key.key, new_public_key)
            .await
            .map_err(Self::map_rpc_error)?;
        Ok(WireguardData {
            private_key: new_key,
            addresses,
            created: Utc::now(),
        })
    }

    fn map_rpc_error(err: mullvad_rpc::rest::Error) -> Error {
        match &err {
            // TODO: Consider handling the invalid account case too.
            mullvad_rpc::rest::Error::ApiError(status, message)
                if *status == mullvad_rpc::StatusCode::BAD_REQUEST
                    && message == mullvad_rpc::KEY_LIMIT_REACHED =>
            {
                Error::TooManyKeys
            }
            _ => Error::RestError(err),
        }
    }

    async fn wait_for_key_expiry(key: &PublicKey, rotation_interval_secs: u64) {
        let mut interval = tokio::time::interval(KEY_CHECK_INTERVAL);
        loop {
            interval.tick().await;
            if (Utc::now().signed_duration_since(key.created)).num_seconds() as u64
                >= rotation_interval_secs
            {
                return;
            }
        }
    }

    async fn create_automatic_rotation(
        daemon_tx: DaemonEventSender,
        availability_handle: ApiAvailabilityHandle,
        http_handle: MullvadRestHandle,
        mut public_key: PublicKey,
        rotation_interval_secs: u64,
        account_token: AccountToken,
    ) {
        tokio::time::sleep(ROTATION_START_DELAY).await;

        let rotate_key_for_account =
            move |old_key: &PublicKey| -> Pin<Box<dyn Future<Output = Result<PublicKey>> + Send>> {
                let wait_available = availability_handle.wait_available();
                let rotate = Self::rotate_key(
                    daemon_tx.clone(),
                    http_handle.clone(),
                    account_token.clone(),
                    old_key.clone(),
                );
                Box::pin(async move {
                    wait_available.await?;
                    rotate.await
                })
            };

        loop {
            Self::wait_for_key_expiry(&public_key, rotation_interval_secs).await;

            let rotate_key_for_account_copy = rotate_key_for_account.clone();
            match Self::rotate_key_with_retries(public_key.clone(), rotate_key_for_account_copy)
                .await
            {
                Ok(new_key) => public_key = new_key,
                Err(error) => {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg(
                            "Stopping automatic key rotation due to an error"
                        )
                    );
                    return;
                }
            }
        }
    }

    fn rotate_key(
        daemon_tx: DaemonEventSender,
        http_handle: MullvadRestHandle,
        account_token: AccountToken,
        old_key: PublicKey,
    ) -> impl Future<Output = Result<PublicKey>> {
        let new_key = PrivateKey::new_from_random();
        let rpc_result =
            Self::replace_key_rpc(http_handle, account_token.clone(), old_key, new_key);

        async move {
            match rpc_result.await {
                Ok(data) => {
                    // Update account data
                    let _ = daemon_tx.send(InternalDaemonEvent::WgKeyEvent((
                        account_token,
                        Ok(data.clone()),
                    )));
                    Ok(data.get_public_key())
                }
                Err(Error::TooManyKeys) => {
                    let _ = daemon_tx.send(InternalDaemonEvent::WgKeyEvent((
                        account_token,
                        Err(Error::TooManyKeys),
                    )));
                    Err(Error::TooManyKeys)
                }
                Err(unknown) => Err(unknown),
            }
        }
    }

    async fn rotate_key_with_retries<F>(old_key: PublicKey, rotate_key: F) -> Result<PublicKey>
    where
        F: FnMut(&PublicKey) -> std::pin::Pin<Box<dyn Future<Output = Result<PublicKey>> + Send>>
            + Clone
            + 'static,
    {
        let retry_strategy = Jittered::jitter(
            ExponentialBackoff::new(RETRY_INTERVAL_INITIAL, RETRY_INTERVAL_FACTOR)
                .max_delay(RETRY_INTERVAL_MAX),
        );
        let should_retry = move |result: &Result<PublicKey>| -> bool {
            match result {
                Ok(_) => false,
                Err(error) => match error {
                    Error::RestError(error) => Self::should_retry(error),
                    _ => false,
                },
            }
        };

        retry_future(
            move || rotate_key.clone()(&old_key),
            should_retry,
            retry_strategy,
        )
        .await
    }

    async fn run_automatic_rotation(&mut self, account_token: AccountToken, public_key: PublicKey) {
        self.stop_automatic_rotation();

        log::debug!("Starting automatic key rotation job");
        // Schedule cancellable series of repeating rotation tasks
        let fut = Self::create_automatic_rotation(
            self.daemon_tx.clone(),
            self.availability_handle.clone(),
            self.http_handle.clone(),
            public_key,
            self.auto_rotation_interval.as_duration().as_secs(),
            account_token,
        );
        let (request, abort_handle) = abortable(Box::pin(fut));

        tokio::spawn(request);
        self.abort_scheduler_tx = Some(abort_handle);
    }

    fn stop_automatic_rotation(&mut self) {
        if let Some(abort_handle) = self.abort_scheduler_tx.take() {
            log::info!("Stopping automatic key rotation");
            abort_handle.abort();
        }
    }
}
