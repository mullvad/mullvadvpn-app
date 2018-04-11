use std::net::IpAddr;
use std::sync::{Arc, Mutex, MutexGuard};

error_chain!{
    errors {
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
    /// List configuration's name server IP addresses.
    fn nameservers(&self) -> &Vec<IpAddr>;

    /// Set the configuration's name servers.
    fn set_nameservers(&mut self, nameservers: &Vec<IpAddr>);
}

/// Handles the interface between the cross-platform abstractions and the platform specific
/// operations.
pub trait DnsConfigInterface {
    /// The system DNS configuration type.
    type Config: DnsConfig;

    /// Representation of system DNS update events.
    type Update;

    /// Error type.
    type Error: ::std::error::Error + Send + 'static;

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
pub trait DnsConfigMonitor<I: DnsConfigInterface>: Sized {
    /// Error type.
    type Error: ::std::error::Error + Send + 'static;

    /// Start the monitor, and notify the handler of any updates.
    fn spawn(handler: Arc<Mutex<DnsConfigHandler<I>>>) -> ::std::result::Result<Self, Self::Error>;
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
    fn new(interface: I) -> Self {
        DnsConfigHandler {
            state: None,
            interface,
        }
    }

    /// Notify that the DNS configuration has changed.
    pub fn update(&mut self, update: I::Update) -> Result<()> {
        let config_to_write = if let Some(ref state) = self.state {
            let new_config = self.interface
                .read_update(update)
                .chain_err(|| ErrorKind::ReadDnsUpdate)?;

            if *new_config.nameservers() != state.servers {
                Some(state.config())
            } else {
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

/// Manages the system DNS configuration to keep it in a desired state.
///
/// The DNS configuration is managed through a [`DnsConfigInterface`] type, which provides the
/// necessary platform specific operations. The [`DnsMonitor`] type is used to monitor the
/// configuration for changes so that it is kept in the same desired state.
///
/// [`DnsConfigInterface`]: trait.DnsConfigInterface.html
/// [`DnsConfigMonitor`]: trait.DnsConfigMonitor.html
pub struct DnsConfigManager<I: DnsConfigInterface, M: DnsConfigMonitor<I>> {
    handler: Arc<Mutex<DnsConfigHandler<I>>>,
    _monitor: M,
}

impl<I, M> DnsConfigManager<I, M>
where
    I: DnsConfigInterface,
    M: DnsConfigMonitor<I>,
{
    /// Create a new instance that uses the provided interface to the platform specific DNS
    /// configuration system.
    pub fn new(interface: I) -> Result<Self> {
        let handler = Arc::new(Mutex::new(DnsConfigHandler::new(interface)));
        let monitor = M::spawn(handler.clone()).chain_err(|| ErrorKind::SpawnDnsMonitor)?;

        Ok(Self {
            handler,
            _monitor: monitor,
        })
    }

    /// Applies a desired configuration.
    pub fn configure(&self, servers: Vec<IpAddr>) -> Result<()> {
        self.lock_handler().configure(servers)
    }

    /// Restores to the original configuration.
    pub fn restore(&self) -> Result<()> {
        self.lock_handler().restore()
    }

    fn lock_handler(&self) -> MutexGuard<DnsConfigHandler<I>> {
        self.handler
            .lock()
            .expect("a thread panicked while using the DNS configuration handler")
    }
}
