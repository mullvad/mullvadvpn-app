use std::ptr;

use boringtun::udp::UdpTransportFactory;
use boringtun::{
    device::{
        DeviceConfig, DeviceHandle,
        api::{ApiClient, ApiServer, command::*},
        peer::AllowedIP,
    },
    packet::{Ipv4Header, Ipv6Header, UdpHeader, WgData},
    tun::{
        IpRecv,
        channel::{TunChannelRx, TunChannelTx},
        tun_async_device::TunDevice as GotaTunDevice,
    },
    udp::{
        channel::{UdpChannelFactory, new_udp_tun_channel},
        socket::UdpSocketFactory,
    },
};
use tun07::{AbstractDevice, AsyncDevice};

mod config;
use config::{ConfigStatus, PeerConfiguration, SwiftGotaTunConfiguration};

#[repr(C)]
pub struct SwiftGotaTun(*mut GotaTun);
impl SwiftGotaTun {
    unsafe fn get_tunnel(&mut self) -> &mut GotaTun {
        unsafe {  &mut *self.0 }
    }
}


// type SinglehopDevice = DeviceHandle<(UdpFactory, GotaTunDevice)>;

struct GotaTun {
    device: Option<DeviceHandle<(UdpSocketFactory, GotaTunDevice)>>,
    device_api: ApiClient,
}

impl GotaTun {
    async fn create_device(tun_fd: i32) -> Result<Self, ConfigStatus> {
        let mut tun_config = tun07::Configuration::default();
        tun_config.raw_fd(tun_fd);

        let Ok(device) = tun07::Device::new(&tun_config) else {
            return Err(ConfigStatus::OpenTunDevice);
        };
        let Ok(async_device) = AsyncDevice::new(device) else {
            return Err(ConfigStatus::SetAsyncDevice);
        };

        let gota_tun_device = GotaTunDevice::from_tun_device(async_device)
            .expect("Failed to fetch MTU for given tunnel device");

        // let tun_device = GotaTunDevice::from_tun_device();
        let (device_api, device_api_server) = ApiServer::new();
        let device_config = DeviceConfig {
            api: Some(device_api_server),
        };
        let device = DeviceHandle::new(
            UdpSocketFactory,
            gota_tun_device.clone(),
            gota_tun_device,
            device_config,
        )
        .await;

        Ok(Self { device: Some(device), device_api })
    }

    async fn configure_device(
        &mut self,
        peer_configuration: &PeerConfiguration,
    ) -> Result<(), ConfigStatus> {
        let Some(device) = &mut self.device else {
            return Err(ConfigStatus::NotRunning);
        };


        let mut set_cmd = peer_configuration.set_command();
        let peer = peer_configuration.get_peer();
        set_cmd.peers.push(SetPeer::builder().peer(peer).build());

        if self.device_api.send(set_cmd).await.is_err() {
            return Err(ConfigStatus::SetConfigFailure);
        }

        Ok(())
    }

    async fn stop(&mut self) {
        if let Some(device) = self.device.take() {
            device.stop().await;
        }

    }
}

///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_gotatun_start(
    tun_ptr: *mut SwiftGotaTun,
    configuration: SwiftGotaTunConfiguration,
    tun_fd: i32,
) -> i32 {
    let config = unsafe { configuration.config() };
    let Some(ref exit) = config.exit else {
        return ConfigStatus::InvalidArg as i32;
    };

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        return ConfigStatus::NoTokioRuntime as i32;
    };

    let result: Result<GotaTun, ConfigStatus> = tokio_handle.block_on(async {
        let mut device = GotaTun::create_device(tun_fd).await?;
        device.configure_device(exit).await?;
        Ok(device)
    });

    match result {
        Ok(device) => {
            let gota_tun = Box::new(device);
            let gota_tun_ptr = Box::into_raw(gota_tun);

            unsafe { tun_ptr.write(SwiftGotaTun(gota_tun_ptr)) };

            0
        }
        Err(err) => err as i32,
    }
}

fn create_set_command(peer_configuration: PeerConfiguration) -> Set {
    let mut set_cmd = peer_configuration.set_command();
    let peer = peer_configuration.get_peer();
    set_cmd.peers.push(SetPeer::builder().peer(peer).build());

    set_cmd
}

/// Rebind sockets when the default route changes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_gotatun_rebind_sockets(tun_ptr: SwiftGotaTun) -> i32 {
    0
}

///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_gotatun_stop(mut tun_ptr: SwiftGotaTun) -> i32 {
    if tun_ptr.0.is_null() {
        return ConfigStatus::NotRunning as i32;
    }

    let tun = unsafe { tun_ptr.get_tunnel() };

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        return ConfigStatus::NoTokioRuntime as i32;
    };

    let result = tokio_handle.block_on(async {
        tun.stop().await
    });

    unsafe { mullvad_ios_gotatun_drop(tun_ptr) };
    return 0;
}

///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_gotatun_drop(mut tun_ptr: SwiftGotaTun) {
    if tun_ptr.0.is_null() {
        return;
    }

    let _ = Box::from_raw(tun_ptr.0);
    tun_ptr.0 = ptr::null_mut();
}
