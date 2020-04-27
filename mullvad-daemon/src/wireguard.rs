use crate::{account_history::AccountHistory, DaemonEventSender, InternalDaemonEvent};
use chrono::offset::Utc;
use futures::{future::Executor, stream::Stream, sync::oneshot, Async, Future, Poll};
use mullvad_rpc::rest::{Error as RestError, MullvadRestHandle};
use mullvad_types::account::AccountToken;
pub use mullvad_types::wireguard::*;
use std::time::Duration;
use talpid_core::mpsc::Sender;
pub use talpid_types::net::wireguard::{
    ConnectionConfig, PrivateKey, TunnelConfig, TunnelParameters,
};
use talpid_types::ErrorExt;
use tokio_core::reactor::Remote;
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    RetryIf,
};
use tokio_timer;

/// Default automatic key rotation
const DEFAULT_AUTOMATIC_KEY_ROTATION: Duration = Duration::from_secs(7 * 24 * 60 * 60);
/// How long to wait before reattempting to rotate keys on failure
const AUTOMATIC_ROTATION_RETRY_DELAY: Duration = Duration::from_secs(5);
/// How often to check whether the key has expired.
/// A short interval is used in case the computer is ever suspended.
const KEY_CHECK_INTERVAL: Duration = Duration::from_secs(60);

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to spawn future")]
    ExectuionError,
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
    tokio_remote: Remote,
    current_job: Option<CancelHandle>,

    abort_scheduler_tx: Option<CancelHandle>,
    auto_rotation_interval: Duration,
}

impl KeyManager {
    pub(crate) fn new(
        daemon_tx: DaemonEventSender,
        http_handle: MullvadRestHandle,
        tokio_remote: Remote,
    ) -> Self {
        Self {
            daemon_tx,
            http_handle,
            tokio_remote,
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
            job.cancel()
        }
    }

    /// Generate a new private key
    pub fn generate_key_sync(&mut self, account: AccountToken) -> Result<WireguardData> {
        self.reset();
        let private_key = PrivateKey::new_from_random();

        self.run_future_sync(self.push_future_generator(account, private_key)())
            .map_err(Self::map_rpc_error)
    }

    /// Run a future on the given tokio remote
    fn run_future_sync<T: Send + 'static, E: Send + 'static>(
        &mut self,
        fut: impl Future<Item = T, Error = E> + Send + 'static,
    ) -> std::result::Result<T, E> {
        self.reset();
        let (tx, rx) = oneshot::channel();

        let _ = self.tokio_remote.execute(fut.then(|result| {
            let _ = tx.send(result);
            Ok(())
        }));
        rx.wait().unwrap()
    }

