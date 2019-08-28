use crate::{get_class, into_java::IntoJava};
use futures::{
    sync::{mpsc, oneshot},
    Async, Future, Poll, Stream,
};
use jni::{
    objects::{GlobalRef, JMethodID, JObject, JValue},
    signature::{JavaType, Primitive},
    AttachGuard, JNIEnv, JavaVM,
};
use std::{
    io,
    os::unix::io::{AsRawFd, RawFd},
    thread,
};
use talpid_core::tunnel::tun_provider::{Tun, TunConfig, TunProvider};
use talpid_types::{BoxedError, ErrorExt};
use tokio_core::reactor::Core;

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

    #[error(display = "Failed to start Tokio reactor")]
    StartReactor(#[error(cause)] io::Error),

    #[error(display = "VpnServiceTunProvider thread stopped unexpectedly")]
    ThreadStopped,
}

/// Commands that can be sent to the VpnServiceTunProvider
pub enum VpnServiceTunCommand {
    GetTunnelInterface(TunConfig, oneshot::Sender<Option<VpnServiceTun>>),
}

/// VpnService tunnel interface provider.
///
/// Responsible for keeping track of the open tunnel interface as well as providing access to it
/// through separate sockets.
pub struct VpnServiceTunProvider<'env> {
    env: AttachGuard<'env>,
    mullvad_vpn_service: GlobalRef,
    create_tun_method: JMethodID<'env>,
    commands: mpsc::UnboundedReceiver<VpnServiceTunCommand>,
}

impl VpnServiceTunProvider<'_> {
    /// Spawn a new VpnServiceTunProvider interfacing with Android's VpnService.
    ///
    /// Returns a handle to the spawned VpnServiceTunProvider.
    pub fn spawn(
        old_env: &JNIEnv,
        old_mullvad_vpn_service: &JObject,
    ) -> Result<VpnServiceTunProviderHandle, Error> {
        let (command_tx, command_rx) = mpsc::unbounded();
        let (result_tx, result_rx) = oneshot::channel();
        let jvm = old_env.get_java_vm().map_err(Error::GetJvmInstance)?;
        let mullvad_vpn_service = old_env
            .new_global_ref(*old_mullvad_vpn_service)
            .map_err(Error::CreateGlobalReference)?;

        thread::spawn(move || {
            match VpnServiceTunProvider::start(&jvm, mullvad_vpn_service, command_rx) {
                Ok((mut reactor, provider)) => {
                    let _ = result_tx.send(Ok(()));
                    let _ = reactor.run(provider);
                }
                Err(error) => {
                    if let Err(Err(unsent_error)) = result_tx.send(Err(error)) {
                        log::error!(
                            "{}",
                            unsent_error.display_chain_with_msg(
                                "Failed to send VpnServiceTunProvider start error"
                            )
                        );
                    }
                }
            }
        });

        result_rx.wait().map_err(|_| Error::ThreadStopped)??;

        Ok(VpnServiceTunProviderHandle(command_tx))
    }
}

impl<'env> VpnServiceTunProvider<'env> {
    fn start(
        jvm: &'env JavaVM,
        mullvad_vpn_service: GlobalRef,
        command_rx: mpsc::UnboundedReceiver<VpnServiceTunCommand>,
    ) -> Result<(Core, Self), Error> {
        let env = jvm
            .attach_current_thread()
            .map_err(Error::AttachJvmToThread)?;
        let reactor = Core::new().map_err(Error::StartReactor)?;
        let provider = VpnServiceTunProvider::new(env, mullvad_vpn_service, command_rx)?;

        Ok((reactor, provider))
    }

    fn new(
        env: AttachGuard<'env>,
        mullvad_vpn_service: GlobalRef,
        commands: mpsc::UnboundedReceiver<VpnServiceTunCommand>,
    ) -> Result<Self, Error> {
        let class = get_class("net/mullvad/mullvadvpn/MullvadVpnService");
        let create_tun_method = env
            .get_method_id(
                &class,
                "createTun",
                "(Lnet/mullvad/mullvadvpn/model/TunConfig;)I",
            )
            .map_err(|cause| Error::FindMethod("MullvadVpnService.createTun", cause))?;

        Ok(VpnServiceTunProvider {
            env,
            mullvad_vpn_service,
            create_tun_method,
            commands,
        })
    }

    fn poll_commands(&mut self) -> Poll<(), ()> {
        loop {
            match self.commands.poll()? {
                Async::Ready(Some(command)) => self.handle_command(command),
                Async::Ready(None) => return Ok(Async::Ready(())),
                Async::NotReady => return Ok(Async::NotReady),
            }
        }
    }

    fn handle_command(&mut self, command: VpnServiceTunCommand) {
        let VpnServiceTunCommand::GetTunnelInterface(config, result_tx) = command;

        match self.create_tun(config) {
            Ok(tun) => {
                if result_tx.send(Some(tun)).is_err() {
                    log::error!("Failed to send tun back to requester");
                }
            }
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to create tunnel")
                );
                let _ = result_tx.send(None);
            }
        }
    }

    fn create_tun(&self, config: TunConfig) -> Result<VpnServiceTun, Error> {
        let result = self
            .env
            .call_method_unchecked(
                self.mullvad_vpn_service.as_obj(),
                self.create_tun_method,
                JavaType::Primitive(Primitive::Int),
                &[JValue::Object(config.into_java(&self.env))],
            )
            .map_err(|cause| Error::CallMethod("MullvadVpnService.createTun", cause))?;

        match result {
            JValue::Int(fd) => Ok(VpnServiceTun {
                tunnel: fd,
                jvm: self.env.get_java_vm().map_err(Error::GetJvmInstance)?,
                object: self.mullvad_vpn_service.clone(),
            }),
            value => Err(Error::InvalidMethodResult(
                "MullvadVpnService.createTun",
                format!("{:?}", value),
            )),
        }
    }
}

impl<'env> Future for VpnServiceTunProvider<'env> {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.poll_commands()
    }
}

#[derive(Clone, Copy, Debug, err_derive::Error)]
enum CreateTunError {
    #[error(display = "Failed to request tunnel provider to create a tunnel")]
    Communication,
    #[error(display = "Tunnel provider failed to create a tunnel")]
    Operation,
}

/// Interface to `VpnServiceTunProvider`.
#[derive(Clone)]
pub struct VpnServiceTunProviderHandle(mpsc::UnboundedSender<VpnServiceTunCommand>);

impl TunProvider for VpnServiceTunProviderHandle {
    fn create_tun(&self, config: TunConfig) -> Result<Box<dyn Tun>, BoxedError> {
        let (tx, rx) = oneshot::channel();

        self.0
            .unbounded_send(VpnServiceTunCommand::GetTunnelInterface(config, tx))
            .map_err(|_| BoxedError::new(CreateTunError::Communication))?;

        rx.wait()
            .map_err(|_| CreateTunError::Communication)
            .and_then(|maybe_tun| {
                let tun = maybe_tun.ok_or(CreateTunError::Operation)?;
                let boxed_tun: Box<dyn Tun> = Box::new(tun);

                Ok(boxed_tun)
            })
            .map_err(BoxedError::new)
    }
}

/// Handle to tunnel created by `VpnServiceTunProvider`.
pub struct VpnServiceTun {
    tunnel: RawFd,
    jvm: JavaVM,
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
        let class = get_class("net/mullvad/mullvadvpn/MullvadVpnService");
        let create_tun_method = env
            .get_method_id(&class, "bypass", "(I)Z")
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
