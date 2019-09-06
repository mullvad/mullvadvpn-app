use crate::{get_class, into_java::IntoJava};
use futures::{
    sync::{mpsc, oneshot},
    Async, Future, Poll, Stream,
};
use ipnetwork::IpNetwork;
use jni::{
    objects::{GlobalRef, JMethodID, JObject, JValue},
    signature::{JavaType, Primitive},
    AttachGuard, JNIEnv, JavaVM,
};
use std::{
    borrow::Cow,
    fs::File,
    io,
    net::{IpAddr, Ipv4Addr},
    os::unix::io::{AsRawFd, FromRawFd, RawFd},
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

    #[error(display = "Failed to duplicate tunnel file descriptor")]
    DuplicateTunFd(#[error(cause)] io::Error),

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
    Bypass(RawFd, oneshot::Sender<bool>),
    CloseTunnel(oneshot::Sender<()>),
    GetTunnelInterface(TunConfig, oneshot::Sender<Option<VpnServiceTun>>),
    OpenTunnel(oneshot::Sender<Result<(), Error>>),
}

/// VpnService tunnel interface provider.
///
/// Responsible for keeping track of the open tunnel interface as well as providing access to it
/// through separate sockets.
pub struct VpnServiceTunProvider<'env> {
    env: AttachGuard<'env>,
    mullvad_vpn_service: GlobalRef,
    bypass_method: JMethodID<'env>,
    create_tun_method: JMethodID<'env>,
    active_tun: Option<File>,
    current_tun_config: TunConfig,
    commands: mpsc::UnboundedReceiver<VpnServiceTunCommand>,
    handle: VpnServiceTunProviderHandle,
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
        let handle = VpnServiceTunProviderHandle(command_tx);
        let handle_to_return = handle.clone();

        thread::spawn(move || {
            match VpnServiceTunProvider::start(&jvm, mullvad_vpn_service, command_rx, handle) {
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

        Ok(handle_to_return)
    }
}

impl<'env> VpnServiceTunProvider<'env> {
    fn start(
        jvm: &'env JavaVM,
        mullvad_vpn_service: GlobalRef,
        command_rx: mpsc::UnboundedReceiver<VpnServiceTunCommand>,
        handle: VpnServiceTunProviderHandle,
    ) -> Result<(Core, Self), Error> {
        let env = jvm
            .attach_current_thread()
            .map_err(Error::AttachJvmToThread)?;
        let reactor = Core::new().map_err(Error::StartReactor)?;
        let provider = VpnServiceTunProvider::new(env, mullvad_vpn_service, command_rx, handle)?;

        Ok((reactor, provider))
    }

    fn new(
        env: AttachGuard<'env>,
        mullvad_vpn_service: GlobalRef,
        commands: mpsc::UnboundedReceiver<VpnServiceTunCommand>,
        handle: VpnServiceTunProviderHandle,
    ) -> Result<Self, Error> {
        let class = get_class("net/mullvad/mullvadvpn/MullvadVpnService");
        let create_tun_method = env
            .get_method_id(
                &class,
                "createTun",
                "(Lnet/mullvad/mullvadvpn/model/TunConfig;)I",
            )
            .map_err(|cause| Error::FindMethod("MullvadVpnService.createTun", cause))?;
        let bypass_method = env
            .get_method_id(&class, "bypass", "(I)Z")
            .map_err(|cause| Error::FindMethod("MullvadVpnService.bypass", cause))?;

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
            env,
            mullvad_vpn_service,
            create_tun_method,
            active_tun: None,
            current_tun_config: initial_tun_config,
            bypass_method,
            commands,
            handle,
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
        use VpnServiceTunCommand::*;
        match command {
            Bypass(socket, result_tx) => self.handle_bypass(socket, result_tx),
            CloseTunnel(result_tx) => self.handle_close_tunnel(result_tx),
            GetTunnelInterface(config, result_tx) => {
                self.handle_get_tunnel_interface(config, result_tx)
            }
            OpenTunnel(result_tx) => self.handle_open_tunnel(result_tx),
        }
    }

    fn handle_open_tunnel(&mut self, result_tx: oneshot::Sender<Result<(), Error>>) {
        if let Err(result) = result_tx.send(self.open_tunnel()) {
            log::error!(
                "Failed to send open tunnel result back to requester, which was: {}",
                match result {
                    Ok(()) => Cow::Borrowed("Success."),
                    Err(error) => Cow::Owned(error.display_chain()),
                }
            )
        }
    }

    fn open_tunnel(&mut self) -> Result<(), Error> {
        if self.active_tun.is_none() {
            self.active_tun = Some(self.create_tun(self.current_tun_config.clone())?);
        }

        Ok(())
    }

    fn handle_close_tunnel(&mut self, result_tx: oneshot::Sender<()>) {
        self.active_tun = None;

        let _ = result_tx.send(());
    }

    fn handle_get_tunnel_interface(
        &mut self,
        config: TunConfig,
        result_tx: oneshot::Sender<Option<VpnServiceTun>>,
    ) {
        match self.get_tunnel_interface(config) {
            Ok(value) => {
                if result_tx.send(Some(value)).is_err() {
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

    fn get_tunnel_interface(&mut self, config: TunConfig) -> Result<VpnServiceTun, Error> {
        let tun = self.prepare_tun(config)?;
        let tun_fd = unsafe { libc::dup(tun.as_raw_fd()) };

        if tun_fd < 0 {
            return Err(Error::DuplicateTunFd(io::Error::last_os_error()));
        }

        Ok(VpnServiceTun {
            tunnel: tun_fd,
            provider: self.handle.clone(),
        })
    }

    fn prepare_tun(&mut self, config: TunConfig) -> Result<&File, Error> {
        if self.active_tun.is_none() || self.current_tun_config != config {
            let tun = self.create_tun(config.clone())?;

            self.active_tun = Some(tun);
            self.current_tun_config = config;
        };

        Ok(self
            .active_tun
            .as_ref()
            .expect("Tunnel should be configured"))
    }

    fn create_tun(&self, config: TunConfig) -> Result<File, Error> {
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
            JValue::Int(fd) => Ok(unsafe { File::from_raw_fd(fd) }),
            value => Err(Error::InvalidMethodResult(
                "MullvadVpnService.createTun",
                format!("{:?}", value),
            )),
        }
    }

    fn handle_bypass(&mut self, socket: RawFd, result_tx: oneshot::Sender<bool>) {
        match self.bypass(socket) {
            Ok(()) => {
                if result_tx.send(true).is_err() {
                    log::error!("Failed to send bypass result to requester");
                }
            }
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to make socket bypass tunnel")
                );
                let _ = result_tx.send(false);
            }
        }
    }

    fn bypass(&mut self, socket: RawFd) -> Result<(), Error> {
        let result = self
            .env
            .call_method_unchecked(
                self.mullvad_vpn_service.as_obj(),
                self.bypass_method,
                JavaType::Primitive(Primitive::Boolean),
                &[JValue::Int(socket)],
            )
            .map_err(|cause| Error::CallMethod("MullvadVpnService.bypass", cause))?;

        match result {
            JValue::Bool(0) => Err(Error::Bypass),
            JValue::Bool(_) => Ok(()),
            value => Err(Error::InvalidMethodResult(
                "MullvadVpnService.bypass",
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
enum BypassError {
    #[error(display = "Failed to request tunnel provider to bypass a socket from the tunnel")]
    Communication,
    #[error(display = "Tunnel provider failed to bypass a socket from the tunnel")]
    Operation,
}

#[derive(Clone, Copy, Debug, err_derive::Error)]
enum CreateTunError {
    #[error(display = "Failed to request tunnel provider to create a tunnel")]
    Communication,
    #[error(display = "Tunnel provider failed to create a tunnel")]
    Operation,
}

#[derive(Clone, Copy, Debug, err_derive::Error)]
enum OpenTunError {
    #[error(
        display = "Failed to request tunnel provider to open a tunnel using the previous configuration"
    )]
    Communication,
    #[error(display = "Tunnel provider failed to open a tunnel using the previous configuration")]
    Operation,
}

#[derive(Clone, Copy, Debug, err_derive::Error)]
#[error(display = "Failed to request tunnel provider to close the tunnel")]
struct CloseTunError;

/// Interface to `VpnServiceTunProvider`.
#[derive(Clone)]
pub struct VpnServiceTunProviderHandle(mpsc::UnboundedSender<VpnServiceTunCommand>);

impl VpnServiceTunProviderHandle {
    fn bypass(&self, socket: RawFd) -> Result<(), BypassError> {
        let (tx, rx) = oneshot::channel();

        self.0
            .unbounded_send(VpnServiceTunCommand::Bypass(socket, tx))
            .map_err(|_| BypassError::Communication)?;

        match rx.wait().map_err(|_| BypassError::Communication)? {
            true => Ok(()),
            false => Err(BypassError::Operation),
        }
    }
}

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

    fn open_tun(&self) -> Result<(), BoxedError> {
        let (tx, rx) = oneshot::channel();

        self.0
            .unbounded_send(VpnServiceTunCommand::OpenTunnel(tx))
            .map_err(|_| BoxedError::new(OpenTunError::Communication))?;

        rx.wait()
            .map_err(|_| BoxedError::new(OpenTunError::Communication))?
            .map_err(|_| BoxedError::new(OpenTunError::Operation))
    }

    fn close_tun(&self) -> Result<(), BoxedError> {
        let (tx, rx) = oneshot::channel();

        self.0
            .unbounded_send(VpnServiceTunCommand::CloseTunnel(tx))
            .map_err(|_| BoxedError::new(CloseTunError))?;

        rx.wait().map_err(|_| BoxedError::new(CloseTunError))
    }
}

/// Handle to tunnel created by `VpnServiceTunProvider`.
pub struct VpnServiceTun {
    tunnel: RawFd,
    provider: VpnServiceTunProviderHandle,
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
        self.provider.bypass(socket).map_err(BoxedError::new)
    }
}
