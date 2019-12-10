use crate::InternalDaemonEvent;
use chrono::{DateTime, offset::Utc};
use futures::{future::Executor, sync::oneshot, Async, Future, Poll};
use jsonrpc_client_core::Error as JsonRpcError;
use mullvad_types::account::AccountToken;
pub use mullvad_types::wireguard::*;
use std::{cmp, sync::mpsc, time::{Duration, Instant}};
pub use talpid_types::net::wireguard::{
    ConnectionConfig, PrivateKey, TunnelConfig, TunnelParameters,
};
use talpid_types::ErrorExt;
use tokio::timer::Delay;
use tokio_core::reactor::Remote;
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    RetryIf,
};

const TOO_MANY_KEYS_ERROR_CODE: i64 = -703;
/// Default automatic key rotation (in hours)
const DEFAULT_AUTOMATIC_KEY_ROTATION: u32 = 7 * 24;
/// How long to wait before reattempting to rotate keys on failure (secs)
const AUTOMATIC_ROTATION_RETRY_DELAY: u64 = 5;


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
    #[error(display = "Failed to create Delay object")]
    Delay,
    #[error(display = "Failed to create key rotation scheduler")]
    CreateAutomaticKeyRotationScheduler,
    #[error(display = "Failed to run automatic key rotation")]
    RunAutomaticKeyRotation,
}

pub type Result<T> = std::result::Result<T, Error>;

use crate::ManagementCommand;
use talpid_core::tunnel_state_machine::TunnelCommand;

use mullvad_types::wireguard;

struct KeyRotationScheduler {
    daemon_tx: mpsc::Sender<InternalDaemonEvent>,
    delay: Box<dyn Future<Item = (), Error = ()> + Send>,
    last_update: Option<DateTime<Utc>>,
    interval: u32,
    key_request_rx: Option<oneshot::Receiver<wireguard::KeygenEvent>>,
}

impl Future for KeyRotationScheduler {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Error> {
        log::debug!("Poll key rotation future");

        if let Some(key_request_rx) = &mut self.key_request_rx {
            let poll_result = key_request_rx.poll();

            match poll_result {
                Ok(Async::Ready(KeygenEvent::NewKey(_))) => {
                    log::debug!("Completed automatic rotation");
                    self.key_request_rx = None;
                    self.last_update = Some(Utc::now());
                    self.delay = Self::next_delay(self.interval, None);
                }
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                _ => {
                    log::error!("Automatic key rotation failed; retrying");
                    self.key_request_rx = None;
                    self.delay = Box::new(Delay::new(
                        Instant::now() + Duration::from_secs(AUTOMATIC_ROTATION_RETRY_DELAY)
                    ).map_err(|_| ()));
                }
            }
        }

        match self.delay.poll() {
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Err(_) => return Err(Error::Delay),
            _ => (),
        }

        let (wg_tx, wg_rx) = oneshot::channel();
        let _ = self.daemon_tx.send(InternalDaemonEvent::ManagementInterfaceEvent(
            ManagementCommand::GenerateWireguardKey(wg_tx)
        ))
        .map_err(|_| Error::RunAutomaticKeyRotation)?;

        log::debug!("Sent key replacement request");

        self.key_request_rx = Some(wg_rx);
        futures::task::current().notify();
        Ok(Async::NotReady)
    }
}

impl KeyRotationScheduler {
    pub(crate) fn new(
        tokio_remote: Remote,
        daemon_tx: mpsc::Sender<InternalDaemonEvent>,
        public_key: Option<PublicKey>,
        interval: Option<u32>,
    ) -> Result<oneshot::Sender<()>> {
        let (
            terminate_auto_rotation_tx,
            terminate_auto_rotation_rx
        ) = oneshot::channel();

        let interval = interval.unwrap_or(DEFAULT_AUTOMATIC_KEY_ROTATION);
        let last_update = public_key.map(|key| key.created.clone());

        let fut = Self {
            daemon_tx: daemon_tx.clone(),
            delay: Self::next_delay(interval, last_update),
            last_update,
            interval,
            key_request_rx: None,
        };

        tokio_remote.execute(
            fut.map_err(|e| {
                log::error!("Failed to run key rotation scheduler: {}", e)
            })
            .select(terminate_auto_rotation_rx.map_err(|_| ()))
            .map_err(|_| ())
            .map(|_| ())
        ).map_err(|e| {
            log::error!("Failed to run key rotation scheduler: {:?}", e);
            Error::CreateAutomaticKeyRotationScheduler
        })?;

        Ok(terminate_auto_rotation_tx)
    }

    fn next_delay(interval_mins: u32, last_update: Option<DateTime<Utc>>) ->
        Box<dyn Future<Item = (), Error = ()> + Send>
    {
        let mut delay = Duration::from_secs(60u64 * interval_mins as u64);

        log::debug!(
            "KeyRotationScheduler::next_delay(last_update.is_none() == {})",
            last_update.is_none(),
        );

        if let Some(last_update) = last_update {
            // Check when the key should expire
            let key_age = Duration::from_secs(
                (Utc::now().signed_duration_since(last_update)).num_seconds() as u64
            );
            let remaining_time = delay.checked_sub(key_age).unwrap_or(
                Duration::from_secs(0)
            );
            delay = cmp::max(Duration::from_secs(0), cmp::min(remaining_time, delay));
        }

        Box::new(Delay::new(Instant::now() + delay).map_err(|_| ()))
    }
}

pub struct KeyManager {
    daemon_tx: mpsc::Sender<InternalDaemonEvent>,
    http_handle: mullvad_rpc::HttpHandle,
    tokio_remote: Remote,
    current_job: Option<CancelHandle>,
    abort_scheduler_tx: Option<oneshot::Sender<()>>,
}

pub struct KeyRotationParameters {
    pub public_key: Option<PublicKey>,
    pub interval: Option<u32>,
}

impl KeyManager {
    pub(crate) fn new(
        daemon_tx: mpsc::Sender<InternalDaemonEvent>,
        http_handle: mullvad_rpc::HttpHandle,
        tokio_remote: Remote,
        automatic_key_rotation: KeyRotationParameters,
    ) -> Self {
        let mut manager = Self {
            daemon_tx,
            http_handle,
            tokio_remote,
            current_job: None,
            abort_scheduler_tx: None,
        };
        manager.update_rotation_interval(automatic_key_rotation);

        manager
    }

    /// Update automatic key rotation interval (given in hours)
    /// Passing `None` for the interval will use the default value.
    /// A value of `0` disables automatic key rotation.
    pub fn update_rotation_interval(
        &mut self,
        automatic_key_rotation: KeyRotationParameters,
    ) {
        log::debug!("update_rotation_interval");
        if self.abort_scheduler_tx.is_some() {
            // Stop existing scheduler, if one exists
            let tx = self.abort_scheduler_tx.take().unwrap();
            let _ = tx.send(());
        }
        self.abort_scheduler_tx = match automatic_key_rotation.interval {
            // Interval=0 disables automatic key rotation
            Some(0) => None,
            _ => KeyRotationScheduler::new(
                self.tokio_remote.clone(),
                self.daemon_tx.clone(),
                automatic_key_rotation.public_key,
                automatic_key_rotation.interval,
            ).ok(),
        };
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
        self.run_future_sync(self.replace_key_rpc(account, old_key, new_key))
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
        &self,
        account: AccountToken,
        old_key: PublicKey,
        new_key: PrivateKey,
    ) -> impl Future<Item = WireguardData, Error = JsonRpcError> + Send {
        let mut rpc = mullvad_rpc::WireguardKeyProxy::new(self.http_handle.clone());
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
