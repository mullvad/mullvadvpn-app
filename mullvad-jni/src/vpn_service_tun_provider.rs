use crate::{get_class, into_java::IntoJava};
use ipnetwork::IpNetwork;
use jni::{
    objects::{GlobalRef, JObject, JValue},
    signature::{JavaType, Primitive},
    JNIEnv, JavaVM,
};
use std::{
    fs::File,
    io,
    net::{IpAddr, Ipv4Addr},
    os::unix::io::{AsRawFd, FromRawFd, RawFd},
};
use talpid_core::tunnel::tun_provider::{Tun, TunConfig, TunProvider};
use talpid_types::BoxedError;

/// Errors that occur while setting up VpnService tunnel.
#[derive(Debug, err_derive::Error)]
#[error(display = "Failed to set up the VpnService")]
pub enum Error {
    #[error(display = "Failed to attach Java VM to tunnel thread")]
    AttachJvmToThread(#[error(cause)] jni::errors::Error),

    #[error(display = "Failed to allow socket to bypass tunnel")]
    Bypass,

    #[error(display = "Failed to call Java method {}", _0)]
    CallMethod(&'static str, #[error(cause)] jni::errors::Error),

    #[error(display = "Failed to create Java VM handle clone")]
    CloneJavaVm(#[error(cause)] jni::errors::Error),

    #[error(display = "Failed to create global reference to MullvadVpnService instance")]
    CreateGlobalReference(#[error(cause)] jni::errors::Error),

    #[error(display = "Failed to duplicate tunnel file descriptor")]
    DuplicateTunFd(#[error(cause)] io::Error),

    #[error(display = "Failed to find {} method", _0)]
    FindMethod(&'static str, #[error(cause)] jni::errors::Error),

    #[error(display = "Failed to get Java VM instance")]
    GetJvmInstance(#[error(cause)] jni::errors::Error),

    #[error(display = "Received an invalid result from {}: {}", _0, _1)]
    InvalidMethodResult(&'static str, String),
}

/// Factory of tunnel devices on Android.
pub struct VpnServiceTunProvider {
    jvm: JavaVM,
    class: GlobalRef,
    object: GlobalRef,
    active_tun: Option<File>,
    current_tun_config: TunConfig,
}

impl VpnServiceTunProvider {
    /// Create a new VpnServiceTunProvider interfacing with Android's VpnService.
    pub fn new(env: &JNIEnv, mullvad_vpn_service: &JObject) -> Result<Self, Error> {
        let jvm = env.get_java_vm().map_err(Error::GetJvmInstance)?;
        let class = get_class("net/mullvad/mullvadvpn/MullvadVpnService");
        let object = env
            .new_global_ref(*mullvad_vpn_service)
            .map_err(Error::CreateGlobalReference)?;

        // Initial configuration simply intercepts all packets. The only field that matters is
        // `routes`, because it determines what must enter the tunnel. All other fields contain
        // stub values.
        let initial_tun_config = TunConfig {
            addresses: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
            dns_servers: Vec::new(),
            routes: vec![IpNetwork::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)
                .expect("Invalid IP network prefix")],
            mtu: 1380,
        };

        Ok(VpnServiceTunProvider {
            jvm,
            class,
            object,
            active_tun: None,
            current_tun_config: initial_tun_config,
        })
    }

    fn get_tun(&mut self, config: TunConfig) -> Result<VpnServiceTun, Error> {
        let tun = self.prepare_tun(config)?;
        let tun_fd = unsafe { libc::dup(tun.as_raw_fd()) };

        if tun_fd < 0 {
            return Err(Error::DuplicateTunFd(io::Error::last_os_error()));
        }

        let jvm = unsafe { JavaVM::from_raw(self.jvm.get_java_vm_pointer()) }
            .map_err(Error::CloneJavaVm)?;

        Ok(VpnServiceTun {
            tunnel: tun_fd,
            jvm,
            class: self.class.clone(),
            object: self.object.clone(),
        })
    }

    fn prepare_tun(&mut self, config: TunConfig) -> Result<&File, Error> {
        if self.active_tun.is_none() || self.current_tun_config != config {
            let tun = self.create_tunnel(config.clone())?;

            self.active_tun = Some(tun);
            self.current_tun_config = config;
        };

        Ok(self
            .active_tun
            .as_ref()
            .expect("Tunnel should be configured"))
    }

    fn create_tunnel(&mut self, config: TunConfig) -> Result<File, Error> {
        let env = self
            .jvm
            .attach_current_thread()
            .map_err(Error::AttachJvmToThread)?;
        let create_tun_method = env
            .get_method_id(
                &self.class,
                "createTun",
                "(Lnet/mullvad/mullvadvpn/model/TunConfig;)I",
            )
            .map_err(|cause| Error::FindMethod("MullvadVpnService.createTun", cause))?;

        let result = env
            .call_method_unchecked(
                self.object.as_obj(),
                create_tun_method,
                JavaType::Primitive(Primitive::Int),
                &[JValue::Object(config.into_java(&env))],
            )
            .map_err(|cause| Error::CallMethod("MullvadVpnService.createTun", cause))?;

        match result {
            JValue::Int(fd) => Ok(unsafe { File::from_raw_fd(fd) }),
            value => Err(Error::InvalidMethodResult(
                "MullvadVpnService.createTun",
                format!("{:?}", value),
            )),
        }
    }
}

impl TunProvider for VpnServiceTunProvider {
    fn create_tun(&mut self, config: TunConfig) -> Result<Box<dyn Tun>, BoxedError> {
        match self.get_tun(config) {
            Ok(tun) => Ok(Box::new(tun)),
            Err(error) => Err(BoxedError::new(error)),
        }
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
        let env = self
            .jvm
            .attach_current_thread()
            .map_err(|cause| BoxedError::new(Error::AttachJvmToThread(cause)))?;
        let create_tun_method =
            env.get_method_id(&self.class, "bypass", "(I)Z")
                .map_err(|cause| {
                    BoxedError::new(Error::FindMethod("MullvadVpnService.bypass", cause))
                })?;

        let result = env
            .call_method_unchecked(
                self.object.as_obj(),
                create_tun_method,
                JavaType::Primitive(Primitive::Boolean),
                &[JValue::Int(socket)],
            )
            .map_err(|cause| {
                BoxedError::new(Error::CallMethod("MullvadVpnService.bypass", cause))
            })?;

        match result {
            JValue::Bool(0) => Err(BoxedError::new(Error::Bypass)),
            JValue::Bool(_) => Ok(()),
            value => Err(BoxedError::new(Error::InvalidMethodResult(
                "MullvadVpnService.bypass",
                format!("{:?}", value),
            ))),
        }
    }
}
