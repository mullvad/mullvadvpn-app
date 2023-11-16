//! This module is responsible for enabling custom [`AccessMethodSetting`]s to
//! be used when connecting to the Mullvad API. In practice this means
//! converting [`AccessMethodSetting`]s to connection details as encoded by
//! [`ApiConnectionMode`], which in turn is used by `mullvad-api` for
//! establishing connections when performing API requests.
#[cfg(target_os = "android")]
use crate::{DaemonCommand, DaemonEventSender};
use futures::{
    channel::{mpsc, oneshot},
    stream::unfold,
    Stream, StreamExt,
};
use mullvad_api::{
    availability::ApiAvailabilityHandle,
    proxy::{ApiConnectionMode, ProxyConfig},
    ApiEndpointUpdateCallback,
};
use mullvad_relay_selector::RelaySelector;
use mullvad_types::access_method::{self, AccessMethod, AccessMethodSetting, BuiltInAccessMethod};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, Weak},
};
#[cfg(target_os = "android")]
use talpid_core::mpsc::Sender;
use talpid_core::tunnel_state_machine::TunnelCommand;
use talpid_types::net::{openvpn::ProxySettings, AllowedEndpoint, Endpoint};

pub enum Message {
    Get(ResponseTx<AccessMethodSetting>),
    Set(ResponseTx<()>, AccessMethodSetting),
    Next(ResponseTx<ApiConnectionMode>),
    Update(ResponseTx<()>, Vec<AccessMethodSetting>),
}

#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Oddly specific.
    #[error(display = "Very Generic error.")]
    Generic,
}

#[derive(Clone)]
pub struct AccessModeSelectorHandle {
    cmd_tx: mpsc::UnboundedSender<Message>,
}

impl AccessModeSelectorHandle {
    pub fn new(
        cache_dir: PathBuf,
        relay_selector: RelaySelector,
        connection_modes: Vec<AccessMethodSetting>,
    ) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::unbounded();

        let mut actor = AccessModeSelector {
            cmd_rx,
            state: ApiConnectionModeProvider::new(cache_dir, relay_selector, connection_modes),
        };
        tokio::spawn(async move { actor.run().await });
        Self { cmd_tx }
    }

    async fn send_command<T>(&self, make_cmd: impl FnOnce(ResponseTx<T>) -> Message) -> Result<T> {
        let (tx, rx) = oneshot::channel();
        // TODO(markus): Error handling
        self.cmd_tx.unbounded_send(make_cmd(tx)).unwrap();
        // TODO(markus): Error handling
        rx.await.unwrap()
    }

    pub async fn get_access_method(&self) -> Result<AccessMethodSetting> {
        self.send_command(Message::Get).await.map_err(|err| {
            log::error!("Failed to get current access method!");
            err
        })
    }

    pub async fn set_access_method(&self, value: AccessMethodSetting) -> Result<()> {
        self.send_command(|tx| Message::Set(tx, value))
            .await
            .map_err(|err| {
                log::error!("Failed to set new access method!");
                err
            })
    }

    pub async fn update_access_methods(&self, values: Vec<AccessMethodSetting>) -> Result<()> {
        self.send_command(|tx| Message::Update(tx, values))
            .await
            .map_err(|err| {
                log::error!("Failed to update new access methods!");
                err
            })
    }

    pub async fn next(&self) -> Result<ApiConnectionMode> {
        self.send_command(Message::Next).await.map_err(|err| {
            log::error!("Failed to update new access methods!");
            err
        })
    }

    /// Stream the connection modes of this actor.
    pub fn as_stream(&self) -> impl Stream<Item = ApiConnectionMode> {
        let handle = self.clone();
        unfold(handle, |handle| async move {
            let connection_mode = handle
                .next()
                .await
                .expect("It should always be safe to `unwrap` a new API connection mode");
            Some((connection_mode, handle))
        })
    }
}

pub struct AccessModeSelector {
    cmd_rx: mpsc::UnboundedReceiver<Message>,
    state: ApiConnectionModeProvider,
}

impl AccessModeSelector {
    async fn run(&mut self) {
        while let Some(cmd) = self.cmd_rx.next().await {
            let _ = match cmd {
                Message::Get(tx) => self.on_get_access_method(tx),
                Message::Set(tx, value) => self.on_set_access_method(tx, value),
                Message::Next(tx) => self.on_next_connection_mode(tx),
                Message::Update(tx, values) => self.on_update_access_methods(tx, values),
            }
            .map_err(|err| {
                log::error!("todo(markus): Some error occured {err}");
                err
            });
        }
    }

