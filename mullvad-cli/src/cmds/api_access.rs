use anyhow::{anyhow, Result};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::access_method::{AccessMethod, AccessMethodSetting, CustomAccessMethod};
use std::net::IpAddr;

use clap::{Args, Subcommand};
use talpid_types::net::{openvpn::SHADOWSOCKS_CIPHERS, TransportProtocol};

#[derive(Subcommand, Debug, Clone)]
pub enum ApiAccess {
    /// Display the current API access method.
    Get,
    /// Add a custom API access method
    #[clap(subcommand)]
    Add(AddCustomCommands),
    /// Lists all API access methods
    ///
    /// * = Enabled
    List,
    /// Edit a custom API access method
    Edit(EditCustomCommands),
    /// Remove a custom API access method
    Remove(SelectItem),
    /// Enable an API access method
    Enable(SelectItem),
    /// Disable an API access method
    Disable(SelectItem),
    /// Try to use a specific API access method (If the API is unreachable, reverts back to the previous access method)
    ///
    /// Selecting "Direct" will connect to the Mullvad API without going through any proxy. This connection use https and is therefore encrypted.
    ///
    /// Selecting "Mullvad Bridges" respects your current bridge settings
    Use(SelectItem),
    /// Try to reach the Mullvad API using a specific access method
    Test(SelectItem),
}

impl ApiAccess {
    pub async fn handle(self) -> Result<()> {
        match self {
            ApiAccess::List => {
                Self::list().await?;
            }
            ApiAccess::Add(cmd) => {
                Self::add(cmd).await?;
            }
            ApiAccess::Edit(cmd) => Self::edit(cmd).await?,
            ApiAccess::Remove(cmd) => Self::remove(cmd).await?,
            ApiAccess::Enable(cmd) => {
                Self::enable(cmd).await?;
            }
            ApiAccess::Disable(cmd) => {
                Self::disable(cmd).await?;
            }
            ApiAccess::Test(cmd) => {
                Self::test(cmd).await?;
            }
            ApiAccess::Use(cmd) => {
                Self::set(cmd).await?;
            }
            ApiAccess::Get => {
                Self::get().await?;
            }
        };
        Ok(())
    }

