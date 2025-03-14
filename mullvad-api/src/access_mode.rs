//! This module is responsible for enabling custom [`AccessMethodSetting`]s to
//! be used when connecting to the Mullvad API. In practice this means
//! converting [`AccessMethodSetting`]s to connection details as encoded by
//! [`ApiConnectionMode`], which in turn is used by `mullvad-api` for
//! establishing connections when performing API requests.

use crate::proxy::{ApiConnectionMode, ConnectionModeProvider};
#[cfg(feature = "api-override")]
use crate::ApiEndpoint;
use async_trait::async_trait;
use futures::{
    channel::{mpsc, oneshot},
    StreamExt,
};
use mullvad_types::access_method::{AccessMethod, AccessMethodSetting, Id, Settings};
use talpid_types::net::AllowedEndpoint;

pub enum Message {
    Get(ResponseTx<ResolvedConnectionMode>),
    Use(ResponseTx<()>, Id),
    Rotate(ResponseTx<ApiConnectionMode>),
    Update(ResponseTx<()>, Settings),
    Resolve(
        ResponseTx<Option<ResolvedConnectionMode>>,
        AccessMethodSetting,
    ),
}

/// Calling [`AccessMethodEvent::send`] will cause a
/// [`crate::InternalDaemonEvent::AccessMethodEvent`] being sent to the daemon,
/// which in turn will handle updating the firewall and notifying clients as
/// applicable.
pub enum AccessMethodEvent {
    /// A [`AccessMethodEvent::New`] event is emitted when the active access
    /// method changes.
    ///
    /// This happens when a [`mullvad_api::rest::RequestService`] requests a new
    /// [`ApiConnectionMode`] from the running [`AccessModeSelector`].
    New {
        /// The new active [`AccessMethodSetting`].
        setting: AccessMethodSetting,
        connection_mode: ApiConnectionMode,
        /// The endpoint which represents how to connect to the Mullvad API and
        /// which clients are allowed to initiate such a connection.
        #[cfg(not(target_os = "android"))]
        endpoint: AllowedEndpoint,
    },
    /// Emitted when the the firewall should be updated.
    ///
    /// This is useful for example when testing if some [`AccessMethodSetting`]
    /// can be used to reach the Mullvad API. In this scenario, the currently
    /// active access method will temporarily change (approximately for the
    /// duration of 1 API call). Since this is just an internal test which
    /// should be opaque to any client, it should not produce any unwanted noise
    /// and as such it is *not* broadcasted after the daemon is done processing
    /// this [`AccessMethodEvent::Allow`].
    #[cfg(not(target_os = "android"))]
    Allow { endpoint: AllowedEndpoint },
}

