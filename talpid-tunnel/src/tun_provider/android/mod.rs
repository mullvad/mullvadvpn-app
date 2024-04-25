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
    last_tun_config: TunConfig,
    allow_lan: bool,
    custom_dns_servers: Option<Vec<IpAddr>>,
    allowed_lan_networks: Vec<IpNetwork>,
    excluded_apps: Vec<String>,
}

impl AndroidTunProvider {
    /// Create a new AndroidTunProvider interfacing with Android's VpnService.
    pub fn new(
        context: AndroidContext,
        allow_lan: bool,
        custom_dns_servers: Option<Vec<IpAddr>>,
        allowed_lan_networks: Vec<IpNetwork>,
        excluded_apps: Vec<String>,
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
            last_tun_config: TunConfig::default(),
            allow_lan,
            custom_dns_servers,
            allowed_lan_networks,
            excluded_apps,
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
    pub fn set_exclude_apps(&mut self, excluded_apps: Vec<String>) -> Result<(), Error> {
        if self.excluded_apps != excluded_apps {
            self.excluded_apps = excluded_apps;
            self.recreate_tun_if_open()?;
        }
        Ok(())
    }

    /// Retrieve a tunnel device with the provided configuration.
    pub fn get_tun(&mut self, config: TunConfig) -> Result<VpnServiceTun, Error> {
        let tun_fd = self.get_tun_fd(config.clone())?;

        self.last_tun_config = config;

        let jvm = unsafe { JavaVM::from_raw(self.jvm.get_java_vm_pointer()) }
            .map_err(Error::CloneJavaVm)?;

        Ok(VpnServiceTun {
            tunnel: tun_fd,
            jvm,
            class: self.class.clone(),
            object: self.object.clone(),
        })
    }

    /// Open a tunnel device that routes everything but custom DNS, and
    /// (potentially) LAN routes via the tunnel device.
    ///
    /// Will open a new tunnel if there is already an active tunnel. The previous tunnel will be
    /// closed.
    pub fn create_blocking_tun(&mut self) -> Result<(), Error> {
        let mut config = TunConfig::default();
        self.prepare_tun_config(&mut config);
        let _ = self.get_tun(config)?;
        Ok(())
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

        if let Some(error) = error {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to close the tunnel")
            );
        }
    }

    /// Return a copy of all excluded apps.
    pub fn get_excluded_apps(&self) -> impl Iterator<Item = String> + '_ {
        self.excluded_apps.iter().cloned()
    }

    fn get_tun_fd(&mut self, mut config: TunConfig) -> Result<RawFd, Error> {
        self.prepare_tun_config(&mut config);

        let env = self.env()?;
        let java_config = config.into_java(&env);

        let result = self.call_method(
            "getTun",
            "(Lnet/mullvad/mullvadvpn/model/TunConfig;)Lnet/mullvad/talpid/CreateTunResult;",
            JavaType::Object("net/mullvad/talpid/CreateTunResult".to_owned()),
            &[JValue::Object(java_config.as_obj())],
        )?;

        match result {
            JValue::Object(result) => CreateTunResult::from_java(&env, result).into(),
            value => Err(Error::InvalidMethodResult("getTun", format!("{:?}", value))),
        }
    }

    fn recreate_tun_if_open(&mut self) -> Result<(), Error> {
        let mut actual_config = self.last_tun_config.clone();

        self.prepare_tun_config(&mut actual_config);

        let env = self.env()?;
        let java_config = actual_config.into_java(&env);

        let result = self.call_method(
            "recreateTunIfOpen",
            "(Lnet/mullvad/mullvadvpn/model/TunConfig;)V",
            JavaType::Primitive(Primitive::Void),
            &[JValue::Object(java_config.as_obj())],
        )?;

        match result {
            JValue::Void => Ok(()),
            value => Err(Error::InvalidMethodResult("getTun", format!("{:?}", value))),
        }
    }

    fn prepare_tun_config(&self, config: &mut TunConfig) {
        self.prepare_tun_config_for_allow_lan(config);
        self.prepare_tun_config_for_custom_dns(config);
        self.prepare_tun_config_for_excluded_apps(config);
    }

    fn prepare_tun_config_for_allow_lan(&self, config: &mut TunConfig) {
        if self.allow_lan {
            let (required_ipv4_routes, required_ipv6_routes) = config
                .required_routes
                .iter()
                .cloned()
                .partition::<Vec<_>, _>(|route| route.is_ipv4());

            let (original_lan_ipv4_networks, original_lan_ipv6_networks) = self
                .allowed_lan_networks
                .iter()
                .cloned()
                .partition::<Vec<_>, _>(|network| network.is_ipv4());

            let lan_ipv4_networks = original_lan_ipv4_networks
                .into_iter()
                .flat_map(|network| network.sub_all(required_ipv4_routes.iter().cloned()))
                .collect::<Vec<_>>();

            let lan_ipv6_networks = original_lan_ipv6_networks
                .into_iter()
                .flat_map(|network| network.sub_all(required_ipv6_routes.iter().cloned()))
                .collect::<Vec<_>>();

            let routes = config
                .routes
                .iter()
                .flat_map(|&route| {
                    if route.is_ipv4() {
                        route.sub_all(lan_ipv4_networks.iter().cloned())
                    } else {
                        route.sub_all(lan_ipv6_networks.iter().cloned())
                    }
                })
                .collect();

            config.routes = routes;
        }
    }

    fn prepare_tun_config_for_custom_dns(&self, config: &mut TunConfig) {
        if let Some(custom_dns_servers) = self.custom_dns_servers.clone() {
            config.dns_servers = custom_dns_servers;
        }
    }

    fn prepare_tun_config_for_excluded_apps(&self, config: &mut TunConfig) {
        config.excluded_packages = self.excluded_apps.clone();
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
            dns_servers: Vec::new(),
            routes: vec![
                IpNetwork::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)
                    .expect("Invalid IP network prefix for IPv4 address"),
                IpNetwork::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 0)
                    .expect("Invalid IP network prefix for IPv6 address"),
            ],
            required_routes: vec![],
            excluded_packages: vec![],
            mtu: 1380,
        }
    }
}

#[derive(FromJava)]
#[jnix(package = "net.mullvad.talpid")]
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