    fn reply<T>(&self, tx: ResponseTx<T>, value: T) -> Result<()> {
        // TODO(markus): The error probably should come from the value/tx
        tx.send(Ok(value)).map_err(|_| Error::Generic)
    }

    fn on_get_access_method(&mut self, tx: ResponseTx<AccessMethodSetting>) -> Result<()> {
        let value = self.get_access_method()?;
        self.reply(tx, value)
    }

    fn get_access_method(&mut self) -> Result<AccessMethodSetting> {
        let connections_modes = self.state.connection_modes.lock().unwrap();
        Ok(connections_modes.peek())
    }

    fn on_set_access_method(
        &mut self,
        tx: ResponseTx<()>,
        value: AccessMethodSetting,
    ) -> Result<()> {
        self.set_access_method(value)?;
        self.reply(tx, ())
    }

    fn set_access_method(&mut self, value: AccessMethodSetting) -> Result<()> {
        let mut connections_modes = self.state.connection_modes.lock().unwrap();
        connections_modes.set_access_method(value);
        Ok(())
    }

    fn on_next_connection_mode(&mut self, tx: ResponseTx<ApiConnectionMode>) -> Result<()> {
        let next = self.next_connection_mode();
        // Save the new connection mode to cache!
        {
            let cache_dir = self.state.cache_dir.clone();
            let next = next.clone();
            tokio::spawn(async move {
                if next.save(&cache_dir).await.is_err() {
                    log::warn!(
                        "Failed to save {connection_mode} to cache",
                        connection_mode = next
                    )
                }
            });
        }
        self.reply(tx, next)
    }

    fn next_connection_mode(&mut self) -> ApiConnectionMode {
        let access_method = {
            let mut connection_modes = self.state.connection_modes.lock().unwrap();
            connection_modes
                .next()
                .map(|access_method_setting| access_method_setting.access_method)
                .unwrap_or(AccessMethod::from(BuiltInAccessMethod::Direct))
        };

        let connection_mode = self.state.from(access_method);
        log::info!("New API connection mode selected: {}", connection_mode);
        connection_mode
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
        let mut connection_modes = self.state.connection_modes.lock().unwrap();
        connection_modes.update_access_methods(values);
        Ok(())
    }
}

type ResponseTx<T> = oneshot::Sender<Result<T>>;
type Result<T> = std::result::Result<T, Error>;

/// A stream that returns the next API connection mode to use for reaching the API.
///
/// When `mullvad-api` fails to contact the API, it requests a new connection
/// mode. The API can be connected to either directly (i.e.,
/// [`ApiConnectionMode::Direct`]) via a bridge ([`ApiConnectionMode::Proxied`])
/// or via any supported custom proxy protocol ([`api_access_methods::ObfuscationProtocol`]).
///
/// The strategy for determining the next [`ApiConnectionMode`] is handled by
/// [`ConnectionModesIterator`].
pub struct ApiConnectionModeProvider {
    cache_dir: PathBuf,
    /// Used for selecting a Bridge when the `Mullvad Bridges` access method is used.
    relay_selector: RelaySelector,
    connection_modes: Arc<Mutex<ConnectionModesIterator>>,
}

impl ApiConnectionModeProvider {
    pub(crate) fn new(
        cache_dir: PathBuf,
        relay_selector: RelaySelector,
        connection_modes: Vec<AccessMethodSetting>,
    ) -> Result<Self, Error> {
        let connection_modes_iterator = ConnectionModesIterator::new(connection_modes)?;
        Ok(Self {
            cache_dir,
            relay_selector,
            connection_modes: Arc::new(Mutex::new(connection_modes_iterator)),
        })
    }

