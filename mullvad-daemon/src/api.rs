//! This module is responsible for enabling custom [`AccessMethodSetting`]s to
//! be used when connecting to the Mullvad API. In practice this means
//! converting [`AccessMethodSetting`]s to connection details as encoded by
//! [`ApiConnectionMode`], which in turn is used by `mullvad-api` for
//! establishing connections when performing API requests.
#[cfg(target_os = "android")]
use crate::DaemonCommand;
use crate::DaemonEventSender;
use futures::{
    channel::{
        mpsc,
        oneshot::{self, Canceled},
    },
    Stream, StreamExt,
};
use mullvad_api::{
    availability::ApiAvailabilityHandle,
    proxy::{ApiConnectionMode, ProxyConfig},
    AddressCache,
};
use mullvad_relay_selector::RelaySelector;
use mullvad_types::access_method::{AccessMethod, AccessMethodSetting, BuiltInAccessMethod};
use std::{net::SocketAddr, path::PathBuf};
use talpid_core::mpsc::Sender;
use talpid_types::net::{AllowedClients, AllowedEndpoint, Endpoint, TransportProtocol};

pub enum Message {
    Get(ResponseTx<ResolvedConnectionMode>),
    Set(ResponseTx<()>, AccessMethodSetting),
    Next(ResponseTx<ApiConnectionMode>),
    Update(ResponseTx<()>, Vec<AccessMethodSetting>),
    Resolve(ResponseTx<ResolvedConnectionMode>, AccessMethodSetting),
}

/// A [`NewAccessMethodEvent`] is emitted when the active access method changes,
/// which happens in any of the following two scenarios:
///
/// * When a [`mullvad_api::rest::RequestService`] requests a new
/// [`ApiConnectionMode`] from the running [`AccessModeSelector`]. This will
/// lead to a [`crate::InternalDaemonEvent::AccessMethodEvent`] being sent to
/// the daemon, which in turn will notify all clients about the new access
/// method.
///
/// * When testing if some [`AccessMethodSetting`] can be used to reach the
/// Mullvad API. In this scenario, the currently active access method will
/// temporarily change (approximately for the duration of 1 API call). Since
/// this is just an internal test which should be opaque to any client, it
/// should not produce any unwanted noise and as such it is *not* broadcasted
/// after the daemon is done processing this [`NewAccessMethodEvent`].
pub struct NewAccessMethodEvent {
    /// The new active [`AccessMethodSetting`].
    pub setting: AccessMethodSetting,
    /// The endpoint which represents how to connect to the Mullvad API and
    /// which clients are allowed to initiate such a connection.
    pub endpoint: AllowedEndpoint,
    /// If the daemon should notify clients about the new access method.
    ///
    /// Defaults to `true`.
    pub announce: bool,
}

impl NewAccessMethodEvent {
    /// Create a new [`NewAccessMethodEvent`] for the daemon to process. A
    /// [`oneshot::Receiver`] can be used to await the daemon while it finishes
    /// handling the new event.
    pub fn new(setting: AccessMethodSetting, endpoint: AllowedEndpoint) -> NewAccessMethodEvent {
        NewAccessMethodEvent {
            setting,
            endpoint,
            announce: true,
        }
    }

    /// Whether the daemon should notify clients about the new access method or
    /// not.
    ///
    /// * If `announce` is set to `true` the daemon will broadcast this event to
    /// clients.
    /// * If `announce` is set to `false` the daemon will *not* broadcast this
    /// event.
    pub fn announce(mut self, announce: bool) -> Self {
        self.announce = announce;
        self
    }

    /// Send an internal daemon event which will punch a hole in the firewall
    /// for the connection mode we are testing.
    ///
    /// Returns the channel on which the daemon will send a message over when it
    /// is done applying the firewall changes.
    pub(crate) async fn send(
        self,
        daemon_event_sender: DaemonEventSender<(NewAccessMethodEvent, oneshot::Sender<()>)>,
    ) -> std::result::Result<(), Canceled> {
        // It is up to the daemon to actually allow traffic to/from `api_endpoint`
        // by updating the firewall. This [`oneshot::Sender`] allows the daemon to
        // communicate when that action is done.
        let (update_finished_tx, update_finished_rx) = oneshot::channel();
        let _ = daemon_event_sender.send((self, update_finished_tx));
        // Wait for the daemon to finish processing `event`.
        update_finished_rx.await
    }
}