    /// Show all API access methods.
    async fn list() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        for (index, api_access_method) in rpc.get_api_access_methods().await?.iter().enumerate() {
            println!(
                "{}. {}",
                index + 1,
                pp::ApiAccessMethodFormatter::new(api_access_method)
            );
        }
        Ok(())
    }

    /// Add a custom API access method.
    async fn add(cmd: AddCustomCommands) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let name = cmd.name().to_string();
        let enabled = cmd.enabled();
        let access_method = AccessMethod::try_from(cmd)?;
        rpc.add_access_method(name, enabled, access_method).await?;
        Ok(())
    }

    /// Remove an API access method.
    async fn remove(cmd: SelectItem) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let access_method = Self::get_access_method(&mut rpc, &cmd).await?;
        rpc.remove_access_method(access_method.get_id())
            .await
            .map_err(Into::<anyhow::Error>::into)
    }

    /// Edit the data of an API access method.
    async fn edit(cmd: EditCustomCommands) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let mut api_access_method = Self::get_access_method(&mut rpc, &cmd.item).await?;

        // Create a new access method combining the new params with the previous values
        let access_method = match api_access_method.as_custom() {
            None => return Err(anyhow!("Can not edit built-in access method")),
            Some(x) => match x.clone() {
                CustomAccessMethod::Shadowsocks(shadowsocks) => {
                    let ip = cmd.params.ip.unwrap_or(shadowsocks.peer.ip()).to_string();
                    let port = cmd.params.port.unwrap_or(shadowsocks.peer.port());
                    let password = cmd.params.password.unwrap_or(shadowsocks.password);
                    let cipher = cmd.params.cipher.unwrap_or(shadowsocks.cipher);
                    mullvad_types::access_method::Shadowsocks::from_args(ip, port, cipher, password)
                        .map(AccessMethod::from)
                }
                CustomAccessMethod::Socks5(socks) => match socks {
                    mullvad_types::access_method::Socks5::Local(local) => {
                        let ip = cmd.params.ip.unwrap_or(local.peer.address.ip()).to_string();
                        let port = cmd.params.port.unwrap_or(local.peer.address.port());
                        let transport_protocol =
                            cmd.params.transport.unwrap_or(local.peer.protocol);
                        let local_port = cmd.params.local_port.unwrap_or(local.port);
                        mullvad_types::access_method::Socks5Local::from_args(
                            ip,
                            port,
                            local_port,
                            transport_protocol,
                        )
                        .map(AccessMethod::from)
                    }
                    mullvad_types::access_method::Socks5::Remote(remote) => {
                        let ip = cmd.params.ip.unwrap_or(remote.peer.ip()).to_string();
                        let port = cmd.params.port.unwrap_or(remote.peer.port());
                        match remote.authentication {
                            None => mullvad_types::access_method::Socks5Remote::from_args(ip, port),
                            Some(mullvad_types::access_method::SocksAuth {
                                username,
                                password,
                            }) => {
                                let username = cmd.params.username.unwrap_or(username);
                                let password = cmd.params.password.unwrap_or(password);
                                mullvad_types::access_method::Socks5Remote::from_args_with_password(
                                    ip, port, username, password,
                                )
                            }
                        }
                        .map(AccessMethod::from)
                    }
                },
            },
        };

        if let Some(name) = cmd.params.name {
            api_access_method.name = name;
        };
        if let Some(access_method) = access_method {
            api_access_method.access_method = access_method;
        }

        rpc.update_access_method(api_access_method).await?;

        Ok(())
    }

    /// Enable a custom API access method.
    async fn enable(item: SelectItem) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let mut access_method = Self::get_access_method(&mut rpc, &item).await?;
        access_method.enable();
        rpc.update_access_method(access_method).await?;
        Ok(())
    }

    /// Disable a custom API access method.
    async fn disable(item: SelectItem) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let mut access_method = Self::get_access_method(&mut rpc, &item).await?;
        access_method.disable();
        rpc.update_access_method(access_method).await?;
        Ok(())
    }

    /// Test an access method to see if it successfully reaches the Mullvad API.
    async fn test(item: SelectItem) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        // Retrieve the currently used access method. We will reset to this
        // after we are done testing.
        let previous_access_method = rpc.get_current_api_access_method().await?;
        let access_method = Self::get_access_method(&mut rpc, &item).await?;

        println!("Testing access method \"{}\"", access_method.name);
        rpc.set_access_method(access_method.get_id()).await?;
        // Make the daemon perform an network request which involves talking to the Mullvad API.
        let result = match rpc.get_api_addresses().await {
            Ok(_) => {
                println!("Success!");
                Ok(())
            }
            Err(_) => Err(anyhow!("Could not reach the Mullvad API")),
        };
        // In any case, switch back to the previous access method.
        rpc.set_access_method(previous_access_method.get_id())
            .await?;
        result
    }

    /// Try to use of a specific [`AccessMethodSetting`] for subsequent calls to
    /// the Mullvad API.
    ///
    /// First, a test will be performed to check that the new
    /// [`AccessMethodSetting`] is able to reach the API. If it can, the daemon
    /// will set this [`AccessMethodSetting`] to be used by the API runtime.
    ///
    /// If the new [`AccessMethodSetting`] fails, the daemon will perform a
    /// roll-back to the previously used [`AccessMethodSetting`]. If that never
    /// worked, or has recently stopped working, the daemon will start to
    /// automatically try to find a working [`AccessMethodSetting`] among the
    /// configured ones.
    async fn set(item: SelectItem) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let previous_access_method = rpc.get_current_api_access_method().await?;
        let mut new_access_method = Self::get_access_method(&mut rpc, &item).await?;
        // Try to reach the API with the newly selected access method.
        rpc.set_access_method(new_access_method.get_id()).await?;
        match rpc.get_api_addresses().await {
            Ok(_) => (),
            Err(_) => {
                // Roll-back to the previous access method
                rpc.set_access_method(previous_access_method.get_id())
                    .await?;
                return Err(anyhow!(
                    "Could not reach the Mullvad API using access method \"{}\". Rolling back to \"{}\"",
                    new_access_method.get_name(),
                    previous_access_method.get_name()
                ));
            }
        };
        // It worked! Let the daemon keep using this access method.
        let display_name = new_access_method.get_name();
        // Toggle the enabled status if needed
        if !new_access_method.enabled() {
            new_access_method.enable();
            rpc.update_access_method(new_access_method).await?;
        }
        println!("Using access method \"{}\"", display_name);
        Ok(())
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let current = rpc.get_current_api_access_method().await?;
        let mut access_method_formatter = pp::ApiAccessMethodFormatter::new(&current);
        access_method_formatter.settings.write_enabled = false;
        println!("{}", access_method_formatter);
        Ok(())
    }

    async fn get_access_method(
        rpc: &mut MullvadProxyClient,
        item: &SelectItem,
    ) -> Result<AccessMethodSetting> {
        rpc.get_api_access_methods()
            .await?
            .get(item.as_array_index()?)
            .cloned()
            .ok_or(anyhow!(format!("Access method {} does not exist", item)))
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum AddCustomCommands {
    /// Configure a SOCKS5 proxy
    #[clap(subcommand)]
    Socks5(AddSocks5Commands),
    /// Configure a custom Shadowsocks proxy to use as an API access method
    Shadowsocks {
        /// An easy to remember name for this custom proxy
        name: String,
        /// The IP of the remote Shadowsocks-proxy
        remote_ip: IpAddr,
        /// Port on which the remote Shadowsocks-proxy listens for traffic
        remote_port: u16,
        /// Password for authentication
        password: String,
        /// Cipher to use
        #[arg(long, value_parser = SHADOWSOCKS_CIPHERS)]
        cipher: String,
        /// Disable the use of this custom access method. It has to be manually
        /// enabled at a later stage to be used when accessing the Mullvad API.
        #[arg(default_value_t = false, short, long)]
        disabled: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum AddSocks5Commands {
    /// Configure a remote SOCKS5 proxy
    Remote {
        /// An easy to remember name for this custom proxy
        name: String,
        /// IP of the remote SOCKS5-proxy
        remote_ip: IpAddr,
        /// Port on which the remote SOCKS5-proxy listens for traffic
        remote_port: u16,
        #[clap(flatten)]
        authentication: Option<SocksAuthentication>,
        /// Disable the use of this custom access method. It has to be manually
        /// enabled at a later stage to be used when accessing the Mullvad API.
        #[arg(default_value_t = false, short, long)]
        disabled: bool,
    },
    /// Configure a local SOCKS5 proxy
    Local {
        /// An easy to remember name for this custom proxy
        name: String,
        /// The port that the server on localhost is listening on
        local_port: u16,
        /// The IP of the remote peer
        remote_ip: IpAddr,
        /// The port of the remote peer
        remote_port: u16,
        /// The Mullvad App can not know which transport protocol that the
        /// remote peer accepts, but it needs to know this in order to correctly
        /// exempt the connection traffic in the firewall.
        ///
        /// By default, the transport protocol is assumed to be `TCP`, but it
        /// can optionally be set to `UDP` as well.
        #[arg(long, default_value_t = TransportProtocol::Tcp)]
        transport: TransportProtocol,
        /// Disable the use of this custom access method. It has to be manually
        /// enabled at a later stage to be used when accessing the Mullvad API.
        #[arg(default_value_t = false, short, long)]
        disabled: bool,
    },
}

#[derive(Args, Debug, Clone)]
pub struct SocksAuthentication {
    /// Username for authentication against a remote SOCKS5 proxy
    #[arg(short, long)]
    username: String,
    /// Password for authentication against a remote SOCKS5 proxy
    #[arg(short, long)]
    password: String,
}

impl AddCustomCommands {
    fn name(&self) -> &str {
        match self {
            AddCustomCommands::Shadowsocks { name, .. }
            | AddCustomCommands::Socks5(AddSocks5Commands::Remote { name, .. })
            | AddCustomCommands::Socks5(AddSocks5Commands::Local { name, .. }) => name,
        }
    }

    fn enabled(&self) -> bool {
        match self {
            AddCustomCommands::Shadowsocks { disabled, .. }
            | AddCustomCommands::Socks5(AddSocks5Commands::Remote { disabled, .. })
            | AddCustomCommands::Socks5(AddSocks5Commands::Local { disabled, .. }) => !disabled,
        }
    }
}

/// A minimal wrapper type allowing the user to supply a list index to some
/// Access Method.
#[derive(Args, Debug, Clone)]
pub struct SelectItem {
    /// Which access method to pick
    index: usize,
}

impl SelectItem {
    /// Transform human-readable (1-based) index to 0-based indexing.
    pub fn as_array_index(&self) -> Result<usize> {
        self.index
            .checked_sub(1)
            .ok_or(anyhow!("Access method 0 does not exist"))
    }
}

impl std::fmt::Display for SelectItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.index)
    }
}

