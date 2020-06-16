use crate::{account_history::AccountHistory, DaemonEventSender, InternalDaemonEvent};
use chrono::offset::Utc;
use mullvad_rpc::rest::{Error as RestError, MullvadRestHandle};
use mullvad_types::account::AccountToken;
pub use mullvad_types::wireguard::*;
use std::{
    future::Future,
    pin::Pin,
    time::{Duration, Instant},
};

use futures::future::{abortable, AbortHandle};
use talpid_core::{
    future_retry::{retry_future_with_backoff, ExponentialBackoff, Jittered},
    mpsc::Sender,
};

pub use talpid_types::net::wireguard::{
    ConnectionConfig, PrivateKey, TunnelConfig, TunnelParameters,
};
use talpid_types::ErrorExt;
use tokio_timer;

/// Default automatic key rotation
pub const DEFAULT_AUTOMATIC_KEY_ROTATION: Duration = Duration::from_secs(7 * 24 * 60 * 60);
/// How long to wait before reattempting to rotate keys on failure
const AUTOMATIC_ROTATION_RETRY_DELAY: Duration = Duration::from_secs(60 * 15);
/// How often to check whether the key has expired.
/// A short interval is used in case the computer is ever suspended.
const KEY_CHECK_INTERVAL: Duration = Duration::from_secs(60);

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Unexpected HTTP request error")]
    RestError(#[error(source)] mullvad_rpc::rest::Error),
    #[error(display = "Account already has maximum number of keys")]
    TooManyKeys,
    #[error(display = "Failed to create rotation timer")]
    RotationScheduleError(#[error(source)] tokio_timer::TimerError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct KeyManager {
    daemon_tx: DaemonEventSender,
    http_handle: MullvadRestHandle,
    current_job: Option<AbortHandle>,

    abort_scheduler_tx: Option<AbortHandle>,
    auto_rotation_interval: Duration,
}

impl KeyManager {
    pub(crate) fn new(daemon_tx: DaemonEventSender, http_handle: MullvadRestHandle) -> Self {
        Self {
            daemon_tx,
            http_handle,
            current_job: None,
            abort_scheduler_tx: None,
            auto_rotation_interval: Duration::new(0, 0),
        }
    }

    /// Reset key rotation, cancelling the current one and starting a new one for the specified
    /// account
    pub fn reset_rotation(
        &mut self,
        account_history: &mut AccountHistory,
        account_token: AccountToken,
    ) {
        match account_history
            .get(&account_token)
            .map(|entry| entry.map(|entry| entry.wireguard.map(|wg| wg.get_public_key())))
        {
            Ok(Some(Some(public_key))) => self.run_automatic_rotation(account_token, public_key),
            Ok(Some(None)) => {
                log::error!("reset_rotation: failed to obtain public key for account entry.")
            }
            Ok(None) => log::error!("reset_rotation: account entry not found."),
            Err(e) => log::error!("reset_rotation: failed to obtain account entry. {}", e),
        };
    }

    /// Update automatic key rotation interval
    /// Passing `None` for the interval will use the default value.
    /// A duration of `0` disables automatic key rotation.
    pub fn set_rotation_interval(
        &mut self,
        account_history: &mut AccountHistory,
        account_token: AccountToken,
        auto_rotation_interval: Option<Duration>,
    ) {
        self.auto_rotation_interval =
            auto_rotation_interval.unwrap_or(DEFAULT_AUTOMATIC_KEY_ROTATION);

        self.reset_rotation(account_history, account_token);
    }

    /// Stop current key generation
    pub fn reset(&mut self) {
        if let Some(job) = self.current_job.take() {
            job.abort()
        }
    }

    /// Generate a new private key
    pub fn generate_key_sync(&mut self, account: AccountToken) -> Result<WireguardData> {
        self.reset();
        let private_key = PrivateKey::new_from_random();

        self.http_handle
            .service()
            .block_on(self.push_future_generator(account, private_key, None)())
            .map_err(Self::map_rpc_error)
    }


    /// Replace a key for an account synchronously
    pub fn replace_key(
        &mut self,
        account: AccountToken,
        old_key: PublicKey,
    ) -> Result<WireguardData> {
        self.reset();

        let new_key = PrivateKey::new_from_random();
        self.http_handle.service().block_on(Self::replace_key_rpc(
            self.http_handle.clone(),
            account,
            old_key,
            new_key,
        ))
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


    /// Generate a new private key asynchronously. The new keys will be sent to the daemon channel.
    pub fn generate_key_async(&mut self, account: AccountToken, timeout: Option<Duration>) {
        self.reset();
        let private_key = PrivateKey::new_from_random();

        let error_tx = self.daemon_tx.clone();
        let error_account = account.clone();

        let mut inner_future_generator =
            self.push_future_generator(account.clone(), private_key, timeout);

        let future_generator = move || {
            let fut = inner_future_generator();
            let error_tx = error_tx.clone();
            let error_account = error_account.clone();
            async move {
                let response = fut.await;
                match response {
                    Ok(addresses) => Ok(addresses),
                    Err(err) => {
                        let should_retry = if let RestError::ApiError(_status, code) = &err {
                            code != mullvad_rpc::KEY_LIMIT_REACHED
                        } else {
                            true
                        };
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
            ExponentialBackoff::from_millis(300).max_delay(Duration::from_secs(60 * 60)),
        );

        let should_retry = move |result: &std::result::Result<_, bool>| -> bool {
            match result {
                Ok(_) => false,
                Err(should_retry) => *should_retry,
            }
        };

        let upload_future =
            retry_future_with_backoff(future_generator, should_retry, retry_strategy);


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


        self.http_handle.service().spawn(Box::pin(future));
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

    async fn key_rotation_timer(key: PublicKey, rotation_interval_secs: u64) {
        let mut interval = tokio02::time::interval(KEY_CHECK_INTERVAL);
        loop {
            interval.tick().await;
            if (Utc::now().signed_duration_since(key.created)).num_seconds() as u64
                >= rotation_interval_secs
            {
                return;
            }
        }
    }

    async fn next_automatic_rotation(
        daemon_tx: DaemonEventSender,
        http_handle: MullvadRestHandle,
        public_key: PublicKey,
        rotation_interval_secs: u64,
        account_token: AccountToken,
    ) -> Result<PublicKey> {
        let account_token_copy = account_token.clone();
        Self::key_rotation_timer(public_key.clone(), rotation_interval_secs).await;

        let private_key = PrivateKey::new_from_random();
        let rpc_result =
            Self::replace_key_rpc(http_handle, account_token, public_key, private_key).await;
        match rpc_result {
            Ok(data) => {
                // Update account data
                let _ = daemon_tx.send(InternalDaemonEvent::WgKeyEvent((
                    account_token_copy,
                    Ok(data.clone()),
                )));
                Ok(data.get_public_key())
            }
            Err(Error::TooManyKeys) => {
                let _ = daemon_tx.send(InternalDaemonEvent::WgKeyEvent((
                    account_token_copy,
                    Err(Error::TooManyKeys),
                )));
                Err(Error::TooManyKeys)
            }
            Err(unknown_err) => Err(unknown_err),
        }
    }

    async fn create_automatic_rotation(
        daemon_tx: DaemonEventSender,
        http_handle: MullvadRestHandle,
        mut public_key: PublicKey,
        rotation_interval_secs: u64,
        account_token: AccountToken,
    ) {
        let mut interval = tokio02::time::interval_at(
            (Instant::now() + AUTOMATIC_ROTATION_RETRY_DELAY).into(),
            AUTOMATIC_ROTATION_RETRY_DELAY,
        );

        loop {
            let daemon_tx = daemon_tx.clone();
            interval.tick().await;
            let new_key_result = Self::next_automatic_rotation(
                daemon_tx,
                http_handle.clone(),
                public_key.clone(),
                rotation_interval_secs,
                account_token.clone(),
            )
            .await;
            match new_key_result {
                Ok(new_key) => public_key = new_key,
                Err(Error::TooManyKeys) => {
                    log::error!("Account has too many keys, stopping automatic rotation");
                    return;
                }
                Err(err) => {
                    log::error!(
                        "{}. Retrying in {} seconds",
                        err.display_chain_with_msg("Key rotation failed:"),
                        AUTOMATIC_ROTATION_RETRY_DELAY.as_secs(),
                    );
                }
            }
        }
    }

    fn run_automatic_rotation(&mut self, account_token: AccountToken, public_key: PublicKey) {
        self.stop_automatic_rotation();

        if self.auto_rotation_interval == Duration::new(0, 0) {
            // disabled
            return;
        }

        log::debug!("Starting automatic key rotation job");
        // Schedule cancellable series of repeating rotation tasks
        let fut = Self::create_automatic_rotation(
            self.daemon_tx.clone(),
            self.http_handle.clone(),
            public_key,
            self.auto_rotation_interval.as_secs(),
            account_token,
        );
        let (request, abort_handle) = abortable(Box::pin(fut));

        self.http_handle.service().spawn(request);
        self.abort_scheduler_tx = Some(abort_handle);
    }

    fn stop_automatic_rotation(&mut self) {
        if let Some(abort_handle) = self.abort_scheduler_tx.take() {
            log::info!("Stopping automatic key rotation");
            abort_handle.abort();
        }
    }
}