/// This struct represent a concrete API endpoint (in the form of an
/// [`ApiConnectionMode`] and [`AllowedEndpoint`]) which has been derived from
/// some [`AccessMethodSetting`] (most likely the currently active access
/// method). These logically related values are sometimes useful to group
/// together into one value, which is encoded by [`ResolvedConnectionMode`].
#[derive(Clone)]
pub struct ResolvedConnectionMode {
    /// The connection strategy to be used by the `mullvad-api` crate when
    /// initializing API requests.
    pub connection_mode: ApiConnectionMode,
    /// The actual endpoint of the Mullvad API and which clients should be
    /// allowed to initialize a connection to this endpoint.
    pub endpoint: AllowedEndpoint,
    /// This is the [`AccessMethodSetting`] which resolved into
    /// `connection_mode` and `endpoint`.
    pub setting: AccessMethodSetting,
}

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "No access methods were provided.")]
    NoAccessMethods,
    #[error(display = "AccessModeSelector is not receiving any messages.")]
    SendFailed(#[error(source)] mpsc::TrySendError<Message>),
    #[error(display = "AccessModeSelector is not receiving any messages.")]
    OneshotSendFailed,
    #[error(display = "AccessModeSelector is not responding.")]
    NotRunning(#[error(source)] oneshot::Canceled),
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Get(_) => f.write_str("Get"),
            Message::Set(..) => f.write_str("Set"),
            Message::Next(_) => f.write_str("Next"),
            Message::Update(..) => f.write_str("Update"),
            Message::Resolve(..) => f.write_str("Resolve"),
        }
    }
}

impl Error {
    /// Check if this error implies that the currenly running
    /// [`AccessModeSelector`] can not continue to operate properly.
    ///
    /// To recover from this kind of error, the daemon will probably have to
    /// intervene.
    fn is_critical_error(&self) -> bool {
        matches!(
            self,
            Error::SendFailed(..) | Error::OneshotSendFailed | Error::NotRunning(..)
        )
    }
}

type ResponseTx<T> = oneshot::Sender<Result<T>>;
type Result<T> = std::result::Result<T, Error>;

/// A channel for sending [`Message`] commands to a running
/// [`AccessModeSelector`].
#[derive(Clone)]
pub struct AccessModeSelectorHandle {
    cmd_tx: mpsc::UnboundedSender<Message>,
}

impl AccessModeSelectorHandle {
    async fn send_command<T>(&self, make_cmd: impl FnOnce(ResponseTx<T>) -> Message) -> Result<T> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx.unbounded_send(make_cmd(tx))?;
        rx.await.map_err(Error::NotRunning)?
    }

    pub async fn get_current(&self) -> Result<ResolvedConnectionMode> {
        self.send_command(Message::Get).await.map_err(|err| {
            log::debug!("Failed to get current access method!");
            err
        })
    }

    pub async fn set_access_method(&self, value: AccessMethodSetting) -> Result<()> {
        self.send_command(|tx| Message::Set(tx, value))
            .await
            .map_err(|err| {
                log::debug!("Failed to set new access method!");
                err
            })
    }

    pub async fn update_access_methods(&self, values: Vec<AccessMethodSetting>) -> Result<()> {
        self.send_command(|tx| Message::Update(tx, values))
            .await
            .map_err(|err| {
                log::debug!("Failed to switch to a new set of access methods");
                err
            })
    }

    pub async fn resolve(&self, setting: AccessMethodSetting) -> Result<ResolvedConnectionMode> {
        self.send_command(|tx| Message::Resolve(tx, setting))
            .await
            .map_err(|err| {
                log::error!("Failed to update new access methods!");
                err
            })
    }

    pub async fn next(&self) -> Result<ApiConnectionMode> {
        self.send_command(Message::Next).await.map_err(|err| {
            log::debug!("Failed while getting the next access method");
            err
        })
    }

    /// Convert this handle to a [`Stream`] of [`ApiConnectionMode`] from the
    /// associated [`AccessModeSelector`].
    ///
    /// Calling `next` on this stream will poll for the next access method,
    /// which will be lazily produced (on-demand rather than speculatively).
    pub fn into_stream(self) -> impl Stream<Item = ApiConnectionMode> {
        futures::stream::unfold(self, |handle| async move {
            match handle.next().await {
                Ok(connection_mode) => Some((connection_mode, handle)),
                // End this stream in case of failure in `next`. `next` should
                // not fail if the actor is in a good state.
                Err(_) => None,
            }
        })
    }
}