    /// Replace a key for an account synchronously
    pub fn replace_key(
        &mut self,
        account: AccountToken,
        old_key: PublicKey,
    ) -> Result<WireguardData> {
        self.reset();
        let new_key = PrivateKey::new_from_random();
        self.run_future_sync(Self::replace_key_rpc(
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
    ) -> impl Future<Item = bool, Error = Error> {
        let mut rpc = mullvad_rpc::WireguardKeyProxy::new(self.http_handle.clone());
        rpc.get_wireguard_key(account, &key)
            .then(|response| match response {
                Ok(_) => Ok(true),
                Err(mullvad_rpc::rest::Error::ApiError(status, _code))
                    if status == mullvad_rpc::StatusCode::NOT_FOUND =>
                {
                    Ok(false)
                }
                Err(err) => Err(Self::map_rpc_error(err)),
            })
    }


    /// Generate a new private key asynchronously. The new keys will be sent to the daemon channel.
    pub fn generate_key_async(&mut self, account: AccountToken) -> Result<()> {
        self.reset();
        let private_key = PrivateKey::new_from_random();
        let future_generator = self.push_future_generator(account.clone(), private_key);

        let retry_strategy = ExponentialBackoff::from_millis(300)
            .max_delay(Duration::from_secs(60 * 60))
            .map(jitter);

        let should_retry = |err: &RestError| -> bool {
            match err {
                RestError::ApiError(_status, code) if code == mullvad_rpc::KEY_LIMIT_REACHED => {
                    false
                }
                _ => true,
            }
        };

        let upload_future =
            RetryIf::spawn(retry_strategy, future_generator, should_retry).map_err(move |err| {
                match err {
                    // This should really be unreachable, since the retry strategy is infinite.
                    tokio_retry::Error::OperationError(e) => {
                        log::error!(
                            "{}",
                            e.display_chain_with_msg("Failed to generate wireguard key:")
                        );
                        Self::map_rpc_error(e)
                    }
                    tokio_retry::Error::TimerError(timer_error) => {
                        log::error!("Tokio timer error {}", timer_error);
                        Error::ExectuionError
                    }
                }
            });


        let (fut, cancel_handle) = Cancellable::new(upload_future);
        let daemon_tx = self.daemon_tx.clone();
        let fut = fut.then(move |result| {
            match result {
                Ok(wireguard_data) => {
                    let _ = daemon_tx.send(InternalDaemonEvent::WgKeyEvent((
                        account,
                        Ok(wireguard_data),
                    )));
                }
                Err(CancelErr::Inner(e)) => {
                    let _ = daemon_tx.send(InternalDaemonEvent::WgKeyEvent((account, Err(e))));
                }
                Err(CancelErr::Cancelled) => {
                    log::error!("Key generation cancelled");
                }
            };
            Ok(())
        });

        let result = self
            .tokio_remote
            .execute(fut)
            .map_err(|_| Error::ExectuionError);
        if result.is_ok() {
            self.current_job = Some(cancel_handle);
        }
        result
    }


    fn push_future_generator(
        &self,
        account: AccountToken,
        private_key: PrivateKey,
    ) -> Box<dyn FnMut() -> Box<dyn Future<Item = WireguardData, Error = RestError> + Send> + Send>
    {
        let mut rpc = mullvad_rpc::WireguardKeyProxy::new(self.http_handle.clone());
        let public_key = private_key.public_key();

        let push_future =
            move || -> Box<dyn Future<Item = WireguardData, Error = RestError> + Send> {
                let key = private_key.clone();
                Box::new(rpc.push_wg_key(account.clone(), public_key.clone()).map(
                    move |addresses| WireguardData {
                        private_key: key,
                        addresses,
                        created: Utc::now(),
                    },
                ))
            };
        Box::new(push_future)
    }

    fn replace_key_rpc(
        http_handle: MullvadRestHandle,
        account: AccountToken,
        old_key: PublicKey,
        new_key: PrivateKey,
    ) -> impl Future<Item = WireguardData, Error = Error> + Send {
        let mut rpc = mullvad_rpc::WireguardKeyProxy::new(http_handle);
        let new_public_key = new_key.public_key();
        rpc.replace_wg_key(account, old_key.key, new_public_key)
            .map_err(Self::map_rpc_error)
            .map(move |addresses| WireguardData {
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

    fn create_rotation_check(
        key: PublicKey,
        rotation_interval_secs: u64,
    ) -> impl Future<Item = (), Error = Error> + Send {
        tokio_timer::wheel()
            .build()
            .interval(KEY_CHECK_INTERVAL)
            .map_err(Error::RotationScheduleError)
            .take_while(move |_| {
                Ok(
                    (Utc::now().signed_duration_since(key.created)).num_seconds() as u64
                        <= rotation_interval_secs,
                )
            })
            .for_each(|_| Ok(()))
    }

    fn next_automatic_rotation(
        daemon_tx: DaemonEventSender,
        http_handle: MullvadRestHandle,
        public_key: PublicKey,
        rotation_interval_secs: u64,
        account_token: AccountToken,
    ) -> impl Future<Item = PublicKey, Error = Error> + Send {
        let expiration_timer =
            Self::create_rotation_check(public_key.clone(), rotation_interval_secs);
        let account_token_copy = account_token.clone();

        expiration_timer
            .and_then(move |_| {
                log::info!("Replacing WireGuard key");

                let private_key = PrivateKey::new_from_random();
                Self::replace_key_rpc(http_handle, account_token, public_key, private_key)
            })
            .then(move |rpc_result| {
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
            })
    }

    fn create_automatic_rotation(
        daemon_tx: DaemonEventSender,
        http_handle: MullvadRestHandle,
        public_key: PublicKey,
        rotation_interval_secs: u64,
        account_token: AccountToken,
    ) -> impl Future<Item = (), Error = ()> + Send {
        tokio_timer::wheel()
            .build()
            .interval(AUTOMATIC_ROTATION_RETRY_DELAY)
            .map_err(Error::RotationScheduleError)
            .fold(public_key, move |old_public_key, _| {
                let fut = Self::next_automatic_rotation(
                    daemon_tx.clone(),
                    http_handle.clone(),
                    old_public_key.clone(),
                    rotation_interval_secs,
                    account_token.clone(),
                );
                fut.then(|result| match result {
                    Ok(new_public_key) => Ok(new_public_key),
                    Err(Error::TooManyKeys) => {
                        log::error!("Account has too many keys, stopping automatic rotation");
                        Err(Error::TooManyKeys)
                    }
                    Err(e) => {
                        log::error!(
                            "{}. Retrying in {} seconds",
                            e.display_chain_with_msg("Key rotation failed:"),
                            AUTOMATIC_ROTATION_RETRY_DELAY.as_secs(),
                        );
                        Ok(old_public_key)
                    }
                })
            })
            .map_err(|_| ())
            .map(|_| ())
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
        let (fut, cancel_handle) = Cancellable::new(fut);

        if let Err(e) = self.tokio_remote.execute(fut.map_err(|_| ())) {
            log::error!("Failed to execute auto key rotation: {:?}", e.kind());
        }
        self.abort_scheduler_tx = Some(cancel_handle);
    }

    fn stop_automatic_rotation(&mut self) {
        if let Some(cancel_handle) = self.abort_scheduler_tx.take() {
            log::info!("Stopping automatic key rotation");
            cancel_handle.cancel();
        }
    }
}

pub enum CancelErr<E> {
    Cancelled,
    Inner(E),
}

pub struct Cancellable<T, E, F: Future<Item = T, Error = E>> {
    rx: oneshot::Receiver<()>,
    f: F,
}

pub struct CancelHandle {
    tx: oneshot::Sender<()>,
}

impl CancelHandle {
    fn cancel(self) {
        let _ = self.tx.send(());
    }
}


impl<T, E, F> Cancellable<T, E, F>
where
    F: Future<Item = T, Error = E>,
{
    fn new(f: F) -> (Self, CancelHandle) {
        let (tx, rx) = oneshot::channel();
        (Self { f, rx }, CancelHandle { tx })
    }
}

impl<T, E, F> Future for Cancellable<T, E, F>
where
    F: Future<Item = T, Error = E>,
{
    type Item = T;
    type Error = CancelErr<E>;

    fn poll(&mut self) -> Poll<T, CancelErr<E>> {
        match self.rx.poll() {
            Ok(Async::Ready(_)) | Err(_) => return Err(CancelErr::Cancelled),
            Ok(Async::NotReady) => (),
        };

        match self.f.poll() {
            Ok(v) => Ok(v),
            Err(e) => Err(CancelErr::Inner(e)),
        }
    }
}
