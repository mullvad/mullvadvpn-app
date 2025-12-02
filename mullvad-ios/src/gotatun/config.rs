use std::{
    ffi::CStr,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    ptr,
    str::FromStr,
};

use boringtun::device::{
    api::command::{Peer, Set},
    peer::AllowedIP,
};
use ipnetwork::IpNetwork;
use libc::c_char;
use talpid_types::net::wireguard::{PeerConfig, PresharedKey, PrivateKey, PublicKey};

#[repr(C)]
pub struct SwiftGotaTunConfiguration(*mut GotaTunConfiguration);

impl SwiftGotaTunConfiguration {
    pub unsafe fn mut_config(&mut self) -> &mut GotaTunConfiguration {
        &mut *self.0
    }

    pub unsafe fn config(&self) -> &GotaTunConfiguration {
        &mut *self.0
    }
}

#[derive(Default, Clone)]
pub struct GotaTunConfiguration {
    pub exit: Option<PeerConfiguration>,
    pub entry: Option<PeerConfiguration>,
    pub private_ip_v4: Option<Ipv4Addr>,
    pub private_ip_v6: Option<Ipv6Addr>,
    // TODO: add DAITA configuration
}

#[repr(C)]
pub enum ConfigStatus {
    Success = 0,
    InvalidArg = -1,
    OpenTunDevice = -2,
    SetAsyncDevice = -3,
    SetConfigFailure = -4,
    NoTokioRuntime = -5,
    NotRunning = -6,
}

impl GotaTunConfiguration {
    pub fn is_valid(&self) -> bool {
        self.exit.is_some()
    }

    unsafe fn set_exit(
        &mut self,
        local_private_key: *const u8,
        local_ephemeral_key: *const u8,
        peer_public_key: *const u8,
        peer_endpoint: *const c_char,
    ) -> ConfigStatus {
        let Some(peer_config) = PeerConfiguration::from_ptrs(
            local_private_key,
            local_ephemeral_key,
            peer_public_key,
            peer_endpoint,
        ) else {
            return ConfigStatus::InvalidArg;
        };

        Self::set_peer(&mut self.exit, peer_config);
        ConfigStatus::Success
    }

    unsafe fn set_entry(
        &mut self,
        local_private_key: *const u8,
        local_ephemeral_key: *const u8,
        peer_public_key: *const u8,
        peer_endpoint: *const c_char,
    ) -> ConfigStatus {
        let Some(peer_config) = PeerConfiguration::from_ptrs(
            local_private_key,
            local_ephemeral_key,
            peer_public_key,
            peer_endpoint,
        ) else {
            return ConfigStatus::InvalidArg;
        };

        Self::set_peer(&mut self.entry, peer_config);
        ConfigStatus::Success
    }

    fn set_peer(peer: &mut Option<PeerConfiguration>, config: PeerConfiguration) {
        *peer = Some(config);
    }

    fn set_ipv4_addr(&mut self, addr: &str) -> ConfigStatus {
        let Ok(addr) = addr.parse() else {
            return ConfigStatus::InvalidArg;
        };

        self.private_ip_v4 = Some(addr);
        ConfigStatus::Success
    }

    fn set_ipv6_addr(&mut self, addr: &str) -> ConfigStatus {
        let Ok(addr) = addr.parse() else {
            return ConfigStatus::InvalidArg;
        };

        self.private_ip_v6 = Some(addr);
        ConfigStatus::Success
    }
}

#[derive(Clone)]
pub struct PeerConfiguration {
    pub local_private_key: PrivateKey,
    pub local_ephemeral_key: PresharedKey,
    pub peer_public_key: PublicKey,
    pub peer_endpoint: SocketAddr,
}

impl PeerConfiguration {
    /// SAFETY: TODO
    unsafe fn from_ptrs(
        local_private_key: *const u8,
        local_ephemeral_key: *const u8,
        peer_public_key: *const u8,
        peer_endpoint: *const c_char,
    ) -> Option<Self> {
        let endpoint_str = CStr::from_ptr(peer_endpoint as *const _).to_str().ok()?;
        let peer_endpoint = SocketAddr::from_str(endpoint_str).ok()?;
        let private_key_bytes = unsafe { Self::read_key(local_private_key) };
        let ephemeral_key_bytes = unsafe { Self::read_key(local_ephemeral_key) };
        let peer_public_key_bytes = unsafe { Self::read_key(peer_public_key) };

        Some(Self {
            local_private_key: PrivateKey::from(private_key_bytes),
            local_ephemeral_key: PresharedKey::from(Box::new(ephemeral_key_bytes)),
            peer_public_key: PublicKey::from(peer_public_key_bytes),
            peer_endpoint,
        })
    }