/// A small actor which takes care of handling the logic around rotating
/// connection modes to be used for Mullvad API request.
///
/// When `mullvad-api` fails to contact the API, it will request a new
/// connection mode. The API can be connected to either directly (i.e.,
/// [`ApiConnectionMode::Direct`]) via a bridge ([`ApiConnectionMode::Proxied`])
/// or via any supported custom proxy protocol
/// ([`api_access_methods::ObfuscationProtocol`]).
///
/// The strategy for determining the next [`ApiConnectionMode`] is handled by
/// [`ConnectionModesIterator`].
pub struct AccessModeSelector {
    cmd_rx: mpsc::UnboundedReceiver<Message>,
    cache_dir: PathBuf,
    /// Used for selecting a Bridge when the `Mullvad Bridges` access method is used.
    relay_selector: RelaySelector,
    connection_modes: ConnectionModesIterator,
    address_cache: AddressCache,
    access_method_event_sender: DaemonEventSender<(NewAccessMethodEvent, oneshot::Sender<()>)>,
    current: ResolvedConnectionMode,
}

impl AccessModeSelector {
    pub(crate) async fn spawn(
        cache_dir: PathBuf,
        relay_selector: RelaySelector,
        connection_modes: Vec<AccessMethodSetting>,
        access_method_event_sender: DaemonEventSender<(NewAccessMethodEvent, oneshot::Sender<()>)>,
        address_cache: AddressCache,
    ) -> Result<AccessModeSelectorHandle> {
        let (cmd_tx, cmd_rx) = mpsc::unbounded();

        let mut connection_modes = match ConnectionModesIterator::new(connection_modes) {
            Ok(provider) => provider,
            Err(Error::NoAccessMethods) | Err(_) => {
                // No settings seem to have been found. Default to using the the
                // direct access method.
                let default = mullvad_types::access_method::Settings::direct();
                ConnectionModesIterator::new(vec![default]).expect(
                    "Failed to create the data structure responsible for managing access methods",
                )
            }
        };

        let initial_connection_mode = {
            let next = connection_modes.next().ok_or(Error::NoAccessMethods)?;
            Self::resolve_inner(next, &relay_selector, &address_cache).await
        };

        let selector = AccessModeSelector {
            cmd_rx,
            cache_dir,
            relay_selector,
            connection_modes,
            address_cache,
            access_method_event_sender,
            current: initial_connection_mode,
        };

        tokio::spawn(selector.into_future());

        Ok(AccessModeSelectorHandle { cmd_tx })
    }

    async fn into_future(mut self) {
        while let Some(cmd) = self.cmd_rx.next().await {
            log::trace!("Processing {cmd} command");
            let execution = match cmd {
                Message::Get(tx) => self.on_get_access_method(tx),
                Message::Set(tx, value) => self.on_set_access_method(tx, value),
                Message::Next(tx) => self.on_next_connection_mode(tx).await,
                Message::Update(tx, values) => self.on_update_access_methods(tx, values),
                Message::Resolve(tx, setting) => self.on_resolve_access_method(tx, setting).await,
            };
            match execution {
                Ok(_) => (),
                Err(error) if error.is_critical_error() => {
                    log::error!("AccessModeSelector failed due to an internal error and won't be able to recover without a restart. {error}");
                    break;
                }
                Err(error) => {
                    log::debug!("AccessModeSelector failed processing command due to {error}");
                }
            }
        }
    }

    fn reply<T>(&self, tx: ResponseTx<T>, value: T) -> Result<()> {
        tx.send(Ok(value)).map_err(|_| Error::OneshotSendFailed)?;
        Ok(())
    }

    fn on_get_access_method(&mut self, tx: ResponseTx<ResolvedConnectionMode>) -> Result<()> {
        self.reply(tx, self.current.clone())
    }

    fn on_set_access_method(
        &mut self,
        tx: ResponseTx<()>,
        value: AccessMethodSetting,
    ) -> Result<()> {
        self.set_access_method(value);
        self.reply(tx, ())
    }

    /// Set the next access method to be returned by the [`Stream`] produced by
    /// calling `into_stream`.
    fn set_access_method(&mut self, value: AccessMethodSetting) {
        self.connection_modes.set_access_method(value);
    }

    async fn on_next_connection_mode(&mut self, tx: ResponseTx<ApiConnectionMode>) -> Result<()> {
        let next = self.next_connection_mode().await?;
        self.reply(tx, next)
    }

