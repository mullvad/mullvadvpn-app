use std::net::IpAddr;
use std::sync::mpsc;
use std::thread;

use error_chain::ChainedError;

error_chain!{
    errors {
        /// Failure to open DNS configuration interface.
        OpenInterface {
            description("failed to open the DNS configuration interface")
        }

        /// DNS configuration handler thread stopped unexpectedly.
        HandlerStopped {
            description("DNS configuration handler thread has stopped unexpectedly")
        }

        /// Failure to read DNS configuration.
        ReadDnsConfig {
            description("failed to read current DNS configuration")
        }

        /// Failure to read DNS configuration change.
        ReadDnsUpdate {
            description("failed to read updated DNS configuration")
        }

        /// Failure to read spawn DNS configuration monitor.
        SpawnDnsMonitor {
            description("failed to spawn DNS configuration monitor")
        }

        /// Failure to apply a DNS configuration.
        WriteDnsConfig {
            description("failed to change DNS configuration")
        }
    }
}

/// Represents a system-specific valid DNS configuration.
///
/// The only thing that matters for this module is how to read the configured name server IP
/// addresses and how to change the them.
pub trait DnsConfig: Clone {
    /// Checks if the configuration uses only the specified nameservers.
    fn uses_nameservers(&self, nameservers: &Vec<IpAddr>) -> bool;

    /// Set the configuration's name servers.
    fn set_nameservers(&mut self, nameservers: &Vec<IpAddr>);

    /// Merges with another configuration.
    ///
    /// This is system specific behavior, but the point is to allow the current configuration to
    /// collect new information from a newer configuration.
    fn merge_with(&mut self, other: Self);

    /// Merges with another configuration ignoring the other configuration's name server list.
    ///
    /// This is similar to the [`merge_with`] method, but here the point is to collect new
    /// information that does not include the list of name servers.
    ///
    /// [`merge_with`]: #method.merge_with
    fn merge_ignoring_nameservers(&mut self, other: Self);
}

/// Handles the interface between the cross-platform abstractions and the platform specific
/// operations.
///
/// A type implementing this interface does not need to implement `Send`.
pub trait DnsConfigInterface: Sized {
    /// The system DNS configuration type.
    type Config: DnsConfig;

    /// Representation of system DNS update events.
    type Update: Send + 'static;

    /// Error type.
    type Error: ::std::error::Error + Send + 'static;

    /// Create a new instance of the interface.
    fn open() -> ::std::result::Result<Self, Self::Error>;

    /// Read current DNS configuration.
    fn read_config(&mut self) -> ::std::result::Result<Self::Config, Self::Error>;

    /// Read DNS configuration for the received update event.
    fn read_update(
        &mut self,
        update: Self::Update,
    ) -> ::std::result::Result<Self::Config, Self::Error>;

    /// Change the system DNS configuration.
    fn write_config(&mut self, config: Self::Config) -> ::std::result::Result<(), Self::Error>;
}

/// System specific type that monitors DNS configuration changes.
///
/// An implementing type should implement the `spawn` method by starting to watch for system DNS
/// changes and notifying the given handler when a change is detected.
///
/// Monitoring should stop when the type is dropped.
pub trait DnsConfigMonitor<U: Send>: Sized {
    /// Error type.
    type Error: ::std::error::Error + Send + 'static;

    /// Start the monitor, and notify the handler of any updates.
    fn spawn(update_events: UpdateSender<U>) -> ::std::result::Result<Self, Self::Error>;
}

struct DnsState<C: DnsConfig> {
    backup: C,
    servers: Vec<IpAddr>,
}

impl<C> DnsState<C>
where
    C: DnsConfig,
{
    fn config(&self) -> C {
        let mut config = self.backup.clone();
        config.set_nameservers(&self.servers);
        config
    }
}

/// Handler for DNS changes.
///
/// Two types of events are handled: external requests to change or restore the configuration and
/// system updates. The public interface only allows system updates to be sent. External requests
/// should be made through a [`DnsConfigManager`].
///
/// An internal state is kept that tracks what is the current desired configuration and the
/// previous configuration, so that it can be restored when requested.
///
/// [`DnsConfigManager`]: struct.DnsConfigManager.html
pub struct DnsConfigHandler<I: DnsConfigInterface> {
    state: Option<DnsState<I::Config>>,
    interface: I,
}

