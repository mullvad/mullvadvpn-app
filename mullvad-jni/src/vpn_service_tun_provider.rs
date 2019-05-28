use crate::{get_class, into_java::IntoJava};
use jni::{
    objects::{GlobalRef, JObject, JValue},
    signature::{JavaType, Primitive},
    JNIEnv, JavaVM,
};
use std::os::unix::io::{AsRawFd, RawFd};
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

    #[error(display = "Failed to create global reference to MullvadVpnService instance")]
    CreateGlobalReference(#[error(cause)] jni::errors::Error),

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
}

impl VpnServiceTunProvider {
    /// Create a new VpnServiceTunProvider interfacing with Android's VpnService.
    pub fn new(env: &JNIEnv, mullvad_vpn_service: &JObject) -> Result<Self, Error> {
        let jvm = env.get_java_vm().map_err(Error::GetJvmInstance)?;
        let class = get_class("net/mullvad/mullvadvpn/MullvadVpnService");
        let object = env
            .new_global_ref(*mullvad_vpn_service)
            .map_err(Error::CreateGlobalReference)?;

        Ok(VpnServiceTunProvider { jvm, class, object })
    }
}

impl TunProvider for VpnServiceTunProvider {
    fn create_tun(&self, config: TunConfig) -> Result<Box<dyn Tun>, BoxedError> {
        let env = self
            .jvm
            .attach_current_thread()
            .map_err(|cause| BoxedError::new(Error::AttachJvmToThread(cause)))?;
        let create_tun_method = env
            .get_method_id(
                &self.class,
                "createTun",
                "(Lnet/mullvad/mullvadvpn/model/TunConfig;)I",
            )
            .map_err(|cause| {
                BoxedError::new(Error::FindMethod("MullvadVpnService.createTun", cause))
            })?;

        let result = env
            .call_method_unchecked(
                self.object.as_obj(),
                create_tun_method,
                JavaType::Primitive(Primitive::Int),
                &[JValue::Object(config.into_java(&env))],
            )
            .map_err(|cause| {
                BoxedError::new(Error::CallMethod("MullvadVpnService.createTun", cause))
            })?;

        match result {
            JValue::Int(fd) => Ok(Box::new(VpnServiceTun {
                tunnel: fd,
                jvm: env
                    .get_java_vm()
                    .map_err(|cause| BoxedError::new(Error::GetJvmInstance(cause)))?,
                class: self.class.clone(),
                object: self.object.clone(),
            })),
            value => Err(BoxedError::new(Error::InvalidMethodResult(
                "MullvadVpnService.createTun",
                format!("{:?}", value),
            ))),
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