    async fn next_connection_mode(&mut self) -> Result<ApiConnectionMode> {
        let access_method = self.connection_modes.next().ok_or(Error::NoAccessMethods)?;
        log::info!(
            "A new API access method has been selected: {name}",
            name = access_method.name
        );
        let resolved = self.resolve(access_method).await;
        // Note: If the daemon is busy waiting for a call to this function
        // to complete while we wait for the daemon to fully handle this
        // `NewAccessMethodEvent`, then we find ourselves in a deadlock.
        // This can happen during daemon startup when spawning a new
        // `MullvadRestHandle`, which will call and await `next` on a Stream
        // created from this `AccessModeSelector` instance. As such, the
        // completion channel is discarded in this instance.
        let setting = resolved.setting.clone();
        let endpoint = resolved.endpoint.clone();
        let daemon_sender = self.access_method_event_sender.clone();
        tokio::spawn(async move {
            let _ = NewAccessMethodEvent::new(setting, endpoint)
                .send(daemon_sender)
                .await;
        });

        // Save the new connection mode to cache!
        let cache_dir = self.cache_dir.clone();
        let new_connection_mode = resolved.connection_mode.clone();
        tokio::spawn(async move {
            if new_connection_mode.save(&cache_dir).await.is_err() {
                log::warn!(
                    "Failed to save {connection_mode} to cache",
                    connection_mode = new_connection_mode
                )
            }
        });

        self.current = resolved;
        Ok(self.current.connection_mode.clone())
    }
    fn on_update_access_methods(
        &mut self,
        tx: ResponseTx<()>,
        values: Vec<AccessMethodSetting>,
    ) -> Result<()> {
        self.update_access_methods(values)?;
        self.reply(tx, ())
    }

    fn update_access_methods(&mut self, values: Vec<AccessMethodSetting>) -> Result<()> {
        self.connection_modes.update_access_methods(values)
    }

    pub async fn on_resolve_access_method(
        &mut self,
        tx: ResponseTx<ResolvedConnectionMode>,
        setting: AccessMethodSetting,
    ) -> Result<()> {
        let reply = self.resolve(setting).await;
        self.reply(tx, reply)
    }

    async fn resolve(&mut self, access_method: AccessMethodSetting) -> ResolvedConnectionMode {
        Self::resolve_inner(access_method, &self.relay_selector, &self.address_cache).await
    }

    async fn resolve_inner(
        access_method: AccessMethodSetting,
        relay_selector: &RelaySelector,
        address_cache: &AddressCache,
    ) -> ResolvedConnectionMode {
        let connection_mode =
            resolve_connection_mode(access_method.access_method.clone(), relay_selector);
        let endpoint =
            resolve_allowed_endpoint(&connection_mode, address_cache.get_address().await);
        ResolvedConnectionMode {
            connection_mode,
            endpoint,
            setting: access_method,
        }
    }
}

/// Ad-hoc version of [`std::convert::From::from`], but since some
/// [`ApiConnectionMode`]s require extra logic/data from [`RelaySelector`] to be
/// instantiated the standard [`std::convert::From`] trait can not be
/// implemented.
fn resolve_connection_mode(
    access_method: AccessMethod,
    relay_selector: &RelaySelector,
) -> ApiConnectionMode {
    match access_method {
        AccessMethod::BuiltIn(BuiltInAccessMethod::Direct) => ApiConnectionMode::Direct,
        AccessMethod::BuiltIn(BuiltInAccessMethod::Bridge) => relay_selector
            .get_bridge_forced()
            .map(ProxyConfig::from)
            .map(ApiConnectionMode::Proxied)
            .unwrap_or_else(|| {
                log::error!(
                    "Received unexpected proxy settings type. Defaulting to direct API connection"
                );
                ApiConnectionMode::Direct
            }),
        AccessMethod::Custom(config) => ApiConnectionMode::Proxied(ProxyConfig::from(config)),
    }
}

/// An iterator which will always produce an [`AccessMethod`].
///
/// Safety: It is always safe to [`unwrap`] after calling [`next`] on a
/// [`std::iter::Cycle`], so thereby it is safe to always call [`unwrap`] on a
/// [`ConnectionModesIterator`].
///
/// [`unwrap`]: Option::unwrap
/// [`next`]: std::iter::Iterator::next
pub struct ConnectionModesIterator {
    available_modes: Box<dyn Iterator<Item = AccessMethodSetting> + Send>,
    next: Option<AccessMethodSetting>,
    current: AccessMethodSetting,
}