    /// SAFETY:
    ///  This function is only safe if the pointer is valid for at least 32 bytes.
    unsafe fn read_key(key_ptr: *const u8) -> [u8; 32] {
        ptr::read(key_ptr as *const _)
    }

    pub fn get_peer(&self) -> Peer {
        Peer::builder()
            .public_key(*self.peer_public_key.as_bytes())
            .endpoint(self.peer_endpoint)
            .allowed_ip(vec![
                AllowedIP {
                    addr: Ipv4Addr::UNSPECIFIED.into(),
                    cidr: 0,
                },
                AllowedIP {
                    addr: Ipv6Addr::UNSPECIFIED.into(),
                    cidr: 0,
                },
            ])
            .build()
    }

    pub fn set_command(&self) -> Set {
        let mut set_cmd = Set::builder()
            .private_key(self.local_private_key.to_bytes())
            .listen_port(0u16)
            .replace_peers()
            .build();
        set_cmd
    }
}

///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_gotatun_config_new() -> SwiftGotaTunConfiguration {
    let config = Box::new(GotaTunConfiguration::default());
    SwiftGotaTunConfiguration(Box::into_raw(config))
}

///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_gotatun_config_set_exit(
    mut config: SwiftGotaTunConfiguration,
    local_private_key: *const u8,
    local_ephemeral_key: *const u8,
    peer_public_key: *const u8,
    peer_endpoint: *const c_char,
) -> i32 {
    let cfg = unsafe { config.mut_config() };
    unsafe {
        cfg.set_exit(
            local_private_key,
            local_ephemeral_key,
            peer_public_key,
            peer_endpoint,
        ) as i32
    }
}

///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_gotatun_config_set_private_ipv4(
    mut config: SwiftGotaTunConfiguration,
    ipv4: *const u8,
    peer_endpoint: *const c_char,
) -> i32 {
    let cfg = unsafe { config.mut_config() };
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_gotatun_config_set_private_ipv4(
    mut config: SwiftGotaTunConfiguration,
    ipv4: *const u8,
    peer_endpoint: *const c_char,
) -> i32 {
    let cfg = unsafe { config.mut_config() };
    let cstr = unsafe { CStr::from_ptr(ptr) };
    let Ok(s) = cstr.to_str() {
        return ConfigStatus::InvalidArg;
    }

    config.set_ipv4_addr(&s);
}
    let cstr = unsafe { CStr::from_ptr(ptr) };
    let Ok(s) = cstr.to_str() else {
        return ConfigStatus::InvalidArg;
    }

    config.set_ipv4_addr(&s);
}

///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_gotatun_config_set_private_ipv6(
    mut config: SwiftGotaTunConfiguration,
    ipv4: *const u8,
    peer_endpoint: *const c_char,
) -> i32 {
    let cfg = unsafe { config.mut_config() };
    let cstr = unsafe { CStr::from_ptr(ptr) };
    let Ok(s) = cstr.to_str() else {
        return ConfigStatus::InvalidArg;
    }

    config.set_ipv6_addr(&s);
}

///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_gotatun_config_set_entry(
    mut config: SwiftGotaTunConfiguration,
    local_private_key: *const u8,
    local_ephemeral_key: *const u8,
    peer_public_key: *const u8,
    peer_endpoint: *const c_char,
) -> i32 {
    let cfg = unsafe { config.mut_config() };
    unsafe {
        cfg.set_entry(
            local_private_key,
            local_ephemeral_key,
            peer_public_key,
            peer_endpoint,
        ) as i32
    }
}

///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_ios_gotatun_config_drop(config: SwiftGotaTunConfiguration) {
    if !config.0.is_null() {
        let _ = Box::from_raw(config.0);
    }
}
