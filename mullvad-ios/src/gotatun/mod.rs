use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ptr;

use boringtun::tun::MtuWatcher;
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

use crate::gotatun::config::GotaTunConfiguration;

#[repr(C)]
pub struct SwiftGotaTun(*mut GotaTun);
impl SwiftGotaTun {
    unsafe fn get_tunnel(&mut self) -> &mut GotaTun {
        unsafe { &mut *self.0 }
    }
}

type SinglehopDevice = DeviceHandle<(UdpSocketFactory, GotaTunDevice)>;
type EntryDevice = DeviceHandle<(UdpSocketFactory, TunChannelTx, TunChannelRx)>;
type ExitDevice = DeviceHandle<(UdpChannelFactory, GotaTunDevice)>;

struct GotaTun {
    devices: Option<Devices>,
}

enum Devices {
    Singlehop {
        device: SinglehopDevice,
        api: ApiClient,
    },

    Multihop {
        entry_device: EntryDevice,
        entry_api: ApiClient,

        exit_device: ExitDevice,
        exit_api: ApiClient,
    },
}

impl GotaTun {
    async fn create_devices(
        tun_fd: i32,
        config: &config::GotaTunConfiguration,
    ) -> Result<Self, ConfigStatus> {
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

        // Single hop config
        if config.entry.is_none() {
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

            let singlehop_device = Devices::Singlehop {
                device,
                api: device_api,
            };

            Ok(Self {
                devices: Some(singlehop_device),
            })
        } else {
            let entry_peer = config.entry.as_ref().unwrap().get_peer();
            let exit_peer = config.exit.as_ref().unwrap().get_peer();

            let source_v4 = config.private_ip_v4.unwrap_or(Ipv4Addr::UNSPECIFIED);
            let source_v6 = config.private_ip_v6.unwrap_or(Ipv6Addr::UNSPECIFIED);


            let multihop_overhead = match exit_peer.endpoint.unwrap().ip() {
                IpAddr::V4(..) => Ipv4Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
                IpAddr::V6(..) => Ipv6Header::LEN + UdpHeader::LEN + WgData::OVERHEAD,
            };
            let exit_mtu: MtuWatcher = gota_tun_device.mtu();
            let entry_mtu = exit_mtu.increase(multihop_overhead as u16).unwrap();

            let (tun_channel_tx, tun_channel_rx, udp_channels) =
                new_udp_tun_channel(100, source_v4, source_v6, entry_mtu);

            let (exit_api, exit_api_server) = ApiServer::new();
            let exit_device = ExitDevice::new(
                udp_channels,
                gota_tun_device.clone(),
                gota_tun_device,
                DeviceConfig {
                    api: Some(exit_api_server),
                },
            )
            .await;

            let (entry_api, entry_api_server) = ApiServer::new();
            let gotatun_entry_config = DeviceConfig {
                api: Some(entry_api_server),
            };

            let entry_device = EntryDevice::new(
                UdpSocketFactory,
                tun_channel_tx,
                tun_channel_rx,
                gotatun_entry_config,
            )
            .await;

            Ok(Self {
                devices: Some(Devices::Multihop {
                    entry_device,
                    entry_api,
                    exit_device,
                    exit_api,
                }),
            })
        }
    }

    async fn configure_devices(
        &mut self,
        config: &GotaTunConfiguration,
    ) -> Result<(), ConfigStatus> {
        match &self.devices {
            Some(Devices::Singlehop { device: _, api }) => {
                let exit_configuration = config.exit.as_ref().ok_or(ConfigStatus::InvalidArg)?;
                let mut set_cmd = exit_configuration.set_command();
                let peer = exit_configuration.get_peer();
                set_cmd.peers.push(SetPeer::builder().peer(peer).build());

                if api.send(set_cmd).await.is_err() {
                    return Err(ConfigStatus::SetConfigFailure);
                }

                Ok(())
            }
            Some(Devices::Multihop {
                entry_device: _,
                entry_api,
                exit_device: _,
                exit_api,
            }) => {
                let entry_configuration = config.entry.as_ref().ok_or(ConfigStatus::InvalidArg)?;
                let mut entry_set_cmd = entry_configuration.set_command();
                let entry_peer = entry_configuration.get_peer();
                entry_set_cmd
                    .peers
                    .push(SetPeer::builder().peer(entry_peer).build());

                if entry_api.send(entry_set_cmd).await.is_err() {
                    return Err(ConfigStatus::SetConfigFailure);
                }

                let exit_configuration = config.exit.as_ref().ok_or(ConfigStatus::InvalidArg)?;
                let mut exit_set_cmd = exit_configuration.set_command();
                let exit_peer = exit_configuration.get_peer();
                exit_set_cmd
                    .peers
                    .push(SetPeer::builder().peer(exit_peer).build());

                if exit_api.send(exit_set_cmd).await.is_err() {
                    return Err(ConfigStatus::SetConfigFailure);
                }
                Ok(())
            }
            None => todo!(),
        }
    }

    async fn stop(&mut self) {
        match self.devices.take() {
            Some(Devices::Singlehop { device, api: _ }) => device.stop().await,
            Some(Devices::Multihop {
                entry_device,
                entry_api,
                exit_device,
                exit_api,
            }) => todo!(),
            None => todo!(),
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
        let mut devices = GotaTun::create_devices(tun_fd, config).await?;
        devices.configure_devices(config).await?;
        Ok(devices)
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

    let result = tokio_handle.block_on(async { tun.stop().await });

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
