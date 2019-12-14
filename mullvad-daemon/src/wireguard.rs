use crate::{account_history::AccountHistory, InternalDaemonEvent};
use chrono::offset::Utc;
use futures::{
    future::{Executor, IntoFuture},
    sync::oneshot,
    Async, Future, Poll,
};
use jsonrpc_client_core::Error as JsonRpcError;
use mullvad_types::account::AccountToken;
pub use mullvad_types::wireguard::*;
use std::{cmp, sync::mpsc, time::Duration};
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

const TOO_MANY_KEYS_ERROR_CODE: i64 = -703;

/// Default automatic key rotation (in minutes)
const DEFAULT_AUTOMATIC_KEY_ROTATION: u32 = 7 * 24 * 60;
/// How long to wait before reattempting to rotate keys on failure (secs)
const AUTOMATIC_ROTATION_RETRY_DELAY: u64 = 5;
/// Minimum interval used by automatic rotation (secs)
const MINIMUM_ROTATION_INTERVAL: u64 = 5;


#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to generate private key")]
    GenerationError(#[error(source)] rand::Error),
    #[error(display = "Failed to spawn future")]
    ExectuionError,
    #[error(display = "Unexpected RPC error")]
    RpcError(#[error(source)] jsonrpc_client_core::Error),
    #[error(display = "Account already has maximum number of keys")]
    TooManyKeys,
    #[error(display = "Failed to create rotation timer")]
    RotationScheduleError(#[error(source)] tokio_timer::TimerError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct KeyManager {
    daemon_tx: mpsc::Sender<InternalDaemonEvent>,
    http_handle: mullvad_rpc::HttpHandle,
    tokio_remote: Remote,
    current_job: Option<CancelHandle>,

    abort_scheduler_tx: Option<CancelHandle>,
    account_token: Option<AccountToken>,
    public_key: Option<PublicKey>,
    // unit: minutes
    auto_rotation_interval: u32,
}

impl KeyManager {
    pub(crate) fn new(
        daemon_tx: mpsc::Sender<InternalDaemonEvent>,
        http_handle: mullvad_rpc::HttpHandle,
        tokio_remote: Remote,
        account_token: Option<AccountToken>,
    ) -> Self {
        Self {
            daemon_tx,
            http_handle,
            tokio_remote,
            current_job: None,
            abort_scheduler_tx: None,

            account_token,
            public_key: None,
            auto_rotation_interval: 0,
        }
    }

    fn update_public_key(&mut self, account_history: &mut AccountHistory) {
        log::debug!("update_public_key");
        let _ = self.public_key.take();

        let token = if let Some(token) = &self.account_token {
            token
        } else {
            log::warn!("Cannot update public key; no account token is set");
            return ();
        };

        self.public_key = match account_history.get(&token) {
            Ok(v) => v
                .map(|entry| entry.wireguard.map(|wg| wg.get_public_key()))
                .unwrap(),
            Err(e) => {
                log::error!("KeyManager failed to obtain public key. {}", e);
                None
            }
        };
    }

    /// Update automatic key rotation interval (given in minutes)
    /// Passing `None` for the interval will use the default value.
    /// A value of `0` disables automatic key rotation.
    pub fn set_rotation_interval(
        &mut self,
        account_history: &mut AccountHistory,
        auto_rotation_interval_mins: Option<u32>,
    ) {
        log::debug!("set_rotation_interval");

        self.auto_rotation_interval =
            auto_rotation_interval_mins.unwrap_or(DEFAULT_AUTOMATIC_KEY_ROTATION);

        self.stop_automatic_rotation();
        self.update_public_key(account_history);
        self.run_automatic_rotation();
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
        let private_key = PrivateKey::new_from_random().map_err(Error::GenerationError)?;

        self.run_future_sync(self.push_future_generator(account, private_key)())
            .map_err(Self::map_rpc_error)
    }

    pub fn run_future_sync<T: Send + 'static, E: Send + 'static>(
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

    pub fn replace_key(
        &mut self,
        account: AccountToken,
        old_key: PublicKey,
    ) -> Result<WireguardData> {
        self.reset();
        let new_key = PrivateKey::new_from_random().map_err(Error::GenerationError)?;
        self.run_future_sync(Self::replace_key_rpc(
            self.http_handle.clone(),
            account,
            old_key,
            new_key,
        ))
        .map_err(Self::map_rpc_error)
    }


    /// Generate a new private key asyncronously. The new keys will be sent to the daemon channel.
    pub fn generate_key_async(&mut self, account: AccountToken) -> Result<()> {
        self.reset();
        let private_key = PrivateKey::new_from_random().map_err(Error::GenerationError)?;
        let future_generator = self.push_future_generator(account.clone(), private_key);

        let retry_strategy = ExponentialBackoff::from_millis(300)
            .max_delay(Duration::from_secs(60 * 60))
            .map(jitter);

        let should_retry = |err: &jsonrpc_client_core::Error| -> bool {
            match err.kind() {
                jsonrpc_client_core::ErrorKind::JsonRpcError(err)
                    if err.code.code() == TOO_MANY_KEYS_ERROR_CODE =>
                {
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

        match self
            .tokio_remote
            .execute(fut)
            .map_err(|_| Error::ExectuionError)
        {
            Ok(res) => {
                self.current_job = Some(cancel_handle);
                Ok(res)
            }
            Err(e) => Err(e),
        }
    }


    fn push_future_generator(
        &self,
        account: AccountToken,
        private_key: PrivateKey,
    ) -> Box<dyn FnMut() -> Box<dyn Future<Item = WireguardData, Error = JsonRpcError> + Send> + Send>
    {
        let mut rpc = mullvad_rpc::WireguardKeyProxy::new(self.http_handle.clone());
        let public_key = private_key.public_key();

        let push_future =
            move || -> Box<dyn Future<Item = WireguardData, Error = JsonRpcError> + Send> {
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
        http_handle: mullvad_rpc::HttpHandle,
        account: AccountToken,
        old_key: PublicKey,
        new_key: PrivateKey,
    ) -> impl Future<Item = WireguardData, Error = JsonRpcError> + Send {
        let mut rpc = mullvad_rpc::WireguardKeyProxy::new(http_handle.clone());
        let new_public_key = new_key.public_key();
        rpc.replace_wg_key(account.clone(), old_key.key, new_public_key)
            .map(move |addresses| WireguardData {
                private_key: new_key,
                addresses,
                created: Utc::now(),
            })
    }

    fn map_rpc_error(err: jsonrpc_client_core::Error) -> Error {
        match err.kind() {
            // TODO: Consider handling the invalid account case too.
            jsonrpc_client_core::ErrorKind::JsonRpcError(err) if err.code.code() == -703 => {
                Error::TooManyKeys
            }
            _ => Error::RpcError(err),
        }
    }

    pub fn set_account_token(
        &mut self,
        account_history: &mut AccountHistory,
        account_token: Option<AccountToken>,
    ) {
        log::debug!("set_account_token");
        self.account_token = account_token;

        self.set_rotation_interval(account_history, Some(self.auto_rotation_interval));
    }

    fn create_key_expiration_timer(
        public_key: PublicKey,
        rotation_interval_secs: u64,
    ) -> impl Future<Item = (), Error = Error> + Send {
        let last_update = public_key.created.clone();
        let key_age = Duration::from_secs(
            (Utc::now().signed_duration_since(last_update)).num_seconds() as u64,
        );

        let interval_duration = Duration::from_secs(rotation_interval_secs);
        let remaining_time = interval_duration
            .checked_sub(key_age)
            .unwrap_or(Duration::from_secs(0));
        let key_expiry = cmp::max(
            Duration::from_secs(MINIMUM_ROTATION_INTERVAL),
            remaining_time,
        );

        log::info!("Next key rotation (time left): {:?}", key_expiry);

        tokio_timer::wheel()
            .max_timeout(Duration::new(std::u64::MAX, 0))
            .build()
            .sleep(key_expiry)
            .map_err(|e| Error::RotationScheduleError(e))
    }

    fn next_automatic_rotation(
        daemon_tx: mpsc::Sender<InternalDaemonEvent>,
        http_handle: mullvad_rpc::HttpHandle,
        public_key: PublicKey,
        rotation_interval_secs: u64,
        account_token: AccountToken,
    ) -> impl Future<Item = WireguardData, Error = Error> + Send {
        let expiration_timer =
            Self::create_key_expiration_timer(public_key.clone(), rotation_interval_secs);

        let account_token_copy = account_token.clone();

        expiration_timer
            .and_then(move |_| {
                log::info!("Replacing WireGuard key");

                let private_key = PrivateKey::new_from_random()
                    .map_err(Error::GenerationError)
                    .into_future();
                private_key.and_then(move |private_key| {
                    Self::replace_key_rpc(
                        http_handle.clone(),
                        account_token.clone(),
                        public_key.clone(),
                        private_key,
                    )
                    .map_err(|err| Error::RpcError(err))
                })
            })
            .map(move |wireguard_data| {
                // Update account data
                let _ = daemon_tx.send(InternalDaemonEvent::WgKeyEvent((
                    account_token_copy,
                    Ok(wireguard_data.clone()),
                )));

                wireguard_data
            })
    }

    fn create_automatic_rotation(
        daemon_tx: mpsc::Sender<InternalDaemonEvent>,
        http_handle: mullvad_rpc::HttpHandle,
        public_key: PublicKey,
        rotation_interval_secs: u64,
        account_token: AccountToken,
    ) -> Box<dyn Future<Item = (), Error = Error> + Send> {
        log::debug!("create_automatic_rotation");

        let fut = Self::next_automatic_rotation(
            daemon_tx.clone(),
            http_handle.clone(),
            public_key.clone(),
            rotation_interval_secs,
            account_token.clone(),
        );

        let create_repeat_future = move |result: Result<WireguardData>| {
            let next_public_key;
            let next_interval: u64;

            match result {
                Ok(wg_data) => {
                    next_interval = rotation_interval_secs;
                    next_public_key = wg_data.get_public_key();

                    Self::create_automatic_rotation(
                        daemon_tx.clone(),
                        http_handle.clone(),
                        next_public_key,
                        next_interval,
                        account_token.clone(),
                    )
                }
                Err(e) => {
                    log::error!(
                        "Key rotation failed: {}. Retrying in {} seconds",
                        e,
                        AUTOMATIC_ROTATION_RETRY_DELAY,
                    );

                    next_interval = rotation_interval_secs;
                    next_public_key = public_key.clone();

                    let daemon_tx = daemon_tx.clone();
                    let http_handle = http_handle.clone();
                    let account_token = account_token.clone();

                    Box::new(
                        tokio_timer::wheel()
                            .build()
                            .sleep(Duration::from_secs(AUTOMATIC_ROTATION_RETRY_DELAY))
                            .then(move |_| {
                                Self::create_automatic_rotation(
                                    daemon_tx,
                                    http_handle,
                                    next_public_key,
                                    next_interval,
                                    account_token,
                                )
                            }),
                    )
                }
            }
        };

        Box::new(fut.then(create_repeat_future).map(|_| ()))
    }

    fn run_automatic_rotation(&mut self) {
        self.stop_automatic_rotation();

        if let 0 = self.auto_rotation_interval {
            // disabled
            return;
        }

        if let None = self.account_token {
            log::warn!(
                "Not running automatic rotation since no \
                 account token is set"
            );
            return;
        }
        let account_token = self.account_token.as_ref().unwrap().to_string();

        if let None = self.public_key {
            log::warn!(
                "Not running automatic rotation since no \
                 public key is set"
            );
            return;
        }
        let public_key = self.public_key.as_ref().unwrap().clone();

        // Schedule cancellable series of repeating rotation tasks
        let fut = Self::create_automatic_rotation(
            self.daemon_tx.clone(),
            self.http_handle.clone(),
            public_key,
            60u64 * (self.auto_rotation_interval as u64),
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
