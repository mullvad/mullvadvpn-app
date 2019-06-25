use crate::InternalDaemonEvent;
use futures::{future::Executor, sync::oneshot, Async, Future, Poll};
use jsonrpc_client_core::Error as JsonRpcError;
use mullvad_types::{account::AccountToken, wireguard::WireguardData};
use std::{sync::mpsc, time::Duration};
pub use talpid_types::net::wireguard::*;
use talpid_types::ErrorExt;
use tokio_core::reactor::Remote;
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    RetryIf,
};

const TOO_MANY_KEYS_ERROR_CODE: i64 = -703;


#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to generate private key")]
    GenerationError(#[error(cause)] rand::Error),
    #[error(display = "Failed to spawn future")]
    ExectuionError,
    #[error(display = "Unexpected RPC error")]
    RpcError(#[error(cause)] jsonrpc_client_core::Error),
    #[error(display = "Account already has maximum number of keys")]
    TooManyKeys,
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub struct KeyManager {
    daemon_tx: mpsc::Sender<InternalDaemonEvent>,
    http_handle: mullvad_rpc::HttpHandle,
    tokio_remote: Remote,
    current_job: Option<CancelHandle>,
}

impl KeyManager {
    pub(crate) fn new(
        daemon_tx: mpsc::Sender<InternalDaemonEvent>,
        http_handle: mullvad_rpc::HttpHandle,
        tokio_remote: Remote,
    ) -> Self {
        Self {
            daemon_tx,
            http_handle,
            tokio_remote,
            current_job: None,
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
        let (tx, rx) = oneshot::channel();
        let fut = self.push_future_generator(account, private_key)().then(|result| {
            let _ = tx.send(result);
            Ok(())
        });
        self.tokio_remote
            .execute(fut)
            .map_err(|_e| Error::ExectuionError)?;

        rx.wait()
            .map_err(|_| Error::ExectuionError)?
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
                    },
                ))
            };
        Box::new(push_future)
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