impl<I> DnsConfigHandler<I>
where
    I: DnsConfigInterface,
{
    fn new() -> Result<Self> {
        Ok(DnsConfigHandler {
            state: None,
            interface: I::open().chain_err(|| ErrorKind::OpenInterface)?,
        })
    }

    fn configure(&mut self, servers: Vec<IpAddr>) -> Result<()> {
        let state = match self.state.take() {
            Some(existing_state) => DnsState {
                backup: existing_state.backup,
                servers,
            },
            None => DnsState {
                backup: self.interface
                    .read_config()
                    .chain_err(|| ErrorKind::ReadDnsConfig)?,
                servers,
            },
        };

        self.write_config(state.config())?;
        self.state = Some(state);

        Ok(())
    }

    fn update(&mut self, update: I::Update) -> Result<()> {
        let config_to_write = if let Some(ref mut state) = self.state {
            let current_config = state.config();
            let new_config = self.interface
                .read_update(update)
                .chain_err(|| ErrorKind::ReadDnsUpdate)?;

            if !new_config.uses_nameservers(&state.servers) {
                state.backup.merge_with(new_config);
                Some(current_config)
            } else {
                state.backup.merge_ignoring_nameservers(new_config);
                None
            }
        } else {
            None
        };

        if let Some(config) = config_to_write {
            self.write_config(config)
        } else {
            Ok(())
        }
    }

    fn restore(&mut self) -> Result<()> {
        if let Some(state) = self.state.take() {
            self.write_config(state.backup)
        } else {
            Ok(())
        }
    }

    fn write_config(&mut self, config: I::Config) -> Result<()> {
        self.interface
            .write_config(config)
            .chain_err(|| ErrorKind::WriteDnsConfig)
    }
}

enum DnsConfigEvent<U: Send> {
    Configure(Vec<IpAddr>, mpsc::Sender<Result<()>>),
    Restore(mpsc::Sender<Result<()>>),
    Update(U),
}

/// Wraps an `mpsc::Sender` to convert update events into DNS configuration events.
pub struct UpdateSender<U: Send> {
    sender: mpsc::Sender<DnsConfigEvent<U>>,
}

impl<U: Send + 'static> UpdateSender<U> {
    fn new(sender: mpsc::Sender<DnsConfigEvent<U>>) -> Self {
        UpdateSender { sender }
    }

    pub fn send(&mut self, event: U) -> Result<()> {
        self.sender
            .send(DnsConfigEvent::Update(event))
            .chain_err(|| ErrorKind::HandlerStopped)
    }
}

/// Manages the system DNS configuration to keep it in a desired state.
///
/// The DNS configuration is managed through a [`DnsConfigInterface`] type, which provides the
/// necessary platform specific operations. The [`DnsMonitor`] type is used to monitor the
/// configuration for changes so that it is kept in the same desired state.
///
/// [`DnsConfigInterface`]: trait.DnsConfigInterface.html
/// [`DnsConfigMonitor`]: trait.DnsConfigMonitor.html
pub struct DnsConfigManager<I, M>
where
    I: DnsConfigInterface,
    M: DnsConfigMonitor<I::Update>,
{
    handler: mpsc::Sender<DnsConfigEvent<I::Update>>,
    _monitor: M,
}

impl<I, M> DnsConfigManager<I, M>
where
    I: DnsConfigInterface,
    M: DnsConfigMonitor<I::Update>,
{
    /// Create a new instance that uses the provided interface to the platform specific DNS
    /// configuration system.
    pub fn spawn() -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel();
        let handler = event_tx.clone();
        let monitor =
            M::spawn(UpdateSender::new(event_tx)).chain_err(|| ErrorKind::SpawnDnsMonitor)?;

        Self::spawn_handler_thread(event_rx)?;

        Ok(Self {
            handler,
            _monitor: monitor,
        })
    }

    /// Applies a desired configuration.
    pub fn configure(&self, servers: Vec<IpAddr>) -> Result<()> {
        let (reply_tx, reply_rx) = mpsc::channel();

        self.handler
            .send(DnsConfigEvent::Configure(servers, reply_tx))
            .chain_err(|| ErrorKind::HandlerStopped)?;

        reply_rx.recv().chain_err(|| ErrorKind::HandlerStopped)?
    }

    /// Restores to the original configuration.
    pub fn restore(&self) -> Result<()> {
        let (reply_tx, reply_rx) = mpsc::channel();

        self.handler
            .send(DnsConfigEvent::Restore(reply_tx))
            .chain_err(|| ErrorKind::HandlerStopped)?;

        reply_rx.recv().chain_err(|| ErrorKind::HandlerStopped)?
    }

    fn spawn_handler_thread(event_rx: mpsc::Receiver<DnsConfigEvent<I::Update>>) -> Result<()> {
        let (handler_result_tx, handler_result_rx) = mpsc::channel();

        thread::spawn(move || match DnsConfigHandler::new() {
            Ok(handler) => {
                let _ = handler_result_tx.send(Ok(()));
                Self::run_handler_thread(handler, event_rx);
            }
            Err(error) => {
                let _ = handler_result_tx.send(Err(error));
            }
        });

        handler_result_rx
            .recv()
            .chain_err(|| ErrorKind::HandlerStopped)?
    }

    fn run_handler_thread(
        mut handler: DnsConfigHandler<I>,
        events: mpsc::Receiver<DnsConfigEvent<I::Update>>,
    ) {
        use self::DnsConfigEvent::*;

        for event in events {
            match event {
                Configure(servers, reply) => {
                    let _ = reply.send(handler.configure(servers));
                }
                Restore(reply) => {
                    let _ = reply.send(handler.restore());
                }
                Update(update) => {
                    if let Err(error) = handler.update(update) {
                        error!(
                            "Failed to reconfigure DNS settings: {}",
                            error.display_chain()
                        );
                    }
                }
            }
        }
    }
}