#[derive(Args, Debug, Clone)]
pub struct EditCustomCommands {
    /// Which API access method to edit
    #[clap(flatten)]
    item: SelectItem,
    /// Editing parameters
    #[clap(flatten)]
    params: EditParams,
}

#[derive(Args, Debug, Clone)]
pub struct EditParams {
    /// Name of the API access method in the Mullvad client [All]
    #[arg(long)]
    name: Option<String>,
    /// Username for authentication [Socks5 (Remote proxy)]
    #[arg(long)]
    username: Option<String>,
    /// Password for authentication [Socks5 (Remote proxy), Shadowsocks]
    #[arg(long)]
    password: Option<String>,
    /// Cipher to use [Shadowsocks]
    #[arg(value_parser = SHADOWSOCKS_CIPHERS, long)]
    cipher: Option<String>,
    /// The IP of the remote proxy server [Socks5 (Local & Remote proxy), Shadowsocks]
    #[arg(long)]
    ip: Option<IpAddr>,
    /// The port of the remote proxy server [Socks5 (Local & Remote proxy), Shadowsocks]
    #[arg(long)]
    port: Option<u16>,
    /// The transport protocol used when communicating with the remote proxy server [Socks5 (Local)]
    #[arg(long)]
    transport: Option<TransportProtocol>,
    /// The port that the server on localhost is listening on [Socks5 (Local proxy)]
    #[arg(long)]
    local_port: Option<u16>,
}

