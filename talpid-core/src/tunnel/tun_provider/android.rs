use super::{Tun, TunConfig, TunProvider};
use ipnetwork::IpNetwork;
use jnix::{
    jni::{
        objects::{GlobalRef, JValue},
        signature::{JavaType, Primitive},
        JavaVM,
    },
    IntoJava, JnixEnv,
};
use std::{
    fs::File,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    os::unix::io::{AsRawFd, FromRawFd, RawFd},
};
use talpid_types::{android::AndroidContext, BoxedError};


/// Errors that occur while setting up VpnService tunnel.
#[derive(Debug, err_derive::Error)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to attach Java VM to tunnel thread")]
    AttachJvmToThread(#[error(source)] jnix::jni::errors::Error),

    #[error(display = "Failed to allow socket to bypass tunnel")]
    Bypass,

    #[error(display = "Failed to call Java method TalpidVpnService.{}", _0)]
    CallMethod(&'static str, #[error(source)] jnix::jni::errors::Error),

    #[error(display = "Failed to create Java VM handle clone")]
    CloneJavaVm(#[error(source)] jnix::jni::errors::Error),

    #[error(display = "Failed to find TalpidVpnService.{} method", _0)]
    FindMethod(&'static str, #[error(source)] jnix::jni::errors::Error),

    #[error(
        display = "Received an invalid result from TalpidVpnService.{}: {}",
        _0,
        _1
    )]
    InvalidMethodResult(&'static str, String),
}

/// Factory of tunnel devices on Android.
pub struct AndroidTunProvider {
    jvm: JavaVM,
    class: GlobalRef,
    object: GlobalRef,
    active_tun: Option<File>,
    last_tun_config: TunConfig,
}

impl AndroidTunProvider {
    /// Create a new AndroidTunProvider interfacing with Android's VpnService.
    pub fn new(context: AndroidContext) -> Self {
        // Initial configuration simply intercepts all packets. The only field that matters is
        // `routes`, because it determines what must enter the tunnel. All other fields contain
        // stub values.
        let initial_tun_config = TunConfig {
            addresses: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
            dns_servers: Vec::new(),
            routes: vec![
                IpNetwork::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)
                    .expect("Invalid IP network prefix for IPv4 address"),
                IpNetwork::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 0)
                    .expect("Invalid IP network prefix for IPv6 address"),
            ],
            mtu: 1380,
        };

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
            active_tun: None,
            last_tun_config: initial_tun_config,
        }
    }

    fn get_tun_fd(&mut self, config: TunConfig) -> Result<RawFd, Error> {
        if self.active_tun.is_none() || self.last_tun_config != config {
            self.open_tun(config)?;
        }

        Ok(self
            .active_tun
            .as_ref()
            .expect("Tunnel should be configured")
            .as_raw_fd())
    }

    fn open_tun(&mut self, config: TunConfig) -> Result<(), Error> {
        let env = JnixEnv::from(
            self.jvm
                .attach_current_thread_as_daemon()
                .map_err(Error::AttachJvmToThread)?,
        );
        let create_tun_method = env
            .get_method_id(
                &self.class,
                "createTun",
                "(Lnet/mullvad/talpid/tun_provider/TunConfig;)I",
            )
            .map_err(|cause| Error::FindMethod("createTun", cause))?;

        let java_config = config.clone().into_java(&env);
        let result = env
            .call_method_unchecked(
                self.object.as_obj(),
                create_tun_method,
                JavaType::Primitive(Primitive::Int),
                &[JValue::Object(java_config.as_obj())],
            )
            .map_err(|cause| Error::CallMethod("createTun", cause))?;

        match result {
            JValue::Int(fd) => {
                let tun = unsafe { File::from_raw_fd(fd) };

                self.active_tun = Some(tun);
                self.last_tun_config = config;

                Ok(())
            }
            value => Err(Error::InvalidMethodResult(
                "createTun",
                format!("{:?}", value),
            )),
        }
    }
}

impl TunProvider for AndroidTunProvider {
    fn get_tun(&mut self, config: TunConfig) -> Result<Box<dyn Tun>, BoxedError> {
        let tun_fd = self.get_tun_fd(config).map_err(BoxedError::new)?;

        let jvm = unsafe { JavaVM::from_raw(self.jvm.get_java_vm_pointer()) }
            .map_err(|cause| BoxedError::new(Error::CloneJavaVm(cause)))?;

        Ok(Box::new(VpnServiceTun {
            tunnel: tun_fd,
            jvm,
            class: self.class.clone(),
            object: self.object.clone(),
        }))
    }

    fn create_tun(&mut self) -> Result<(), BoxedError> {
        self.open_tun(self.last_tun_config.clone())
            .map_err(BoxedError::new)
    }

    fn create_tun_if_closed(&mut self) -> Result<(), BoxedError> {
        if self.active_tun.is_none() {
            self.create_tun()?;
        }

        Ok(())
    }

    fn close_tun(&mut self) {
        self.active_tun = None;
    }
}

struct VpnServiceTun {
    tunnel: RawFd,
    jvm: JavaVM,
    class: GlobalRef,
    object: GlobalRef,
}

impl AsRawFd for VpnServiceTun {
    fn as_raw_fd(&self) -> RawFd {
        self.tunnel
    }
}

impl Tun for VpnServiceTun {
    fn interface_name(&self) -> &str {
        "tun"
    }

    fn bypass(&mut self, socket: RawFd) -> Result<(), BoxedError> {
        let env = JnixEnv::from(
            self.jvm
                .attach_current_thread_as_daemon()
                .map_err(|cause| BoxedError::new(Error::AttachJvmToThread(cause)))?,
        );
        let create_tun_method = env
            .get_method_id(&self.class, "bypass", "(I)Z")
            .map_err(|cause| BoxedError::new(Error::FindMethod("bypass", cause)))?;

        let result = env
            .call_method_unchecked(
                self.object.as_obj(),
                create_tun_method,
                JavaType::Primitive(Primitive::Boolean),
                &[JValue::Int(socket)],
            )
            .map_err(|cause| BoxedError::new(Error::CallMethod("bypass", cause)))?;

        match result {
            JValue::Bool(0) => Err(BoxedError::new(Error::Bypass)),
            JValue::Bool(_) => Ok(()),
            value => Err(BoxedError::new(Error::InvalidMethodResult(
                "bypass",
                format!("{:?}", value),
            ))),
        }
    }
}