    /// Ad-hoc version of [`std::convert::From::from`], but since some
    /// [`ApiConnectionMode`]s require extra logic/data from
    /// [`ApiConnectionModeProvider`] the standard [`std::convert::From`] trait
    /// can not be implemented.
    fn from(&mut self, access_method: AccessMethod) -> ApiConnectionMode {
        match access_method {
            AccessMethod::BuiltIn(access_method) => match access_method {
                BuiltInAccessMethod::Direct => ApiConnectionMode::Direct,
                BuiltInAccessMethod::Bridge => self
                    .relay_selector
                    .get_bridge_forced()
                    .and_then(|settings| match settings {
                        ProxySettings::Shadowsocks(ss_settings) => {
                            let ss_settings: access_method::Shadowsocks =
                                access_method::Shadowsocks::new(
                                    ss_settings.peer,
                                    ss_settings.cipher,
                                    ss_settings.password,
                                );
                            Some(ApiConnectionMode::Proxied(ProxyConfig::Shadowsocks(
                                ss_settings,
                            )))
                        }
                        _ => {
                            log::error!("Received unexpected proxy settings type");
                            None
                        }
                    })
                    .unwrap_or(ApiConnectionMode::Direct),
            },
            AccessMethod::Custom(access_method) => match access_method {
                access_method::CustomAccessMethod::Shadowsocks(shadowsocks_config) => {
                    ApiConnectionMode::Proxied(ProxyConfig::Shadowsocks(shadowsocks_config))
                }
                access_method::CustomAccessMethod::Socks5(socks_config) => {
                    ApiConnectionMode::Proxied(ProxyConfig::Socks(socks_config))
                }
            },
        }
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

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "No access methods were provided.")]
    NoAccessMethods,
}

impl ConnectionModesIterator {
    pub fn new(access_methods: Vec<AccessMethodSetting>) -> Result<ConnectionModesIterator, Error> {
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
    ) -> Result<(), Error> {
        self.available_modes = Self::new_iterator(access_methods)?;
        Ok(())
    }

    /// Create a cyclic iterator of [`AccessMethodSetting`]s.
    ///
    /// If the `access_methods` argument is an empty vector, an [`Error`] is
    /// returned.
    fn new_iterator(
        access_methods: Vec<AccessMethodSetting>,
    ) -> Result<Box<dyn Iterator<Item = AccessMethodSetting> + Send>, Error> {
        if access_methods.is_empty() {
            Err(Error::NoAccessMethods)
        } else {
            Ok(Box::new(access_methods.into_iter().cycle()))
        }
    }

    /// Look at the currently active [`AccessMethod`]
    pub fn peek(&self) -> AccessMethodSetting {
        self.current.clone()
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

/// Notifies the tunnel state machine that the API (real or proxied) endpoint has
/// changed. [ApiEndpointUpdaterHandle::callback()] creates a callback that may
/// be passed to the `mullvad-api` runtime.
pub(super) struct ApiEndpointUpdaterHandle {
    tunnel_cmd_tx: Arc<Mutex<Option<Weak<mpsc::UnboundedSender<TunnelCommand>>>>>,
}

impl ApiEndpointUpdaterHandle {
    pub fn new() -> Self {
        Self {
            tunnel_cmd_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_tunnel_command_tx(&self, tunnel_cmd_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>) {
        *self.tunnel_cmd_tx.lock().unwrap() = Some(tunnel_cmd_tx);
    }

    pub fn callback(&self) -> impl ApiEndpointUpdateCallback {
        let tunnel_tx = self.tunnel_cmd_tx.clone();
        move |allowed_endpoint: AllowedEndpoint| {
            let inner_tx = tunnel_tx.clone();
            async move {
                let tunnel_tx = if let Some(tunnel_tx) = { inner_tx.lock().unwrap().as_ref() }
                    .and_then(|tx: &Weak<mpsc::UnboundedSender<TunnelCommand>>| tx.upgrade())
                {
                    tunnel_tx
                } else {
                    log::error!("Rejecting allowed endpoint: Tunnel state machine is not running");
                    return false;
                };
                let (result_tx, result_rx) = oneshot::channel();
                let _ = tunnel_tx.unbounded_send(TunnelCommand::AllowEndpoint(
                    allowed_endpoint.clone(),
                    result_tx,
                ));
                // Wait for the firewall policy to be updated.
                let _ = result_rx.await;
                log::debug!(
                    "API endpoint: {endpoint}",
                    endpoint = allowed_endpoint.endpoint
                );
                true
            }
        }
    }
}

pub(super) fn get_allowed_endpoint(endpoint: Endpoint) -> AllowedEndpoint {
    #[cfg(unix)]
    let clients = talpid_types::net::AllowedClients::Root;
    #[cfg(windows)]
    let clients = {
        let daemon_exe = std::env::current_exe().expect("failed to obtain executable path");
        vec![
            daemon_exe
                .parent()
                .expect("missing executable parent directory")
                .join("mullvad-problem-report.exe"),
            daemon_exe,
        ]
        .into()
    };

    AllowedEndpoint { endpoint, clients }
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