impl ConnectionModesIterator {
    pub fn new(
        access_methods: Vec<AccessMethodSetting>,
    ) -> std::result::Result<ConnectionModesIterator, Error> {
        let mut iterator = Self::new_iterator(access_methods)?;
        Ok(Self {
            next: None,
            current: iterator.next().ok_or(Error::NoAccessMethods)?,
            available_modes: iterator,
        })
    }

    /// Set the next [`AccessMethod`] to be returned from this iterator.
    pub fn set_access_method(&mut self, next: AccessMethodSetting) {
        self.next = Some(next);
    }

    /// Update the collection of [`AccessMethod`] which this iterator will
    /// return.
    pub fn update_access_methods(
        &mut self,
        access_methods: Vec<AccessMethodSetting>,
    ) -> std::result::Result<(), Error> {
        self.available_modes = Self::new_iterator(access_methods)?;
        Ok(())
    }

    /// Create a cyclic iterator of [`AccessMethodSetting`]s.
    ///
    /// If the `access_methods` argument is an empty vector, an [`Error`] is
    /// returned.
    fn new_iterator(
        access_methods: Vec<AccessMethodSetting>,
    ) -> std::result::Result<Box<dyn Iterator<Item = AccessMethodSetting> + Send>, Error> {
        if access_methods.is_empty() {
            Err(Error::NoAccessMethods)
        } else {
            Ok(Box::new(access_methods.into_iter().cycle()))
        }
    }
}

impl Iterator for ConnectionModesIterator {
    type Item = AccessMethodSetting;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self
            .next
            .take()
            .or_else(|| self.available_modes.next())
            .unwrap();
        self.current = next.clone();
        Some(next)
    }
}

pub fn resolve_allowed_endpoint(
    connection_mode: &ApiConnectionMode,
    fallback: SocketAddr,
) -> AllowedEndpoint {
    let endpoint = match connection_mode.get_endpoint() {
        Some(endpoint) => endpoint,
        None => Endpoint::from_socket_address(fallback, TransportProtocol::Tcp),
    };
    let clients = allowed_clients(connection_mode);
    AllowedEndpoint { endpoint, clients }
}

#[cfg(unix)]
pub fn allowed_clients(connection_mode: &ApiConnectionMode) -> AllowedClients {
    match connection_mode {
        ApiConnectionMode::Proxied(ProxyConfig::Socks5Local(_)) => AllowedClients::All,
        ApiConnectionMode::Direct | ApiConnectionMode::Proxied(_) => AllowedClients::Root,
    }
}

#[cfg(windows)]
pub fn allowed_clients(connection_mode: &ApiConnectionMode) -> AllowedClients {
    match connection_mode {
        ApiConnectionMode::Proxied(ProxyConfig::Socks5Local(_)) => AllowedClients::all(),
        ApiConnectionMode::Direct | ApiConnectionMode::Proxied(_) => {
            let daemon_exe = std::env::current_exe().expect("failed to obtain executable path");
            vec![
                daemon_exe
                    .parent()
                    .expect("missing executable parent directory")
                    .join("mullvad-problem-report.exe"),
                daemon_exe,
            ]
            .into()
        }
    }
}

pub(crate) fn forward_offline_state(
    api_availability: ApiAvailabilityHandle,
    mut offline_state_rx: mpsc::UnboundedReceiver<bool>,
) {
    tokio::spawn(async move {
        let initial_state = offline_state_rx
            .next()
            .await
            .expect("missing initial offline state");
        api_availability.set_offline(initial_state);
        while let Some(is_offline) = offline_state_rx.next().await {
            api_availability.set_offline(is_offline);
        }
    });
}

#[cfg(target_os = "android")]
pub(crate) fn create_bypass_tx(
    event_sender: &DaemonEventSender,
) -> Option<mpsc::Sender<mullvad_api::SocketBypassRequest>> {
    let (bypass_tx, mut bypass_rx) = mpsc::channel(1);
    let daemon_tx = event_sender.to_specialized_sender();
    tokio::spawn(async move {
        while let Some((raw_fd, done_tx)) = bypass_rx.next().await {
            if daemon_tx
                .send(DaemonCommand::BypassSocket(raw_fd, done_tx))
                .is_err()
            {
                log::error!("Can't send socket bypass request to daemon");
                break;
            }
        }
    });
    Some(bypass_tx)
}
