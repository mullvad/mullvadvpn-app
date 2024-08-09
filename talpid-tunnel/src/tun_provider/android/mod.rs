mod ipnetwork_sub;

use self::ipnetwork_sub::IpNetworkSub;
use super::TunConfig;
use ipnetwork::IpNetwork;
use jnix::{
    jni::{
        objects::{GlobalRef, JValue},
        signature::{JavaType, Primitive},
        JavaVM,
    },
    FromJava, IntoJava, JnixEnv,
};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    os::unix::io::{AsRawFd, RawFd},
    sync::Arc,
};
use talpid_types::{android::AndroidContext, ErrorExt};

/// Errors that occur while setting up VpnService tunnel.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to attach Java VM to tunnel thread")]
    AttachJvmToThread(#[source] jnix::jni::errors::Error),

    #[error("Failed to allow socket to bypass tunnel")]
    Bypass,

    #[error("Failed to call Java method TalpidVpnService.{0}")]
    CallMethod(&'static str, #[source] jnix::jni::errors::Error),

    #[error("Failed to create Java VM handle clone")]
    CloneJavaVm(#[source] jnix::jni::errors::Error),

    #[error("Failed to find TalpidVpnService.{0} method")]
    FindMethod(&'static str, #[source] jnix::jni::errors::Error),

    #[error("Attempt to configure the tunnel with an invalid DNS server address(es): {0:?}")]
    InvalidDnsServers(Vec<IpAddr>),

    #[error("Received an invalid result from TalpidVpnService.{0}: {1}")]
    InvalidMethodResult(&'static str, String),

    #[error("Failed to create tunnel device")]
    TunnelDeviceError,

    #[error("Permission denied when trying to create tunnel")]
    PermissionDenied,
}

/// Factory of tunnel devices on Android.
pub struct AndroidTunProvider {
    jvm: Arc<JavaVM>,
    class: GlobalRef,
    object: GlobalRef,
    last_tun_config: Option<(TunConfig, bool)>,
    allow_lan: bool,
    custom_dns_servers: Option<Vec<IpAddr>>,
    allowed_lan_networks: Vec<IpNetwork>,
    excluded_packages: Vec<String>,
}

impl AndroidTunProvider {
    /// Create a new AndroidTunProvider interfacing with Android's VpnService.
    pub fn new(
        context: AndroidContext,
        allow_lan: bool,
        custom_dns_servers: Option<Vec<IpAddr>>,
        allowed_lan_networks: Vec<IpNetwork>,
        excluded_packages: Vec<String>,
    ) -> Self {
        let env = JnixEnv::from(
            context
                .jvm
                .attach_current_thread_as_daemon()
                .expect("Failed to attach thread to Java VM"),
        );
        let talpid_vpn_service_class = env.get_class("net/mullvad/talpid/TalpidVpnService");

        AndroidTunProvider {
            jvm: context.jvm,
            class: talpid_vpn_service_class,
            object: context.vpn_service,
            last_tun_config: None,
            allow_lan,
            custom_dns_servers,
            allowed_lan_networks,
            excluded_packages,
        }
    }

    pub fn set_allow_lan(&mut self, allow_lan: bool) -> Result<(), Error> {
        if self.allow_lan != allow_lan {
            self.allow_lan = allow_lan;
            self.recreate_tun_if_open()?;
        }

        Ok(())
    }

    pub fn set_dns_servers(&mut self, servers: Option<Vec<IpAddr>>) -> Result<(), Error> {
        if self.custom_dns_servers != servers {
            self.custom_dns_servers = servers;
            self.recreate_tun_if_open()?;
        }

        Ok(())
    }

    /// Update the set of excluded paths (split tunnel apps) for the tunnel provider.
    /// This will cause any pre-existing tunnel to be recreated if necessary. See
    /// [`AndroidTunProvider::recreate_tun_if_open()`] for details.
    pub fn set_exclude_apps(&mut self, excluded_packages: Vec<String>) -> Result<(), Error> {
        if self.excluded_packages != excluded_packages {
            self.excluded_packages = excluded_packages;
            self.recreate_tun_if_open()?;
        }
        Ok(())
    }

    /// Retrieve a tunnel device with the provided configuration. Custom DNS and LAN routes are
    /// appended to the provided config.
    pub fn get_tun(&mut self, config: TunConfig) -> Result<VpnServiceTun, Error> {
        self.get_tun_inner(config, false)
    }

    /// Retrieve a tunnel device with the provided configuration.
    fn get_tun_inner(&mut self, config: TunConfig, blocking: bool) -> Result<VpnServiceTun, Error> {
        let service_config = VpnServiceConfig::new(
            config.clone(),
            &self.allowed_lan_networks,
            self.allow_lan,
            if !blocking {
                self.custom_dns_servers.clone()
            } else {
                // Disable DNS
                Some(vec![])
            },
            self.excluded_packages.clone(),
        );

        let tun_fd = self.get_tun_fd(service_config)?;

        self.last_tun_config = Some((config, blocking));

        let jvm = unsafe { JavaVM::from_raw(self.jvm.get_java_vm_pointer()) }
            .map_err(Error::CloneJavaVm)?;

        Ok(VpnServiceTun {
            tunnel: tun_fd,
            jvm,
            class: self.class.clone(),
            object: self.object.clone(),
        })
    }

    /// Open a tunnel device using the previous or the default configuration.
    ///
    /// Will open a new tunnel if there is already an active tunnel. The previous tunnel will be
    /// closed.
    pub fn create_tun(&mut self) -> Result<(), Error> {
        let result = self.call_method(
            "createTun",
            "()V",
            JavaType::Primitive(Primitive::Void),
            &[],
        )?;

        match result {
            JValue::Void => Ok(()),
            value => Err(Error::InvalidMethodResult(
                "createTun",
                format!("{:?}", value),
            )),
        }
    }

    fn get_tun_fd(&self, config: VpnServiceConfig) -> Result<RawFd, Error> {
        let env = self.env()?;
        let java_config = config.into_java(&env);

        let result = self.call_method(
            "getTun",
            "(Lnet/mullvad/talpid/model/TunConfig;)Lnet/mullvad/talpid/model/CreateTunResult;",
            JavaType::Object("net/mullvad/talpid/model/CreateTunResult".to_owned()),
            &[JValue::Object(java_config.as_obj())],
        )?;

        match result {
            JValue::Object(result) => CreateTunResult::from_java(&env, result).into(),
            value => Err(Error::InvalidMethodResult("getTun", format!("{:?}", value))),
        }
    }

    /// Open a tunnel device that routes everything but (potentially) LAN routes via the tunnel
    /// device. Excluded apps will also be kept.
    ///
    /// Will open a new tunnel if there is already an active tunnel. The previous tunnel will be
    /// closed.
    pub fn create_blocking_tun(&mut self) -> Result<(), Error> {
        let _ = self.get_tun_inner(TunConfig::default(), true)?;
        Ok(())
    }

    /// Close currently active tunnel device.
    pub fn close_tun(&mut self) {
        let result = self.call_method("closeTun", "()V", JavaType::Primitive(Primitive::Void), &[]);

        let error = match result {
            Ok(JValue::Void) => None,
            Ok(value) => Some(Error::InvalidMethodResult(
                "closeTun",
                format!("{:?}", value),
            )),
            Err(error) => Some(error),
        };

        self.last_tun_config = None;

        if let Some(error) = error {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to close the tunnel")
            );
        }
    }

    fn recreate_tun_if_open(&mut self) -> Result<(), Error> {
        if let Some((config, blocking)) = self.last_tun_config.clone() {
            let _ = self.get_tun_inner(config, blocking)?;
        }
        Ok(())
    }

    /// Allow a socket to bypass the tunnel.
    pub fn bypass(&mut self, socket: RawFd) -> Result<(), Error> {
        let env = JnixEnv::from(
            self.jvm
                .attach_current_thread_as_daemon()
                .map_err(Error::AttachJvmToThread)?,
        );
        let create_tun_method = env
            .get_method_id(&self.class, "bypass", "(I)Z")
            .map_err(|cause| Error::FindMethod("bypass", cause))?;

        let result = env
            .call_method_unchecked(
                self.object.as_obj(),
                create_tun_method,
                JavaType::Primitive(Primitive::Boolean),
                &[JValue::Int(socket)],
            )
            .map_err(|cause| Error::CallMethod("bypass", cause))?;

        match result {
            JValue::Bool(0) => Err(Error::Bypass),
            JValue::Bool(_) => Ok(()),
            value => Err(Error::InvalidMethodResult("bypass", format!("{:?}", value))),
        }
    }

    fn call_method(
        &self,
        name: &'static str,
        signature: &str,
        return_type: JavaType,
        parameters: &[JValue<'_>],
    ) -> Result<JValue<'_>, Error> {
        let env = JnixEnv::from(
            self.jvm
                .attach_current_thread_as_daemon()
                .map_err(Error::AttachJvmToThread)?,
        );
        let method_id = env
            .get_method_id(&self.class, name, signature)
            .map_err(|cause| Error::FindMethod(name, cause))?;

        env.call_method_unchecked(self.object.as_obj(), method_id, return_type, parameters)
            .map_err(|cause| Error::CallMethod(name, cause))
    }

    fn env(&self) -> Result<JnixEnv<'_>, Error> {
        let jni_env = self
            .jvm
            .attach_current_thread_as_daemon()
            .map_err(Error::AttachJvmToThread)?;

        Ok(JnixEnv::from(jni_env))
    }
}

/// Configuration to use for VpnService
#[derive(Clone, Debug, Eq, PartialEq, IntoJava)]
#[jnix(class_name = "net.mullvad.talpid.model.TunConfig")]
struct VpnServiceConfig {
    /// IP addresses for the tunnel interface.
    pub addresses: Vec<IpAddr>,

    /// IP addresses for the DNS servers to use.
    pub dns_servers: Vec<IpAddr>,

    /// Routes to configure for the tunnel.
    pub routes: Vec<InetNetwork>,

    /// App packages that should be excluded from the tunnel.
    pub excluded_packages: Vec<String>,

    /// Maximum Transmission Unit in the tunnel.
    #[jnix(map = "|mtu| mtu as i32")]
    pub mtu: u16,
}

impl VpnServiceConfig {
    pub fn new(
        tun_config: TunConfig,
        allowed_lan_networks: &[IpNetwork],
        allow_lan: bool,
        dns_servers: Option<Vec<IpAddr>>,
        excluded_packages: Vec<String>,
    ) -> VpnServiceConfig {
        let dns_servers = Self::resolve_dns_servers(&tun_config, dns_servers);
        let routes = Self::resolve_routes(&tun_config, allowed_lan_networks, allow_lan);

        VpnServiceConfig {
            addresses: tun_config.addresses,
            dns_servers,
            routes,
            excluded_packages,
            mtu: tun_config.mtu,
        }
    }

    /// Return a list of custom DNS servers. If not specified, gateway addresses are used for DNS.
    /// Note that `Some(vec![])` is different from `None`. `Some(vec![])` disables DNS.
    fn resolve_dns_servers(config: &TunConfig, custom_dns: Option<Vec<IpAddr>>) -> Vec<IpAddr> {
        custom_dns.unwrap_or_else(|| config.gateway_ips())
    }

    /// Potentially subtract LAN nets from the VPN service routes, excepting gateways.
    /// This prevents LAN traffic from going in the tunnel.
    fn resolve_routes(
        config: &TunConfig,
        allowed_lan_networks: &[IpNetwork],
        allow_lan: bool,
    ) -> Vec<InetNetwork> {
        if !allow_lan {
            return config
                .routes
                .iter()
                .cloned()
                .map(InetNetwork::from)
                .collect();
        }

        let required_ipv4_routes = vec![IpNetwork::from(IpAddr::from(config.ipv4_gateway))];
        let required_ipv6_routes = config
            .ipv6_gateway
            .map(|addr| IpNetwork::from(IpAddr::from(addr)))
            .into_iter()
            .collect::<Vec<IpNetwork>>();

        let (original_lan_ipv4_networks, original_lan_ipv6_networks) = allowed_lan_networks
            .iter()
            .cloned()
            .partition::<Vec<_>, _>(|network| network.is_ipv4());

        let lan_ipv4_networks = original_lan_ipv4_networks
            .into_iter()
            .flat_map(|network| network.sub_all(required_ipv4_routes.clone()))
            .collect::<Vec<_>>();

        let lan_ipv6_networks = original_lan_ipv6_networks
            .into_iter()
            .flat_map(|network| network.sub_all(required_ipv6_routes.clone()))
            .collect::<Vec<_>>();

        config
            .routes
            .iter()
            .flat_map(|&route| {
                if route.is_ipv4() {
                    route.sub_all(lan_ipv4_networks.clone())
                } else {
                    route.sub_all(lan_ipv6_networks.clone())
                }
            })
            .map(InetNetwork::from)
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, IntoJava)]
#[jnix(package = "net.mullvad.talpid.model")]
struct InetNetwork {
    address: IpAddr,
    prefix: i16,
}

impl From<IpNetwork> for InetNetwork {
    fn from(ip_network: IpNetwork) -> Self {
        InetNetwork {
            address: ip_network.ip(),
            prefix: ip_network.prefix() as i16,
        }
    }
}

/// Handle to a tunnel device on Android.
pub struct VpnServiceTun {
    tunnel: RawFd,
    jvm: JavaVM,
    class: GlobalRef,
    object: GlobalRef,
}

impl VpnServiceTun {
    /// Retrieve the tunnel interface name.
    pub fn interface_name(&self) -> &str {
        "tun"
    }

    /// Allow a socket to bypass the tunnel.
    pub fn bypass(&mut self, socket: RawFd) -> Result<(), Error> {
        let env = JnixEnv::from(
            self.jvm
                .attach_current_thread_as_daemon()
                .map_err(Error::AttachJvmToThread)?,
        );
        let create_tun_method = env
            .get_method_id(&self.class, "bypass", "(I)Z")
            .map_err(|cause| Error::FindMethod("bypass", cause))?;

        let result = env
            .call_method_unchecked(
                self.object.as_obj(),
                create_tun_method,
                JavaType::Primitive(Primitive::Boolean),
                &[JValue::Int(socket)],
            )
            .map_err(|cause| Error::CallMethod("bypass", cause))?;

        if !bool::from_java(&env, result) {
            return Err(Error::Bypass);
        }
        Ok(())
    }
}

impl AsRawFd for VpnServiceTun {
    fn as_raw_fd(&self) -> RawFd {
        self.tunnel
    }
}

impl Default for TunConfig {
    fn default() -> Self {
        // Default configuration simply intercepts all packets. The only field that matters is
        // `routes`, because it determines what must enter the tunnel. All other fields contain
        // stub values.
        TunConfig {
            addresses: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
            ipv4_gateway: Ipv4Addr::new(10, 64, 0, 1),
            ipv6_gateway: None,
            routes: vec![
                IpNetwork::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)
                    .expect("Invalid IP network prefix for IPv4 address"),
                IpNetwork::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 0)
                    .expect("Invalid IP network prefix for IPv6 address"),
            ],
            mtu: 1380,
        }
    }
}

#[derive(FromJava)]
#[jnix(package = "net.mullvad.talpid.model")]
enum CreateTunResult {
    Success { tun_fd: i32 },
    InvalidDnsServers { addresses: Vec<IpAddr> },
    PermissionDenied,
    TunnelDeviceError,
}

impl From<CreateTunResult> for Result<RawFd, Error> {
    fn from(result: CreateTunResult) -> Self {
        match result {
            CreateTunResult::Success { tun_fd } => Ok(tun_fd),
            CreateTunResult::InvalidDnsServers { addresses } => {
                Err(Error::InvalidDnsServers(addresses))
            }
            CreateTunResult::PermissionDenied => Err(Error::PermissionDenied),
            CreateTunResult::TunnelDeviceError => Err(Error::TunnelDeviceError),
        }
    }
}