/// Implement conversions from CLI types to Daemon types.
///
/// Since these are not supposed to be used outside of the CLI,
/// we define them in a hidden-away module.
mod conversions {
    use anyhow::{anyhow, Error};
    use mullvad_types::access_method as daemon_types;

    use super::{AddCustomCommands, AddSocks5Commands, SocksAuthentication};

    impl TryFrom<AddCustomCommands> for daemon_types::AccessMethod {
        type Error = Error;

        fn try_from(value: AddCustomCommands) -> Result<Self, Self::Error> {
            Ok(match value {
                AddCustomCommands::Socks5(socks) => match socks {
                    AddSocks5Commands::Local {
                        local_port,
                        remote_ip,
                        remote_port,
                        name: _,
                        disabled: _,
                        transport: remote_transport_protocol,
                    } => {
                        println!("Adding SOCKS5-proxy: localhost:{local_port} => {remote_ip}:{remote_port}/{remote_transport_protocol}");
                        daemon_types::Socks5Local::from_args(
                            remote_ip.to_string(),
                            remote_port,
                            local_port,
                            remote_transport_protocol,
                        )
                        .map(daemon_types::Socks5::Local)
                        .map(daemon_types::AccessMethod::from)
                        .ok_or(anyhow!("Could not create a local Socks5 access method"))?
                    }
                    AddSocks5Commands::Remote {
                        remote_ip,
                        remote_port,
                        authentication,
                        name: _,
                        disabled: _,
                    } => {
                        match authentication {
                            Some(SocksAuthentication { username, password }) => {
                                println!("Adding SOCKS5-proxy: {username}:{password}@{remote_ip}:{remote_port}");
                                daemon_types::Socks5Remote::from_args_with_password(
                                    remote_ip.to_string(),
                                    remote_port,
                                    username,
                                    password
                                )
                            }
                            None => {
                                println!("Adding SOCKS5-proxy: {remote_ip}:{remote_port}");
                                daemon_types::Socks5Remote::from_args(
                                    remote_ip.to_string(),
                                    remote_port,
                                )
                            }
                        }
                        .map(daemon_types::Socks5::Remote)
                        .map(daemon_types::AccessMethod::from)
                        .ok_or(anyhow!("Could not create a remote Socks5 access method"))?
                    }
                },
                AddCustomCommands::Shadowsocks {
                    remote_ip,
                    remote_port,
                    password,
                    cipher,
                    name: _,
                    disabled: _,
                } => {
                    println!(
                "Adding Shadowsocks-proxy: {password} @ {remote_ip}:{remote_port} using {cipher}"
                    );
                    daemon_types::Shadowsocks::from_args(
                        remote_ip.to_string(),
                        remote_port,
                        cipher,
                        password,
                    )
                    .map(daemon_types::AccessMethod::from)
                    .ok_or(anyhow!("Could not create a Shadowsocks access method"))?
                }
            })
        }
    }
}

