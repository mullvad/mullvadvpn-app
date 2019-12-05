use crate::InternalDaemonEvent;
use chrono::offset::Utc;
use futures::{future::Executor, sync::oneshot, Async, Future, Poll};
use jsonrpc_client_core::Error as JsonRpcError;
use mullvad_types::account::AccountToken;
pub use mullvad_types::wireguard::*;
use std::{sync::mpsc, time::{Duration, Instant}};
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
}

pub type Result<T> = std::result::Result<T, Error>;

use crate::ManagementCommand;
use talpid_core::tunnel_state_machine::TunnelCommand;

pub struct KeyRotationScheduler {
    daemon_tx: mpsc::Sender<InternalDaemonEvent>,
    delay: Option<Box<dyn Future<Item = (), Error = ()> + Send>>,
}

impl Future for KeyRotationScheduler {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Error> {
        log::debug!("Poll key rotation future");

        if let Some(delay) = &mut self.delay {
            match delay.poll() {
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(_) => return Err(Error::Delay),
                _ => (),
            }
        }

        let (wg_tx, wg_rx) = oneshot::channel();

        let _ = self.daemon_tx.send(InternalDaemonEvent::ManagementInterfaceEvent(
            ManagementCommand::GenerateWireguardKey(wg_tx)
        )).map_err(|_| Error::Delay)?;

        let somedelay = Instant::now() + Duration::from_secs(30);
        self.delay = Some(Box::new(Delay::new(somedelay)
            .map_err(|_| ())
        ));
        return self.delay
            .as_mut()
            .unwrap()
            .poll()
            .map_err(|_| Error::Delay);
    }
}

impl KeyRotationScheduler {
    pub(crate) fn new(
        tokio_remote: Remote,
        daemon_tx: mpsc::Sender<InternalDaemonEvent>,
        initial_delay: Option<Duration>,
    ) -> Result<oneshot::Sender<()>> {
        let (
            terminate_auto_rotation_tx,
            terminate_auto_rotation_rx
        ) = oneshot::channel();

        let delay: Option<Box<dyn Future<Item = (), Error = ()> + Send>> =
            if let Some(delay) = initial_delay {
                Some( Box::new(Delay::new(Instant::now() + delay).map_err(|_| ())) )
            } else {
                None
            };

        let fut = Self {
            daemon_tx: daemon_tx.clone(),
            delay,
        };

        tokio_remote.execute(
            fut.map_err(|_| {
                log::error!("Failed to run key rotation scheduler")
            }) // FIXME: err
        ); // FIXME: select terminate rx

        Ok(terminate_auto_rotation_tx)
    }
}

pub struct KeyManager {
    daemon_tx: mpsc::Sender<InternalDaemonEvent>,
    http_handle: mullvad_rpc::HttpHandle,
    tokio_remote: Remote,
    current_job: Option<CancelHandle>,
    abort_scheduler_tx: Option<oneshot::Sender<()>>,
}

impl KeyManager {
    pub(crate) fn new(
        daemon_tx: mpsc::Sender<InternalDaemonEvent>,
        http_handle: mullvad_rpc::HttpHandle,
        tokio_remote: Remote,
    ) -> Self {
        let remote_clone = tokio_remote.clone();
        let daemon_tx_clone = daemon_tx.clone();

        Self {
            daemon_tx,
            http_handle,
            tokio_remote,
            current_job: None,
            abort_scheduler_tx: KeyRotationScheduler::new(
                remote_clone,
                daemon_tx_clone,
                Some(Duration::from_secs(30)),
            ).ok()
        }
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