impl AccessMethodEvent {
    pub async fn send(
        self,
        daemon_event_sender: mpsc::UnboundedSender<(AccessMethodEvent, oneshot::Sender<()>)>,
    ) -> Result<()> {
        // It is up to the daemon to actually allow traffic to/from `api_endpoint`
        // by updating the firewall. This [`oneshot::Sender`] allows the daemon to
        // communicate when that action is done.
        let (update_finished_tx, update_finished_rx) = oneshot::channel();
        let _ = daemon_event_sender.unbounded_send((self, update_finished_tx));
        // Wait for the daemon to finish processing `event`.
        update_finished_rx.await.map_err(Error::NotRunning)
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

/// Describes all the ways the daemon service which handles access methods can
/// fail.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No access methods were provided.")]
    NoAccessMethods,
    #[error("Could not resolve access method {access_method:#?}")]
    Resolve { access_method: AccessMethod },
    #[error("AccessModeSelector is not receiving any messages.")]
    SendFailed(#[from] mpsc::TrySendError<Message>),
    #[error("AccessModeSelector is not receiving any messages.")]
    OneshotSendFailed,
    #[error("AccessModeSelector is not responding.")]
    NotRunning(#[from] oneshot::Canceled),
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Get(_) => f.write_str("Get"),
            Message::Use(..) => f.write_str("Set"),
            Message::Rotate(_) => f.write_str("Rotate"),
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
        self.send_command(Message::Get).await.inspect_err(|_| {
            log::debug!("Failed to get current access method!");
        })
    }

    pub async fn use_access_method(&self, value: Id) -> Result<()> {
        self.send_command(|tx| Message::Use(tx, value))
            .await
            .inspect_err(|_| {
                log::debug!("Failed to set new access method!");
            })
    }

    pub async fn update_access_methods(&self, access_methods: Settings) -> Result<()> {
        self.send_command(|tx| Message::Update(tx, access_methods))
            .await
            .inspect_err(|_| {
                log::debug!("Failed to switch to a new set of access methods");
            })
    }

    /// Try to resolve an access method into a set of connection details.
    ///
    /// This might fail if the underlying store/cache where `setting` is the key is empty.
    /// In that case, `Ok(None)` is returned.
    pub async fn resolve(
        &self,
        setting: AccessMethodSetting,
    ) -> Result<Option<ResolvedConnectionMode>> {
        self.send_command(|tx| Message::Resolve(tx, setting))
            .await
            .inspect_err(|_| {
                log::error!("Failed to update new access methods!");
            })
    }

    pub async fn rotate(&self) -> Result<ApiConnectionMode> {
        self.send_command(Message::Rotate).await.inspect_err(|_| {
            log::debug!("Failed while getting the next access method");
        })
    }
}

pub struct AccessModeConnectionModeProvider {
    initial: ApiConnectionMode,
    handle: AccessModeSelectorHandle,
    change_rx: mpsc::UnboundedReceiver<ApiConnectionMode>,
}

impl AccessModeConnectionModeProvider {
    fn new(
        handle: AccessModeSelectorHandle,
        initial_connection_mode: ApiConnectionMode,
        change_rx: mpsc::UnboundedReceiver<ApiConnectionMode>,
    ) -> Result<Self> {
        Ok(Self {
            initial: initial_connection_mode,
            handle,
            change_rx,
        })
    }
}

impl ConnectionModeProvider for AccessModeConnectionModeProvider {
    fn initial(&self) -> ApiConnectionMode {
        self.initial.clone()
    }

    fn receive(&mut self) -> impl std::future::Future<Output = Option<ApiConnectionMode>> + Send {
        self.change_rx.next()
    }

    fn rotate(&self) -> impl std::future::Future<Output = ()> + Send {
        let handle = self.handle.clone();
        async move {
            handle.rotate().await.ok();
        }
    }
}

/// A small actor which takes care of handling the logic around rotating
/// connection modes to be used for Mullvad API request.
///
/// When `mullvad-api` fails to contact the API, it will request a new
/// connection mode. The API can be connected to either directly (i.e.,
/// [`ApiConnectionMode::Direct`]) via a bridge ([`ApiConnectionMode::Proxied`])
/// or via any supported custom proxy protocol
/// ([`talpid_types::net::proxy::CustomProxy`]).
pub struct AccessModeSelector<B: AccessMethodResolver> {
    #[cfg(feature = "api-override")]
    api_endpoint: ApiEndpoint,
    cmd_rx: mpsc::UnboundedReceiver<Message>,
    bridge_dns_proxy_provider: B,
    access_method_settings: Settings,
    access_method_event_sender: mpsc::UnboundedSender<(AccessMethodEvent, oneshot::Sender<()>)>,
    connection_mode_provider_sender: mpsc::UnboundedSender<ApiConnectionMode>,
    current: ResolvedConnectionMode,
    /// `index` is used to keep track of the [`AccessMethodSetting`] to use.
    index: usize,
}

impl<B: AccessMethodResolver + 'static> AccessModeSelector<B> {
    pub async fn spawn(
        mut bridge_dns_proxy_provider: B,
        #[cfg_attr(not(feature = "api-override"), allow(unused_mut))]
        mut access_method_settings: Settings,
        #[cfg(feature = "api-override")] api_endpoint: ApiEndpoint,
        access_method_event_sender: mpsc::UnboundedSender<(AccessMethodEvent, oneshot::Sender<()>)>,
    ) -> Result<(AccessModeSelectorHandle, AccessModeConnectionModeProvider)> {
        let (cmd_tx, cmd_rx) = mpsc::unbounded();

        #[cfg(feature = "api-override")]
        {
            if api_endpoint.force_direct {
                access_method_settings
                    .update(|setting| setting.is_direct(), |setting| setting.enable());
            }
        }

        // Always start looking from the position of `Direct`.
        let (index, next) = Self::find_next_active(0, &access_method_settings);
        let initial_connection_mode =
            Self::resolve_with_default(&next, &mut bridge_dns_proxy_provider).await;

        let (change_tx, change_rx) = mpsc::unbounded();

        let api_connection_mode = initial_connection_mode.connection_mode.clone();

        let selector = AccessModeSelector {
            #[cfg(feature = "api-override")]
            api_endpoint,
            cmd_rx,
            bridge_dns_proxy_provider,
            access_method_settings,
            access_method_event_sender,
            connection_mode_provider_sender: change_tx,
            current: initial_connection_mode,
            index,
        };

        tokio::spawn(selector.into_future());

        let handle = AccessModeSelectorHandle { cmd_tx };

        let connection_mode_provider =
            AccessModeConnectionModeProvider::new(handle.clone(), api_connection_mode, change_rx)?;

        Ok((handle, connection_mode_provider))
    }

    async fn into_future(mut self) {
        while let Some(cmd) = self.cmd_rx.next().await {
            log::trace!("Processing {cmd} command");
            let execution = match cmd {
                Message::Get(tx) => self.on_get_access_method(tx),
                Message::Use(tx, id) => self.on_use_access_method(tx, id).await,
                Message::Rotate(tx) => self.on_next_connection_mode(tx).await,
                Message::Update(tx, values) => self.on_update_access_methods(tx, values).await,
                Message::Resolve(tx, setting) => self.on_resolve_access_method(tx, setting).await,
            };
            match execution {
                Ok(_) => (),
                Err(error) if error.is_critical_error() => {
                    log::error!(
                        "AccessModeSelector failed due to an internal error and won't be able to recover without a restart. {error}"
                    );
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

    async fn on_use_access_method(&mut self, tx: ResponseTx<()>, id: Id) -> Result<()> {
        self.use_access_method(id).await;
        self.reply(tx, ())
    }

    /// Set and announce the specified access method as the current one.
    async fn use_access_method(&mut self, id: Id) {
        #[cfg(feature = "api-override")]
        {
            if self.api_endpoint.force_direct {
                log::debug!("API proxies are disabled");
                return;
            }
        }

        let Some((index, method)) = self
            .access_method_settings
            .iter()
            .enumerate()
            .find(|(_, access_method)| access_method.get_id() == id)
        else {
            return;
        };

        self.index = index;
        self.set_current(method.to_owned()).await;
    }

    async fn on_next_connection_mode(&mut self, tx: ResponseTx<ApiConnectionMode>) -> Result<()> {
        let next = self.next_connection_mode().await?;
        self.reply(tx, next)
    }

    async fn next_connection_mode(&mut self) -> Result<ApiConnectionMode> {
        #[cfg(feature = "api-override")]
        {
            if self.api_endpoint.force_direct {
                log::debug!("API proxies are disabled");
                return Ok(ApiConnectionMode::Direct);
            }

            log::debug!(
                "The `api-override` feature is enabled, but a direct connection \
                 is not enforced. Selecting API access methods as normal"
            );
        }

        let (next_index, next) =
            Self::find_next_active(self.index + 1, &self.access_method_settings);
        self.index = next_index;
        self.set_current(next).await;
        Ok(self.current.connection_mode.clone())
    }

    async fn set_current(&mut self, access_method: AccessMethodSetting) {
        let resolved =
            Self::resolve_with_default(&access_method, &mut self.bridge_dns_proxy_provider).await;

        // Note: If the daemon is busy waiting for a call to this function
        // to complete while we wait for the daemon to fully handle this
        // `NewAccessMethodEvent`, then we find ourselves in a deadlock.
        // This can happen during daemon startup when spawning a new
        // `MullvadRestHandle`, which will call and await `next` on a Stream
        // created from this `AccessModeSelector` instance. As such, the
        // completion channel is discarded in this instance.
        let setting = resolved.setting.clone();
        #[cfg(not(target_os = "android"))]
        let endpoint = resolved.endpoint.clone();
        let daemon_sender = self.access_method_event_sender.clone();
        let connection_mode = resolved.connection_mode.clone();
        tokio::spawn(async move {
            let _ = AccessMethodEvent::New {
                setting,
                connection_mode,
                #[cfg(not(target_os = "android"))]
                endpoint,
            }
            .send(daemon_sender)
            .await;
        });

        // Notify REST client
        let _ = self
            .connection_mode_provider_sender
            .unbounded_send(resolved.connection_mode.clone());

        self.current = resolved;

        log::info!(
            "A new API access method has been selected: {name}",
            name = self.current.setting.name
        );
    }

    /// Find the next access method to use.
    ///
    /// * `start`: From which point in `access_methods` to start the search.
    /// * `access_methods`: The search space.
    fn find_next_active(start: usize, access_methods: &Settings) -> (usize, AccessMethodSetting) {
        access_methods
            .iter()
            .cloned()
            .enumerate()
            .cycle()
            .skip(start)
            .take(access_methods.cardinality())
            .find(|(_index, access_method)| access_method.enabled())
            .unwrap_or_else(|| (0, access_methods.direct().clone()))
    }

    async fn on_update_access_methods(
        &mut self,
        tx: ResponseTx<()>,
        access_methods: Settings,
    ) -> Result<()> {
        self.update_access_methods(access_methods).await?;
        self.reply(tx, ())
    }

    async fn update_access_methods(&mut self, access_methods: Settings) -> Result<()> {
        self.access_method_settings = access_methods;

        let new_current = self
            .access_method_settings
            .iter()
            .enumerate()
            .find(|(_, access_method)| access_method.get_id() == self.current.setting.get_id());

        match new_current {
            Some((index, new_current)) => {
                // If the current method was modified, announce changes
                self.index = index;
                if self.current.setting != *new_current {
                    if new_current.enabled() {
                        self.set_current(new_current.to_owned()).await;
                    } else {
                        self.next_connection_mode().await?;
                    }
                }
            }
            None => {
                // Current method was removed: A new access method will suddenly have the same index
                // as the one we are removing, but we want it to still be a
                // candidate. A minor hack to achieve this is to simply decrement
                // the current index.
                self.index = self.index.saturating_sub(1);
                self.next_connection_mode().await?;
            }
        }
        Ok(())
    }

    pub async fn on_resolve_access_method(
        &mut self,
        tx: ResponseTx<Option<ResolvedConnectionMode>>,
        setting: AccessMethodSetting,
    ) -> Result<()> {
        let reply = self.resolve(setting).await;
        self.reply(tx, reply)
    }

    async fn resolve(
        &mut self,
        method_setting: AccessMethodSetting,
    ) -> Option<ResolvedConnectionMode> {
        let (endpoint, connection_mode) = self
            .bridge_dns_proxy_provider
            .resolve_access_method_setting(&method_setting.access_method)
            .await?;
        Some(ResolvedConnectionMode {
            connection_mode,
            endpoint,
            setting: method_setting,
        })
    }

    /// Resolve an access method into a set of connection details - fall back to
    /// [`ApiConnectionMode::Direct`] in case `access_method` does not yield anything.
    async fn resolve_with_default(
        method_setting: &AccessMethodSetting,
        bridge_dns_proxy_provider: &mut B,
    ) -> ResolvedConnectionMode {
        let (endpoint, connection_mode) = match bridge_dns_proxy_provider
            .resolve_access_method_setting(&method_setting.access_method)
            .await
        {
            Some(resolved) => resolved,
            None => (
                bridge_dns_proxy_provider.default_connection_mode().await,
                ApiConnectionMode::Direct,
            ),
        };
        ResolvedConnectionMode {
            connection_mode,
            endpoint,
            setting: method_setting.clone(),
        }
    }
}

#[async_trait]
pub trait AccessMethodResolver: Send + Sync {
    async fn resolve_access_method_setting(
        &mut self,
        access_method: &AccessMethod,
    ) -> Option<(AllowedEndpoint, ApiConnectionMode)>;

    async fn default_connection_mode(&self) -> AllowedEndpoint;
}