/// Pretty printing of [`ApiAccessMethod`]s
mod pp {
    use mullvad_types::access_method::{
        AccessMethod, AccessMethodSetting, CustomAccessMethod, Socks5, SocksAuth,
    };

    pub struct ApiAccessMethodFormatter<'a> {
        api_access_method: &'a AccessMethodSetting,
        pub settings: FormatterSettings,
    }

    pub struct FormatterSettings {
        /// If the formatter should print the enabled status of an
        /// [`AcessMethodSetting`] (*) next to its name.
        pub write_enabled: bool,
    }

    impl Default for FormatterSettings {
        fn default() -> Self {
            Self {
                write_enabled: true,
            }
        }
    }

    impl<'a> ApiAccessMethodFormatter<'a> {
        pub fn new(api_access_method: &'a AccessMethodSetting) -> ApiAccessMethodFormatter<'a> {
            ApiAccessMethodFormatter {
                api_access_method,
                settings: Default::default(),
            }
        }
    }

    impl<'a> std::fmt::Display for ApiAccessMethodFormatter<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use crate::print_option;

            let write_status = |f: &mut std::fmt::Formatter<'_>, enabled: bool| {
                if enabled {
                    write!(f, " *")
                } else {
                    write!(f, "")
                }
            };

            match &self.api_access_method.access_method {
                AccessMethod::BuiltIn(method) => {
                    write!(f, "{}", method.canonical_name())?;
                    if self.settings.write_enabled {
                        write_status(f, self.api_access_method.enabled())?;
                    }
                    Ok(())
                }
                AccessMethod::Custom(method) => match &method {
                    CustomAccessMethod::Shadowsocks(shadowsocks) => {
                        write!(f, "{}", self.api_access_method.get_name())?;
                        if self.settings.write_enabled {
                            write_status(f, self.api_access_method.enabled())?;
                        }
                        writeln!(f)?;
                        print_option!("Protocol", format!("Shadowsocks [{}]", shadowsocks.cipher));
                        print_option!("Peer", shadowsocks.peer);
                        print_option!("Password", shadowsocks.password);
                        Ok(())
                    }
                    CustomAccessMethod::Socks5(socks) => match socks {
                        Socks5::Remote(remote) => {
                            write!(f, "{}", self.api_access_method.get_name())?;
                            if self.settings.write_enabled {
                                write_status(f, self.api_access_method.enabled())?;
                            }
                            writeln!(f)?;
                            print_option!("Protocol", "Socks5");
                            print_option!("Peer", remote.peer);
                            match &remote.authentication {
                                Some(SocksAuth { username, password }) => {
                                    print_option!("Username", username);
                                    print_option!("Password", password);
                                }
                                None => (),
                            }
                            Ok(())
                        }
                        Socks5::Local(local) => {
                            write!(f, "{}", self.api_access_method.get_name())?;
                            if self.settings.write_enabled {
                                write_status(f, self.api_access_method.enabled())?;
                            }
                            writeln!(f)?;
                            print_option!("Protocol", "Socks5 (local)");
                            print_option!("Peer", local.peer);
                            print_option!("Local port", local.port);
                            Ok(())
                        }
                    },
                },
            }
        }
    }
}
