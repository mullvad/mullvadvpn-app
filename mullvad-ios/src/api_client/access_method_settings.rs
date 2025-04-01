use std::{
    ffi::{c_char, c_void},
    ptr::null_mut,
};

use mullvad_types::access_method::{
    AccessMethod, AccessMethodSetting,
    BuiltInAccessMethod::{Bridge, Direct, EncryptedDnsProxy},
    Id, Settings,
};
use talpid_types::net::proxy::{self, Shadowsocks, Socks5Remote};

use super::helpers::{convert_c_string, RustAccessMethodSettingVector};

#[no_mangle]
unsafe extern "C" fn convert_builtin_access_method_setting(
    unique_identifier: *const c_char,
    name: *const c_char,
    is_enabled: bool,
    method_kind: SwiftAccessMethodKind,
    proxy_configuration: *const c_void,
) -> *mut c_void {
    match unsafe {
        convert_builtin_access_method_setting_inner(
            unique_identifier,
            name,
            is_enabled,
            method_kind,
            proxy_configuration,
        )
    } {
        Some(access_method) => Box::into_raw(Box::new(access_method)) as *mut c_void,
        None => null_mut(),
    }
}

unsafe fn convert_builtin_access_method_setting_inner(
    unique_identifier: *const c_char,
    name: *const c_char,
    enabled: bool,
    method_kind: SwiftAccessMethodKind,
    proxy_configuration: *const c_void,
) -> Option<AccessMethodSetting> {
    let id = Id::from_string(convert_c_string(unique_identifier))?;
    let name = convert_c_string(name);
    match method_kind {
        SwiftAccessMethodKind::KindDirect => Some(AccessMethodSetting::with_id(
            id,
            name,
            enabled,
            AccessMethod::BuiltIn(Direct),
        )),
        SwiftAccessMethodKind::KindBridge => Some(AccessMethodSetting::with_id(
            id,
            name,
            enabled,
            AccessMethod::BuiltIn(Bridge),
        )),

        SwiftAccessMethodKind::KindEncryptedDnsProxy => Some(AccessMethodSetting::with_id(
            id,
            name,
            enabled,
            AccessMethod::BuiltIn(EncryptedDnsProxy),
        )),

        SwiftAccessMethodKind::KindShadowsocks => match proxy_configuration.is_null() {
            true => None,
            false => {
                let configuration: Shadowsocks =
                    unsafe { *Box::from_raw(proxy_configuration as *mut _) };
                Some(AccessMethodSetting::with_id(
                    id,
                    name,
                    enabled,
                    AccessMethod::Custom(proxy::CustomProxy::Shadowsocks(configuration)),
                ))
            }
        },
        SwiftAccessMethodKind::KindSocks5Local => match proxy_configuration.is_null() {
            true => None,
            false => {
                let configuration: Socks5Remote =
                    unsafe { *Box::from_raw(proxy_configuration as *mut _) };
                Some(AccessMethodSetting::with_id(
                    id,
                    name,
                    enabled,
                    AccessMethod::Custom(proxy::CustomProxy::Socks5Remote(configuration)),
                ))
            }
        },
    }
}

#[allow(dead_code)]
#[repr(u8)]
pub enum SwiftAccessMethodKind {
    KindDirect = 0,
    KindBridge,
    KindEncryptedDnsProxy,
    KindShadowsocks,
    KindSocks5Local,
}

#[no_mangle]
pub unsafe extern "C" fn init_access_method_settings_wrapper(
    direct_method_raw: *const c_void,
    bridges_method_raw: *const c_void,
    encrypted_dns_method_raw: *const c_void,
    custom_methods_raw: RustAccessMethodSettingVector,
) -> SwiftAccessMethodSettingsWrapper {
    let direct: AccessMethodSetting = unsafe { *Box::from_raw(direct_method_raw as *mut _) };
    let mullvad_bridges: AccessMethodSetting =
        unsafe { *Box::from_raw(bridges_method_raw as *mut _) };
    let encrypted_dns_proxy: AccessMethodSetting =
        unsafe { *Box::from_raw(encrypted_dns_method_raw as *mut _) };

    let custom = custom_methods_raw.into_rust_context().vector;
    let settings = Settings::new(direct, mullvad_bridges, encrypted_dns_proxy, custom);
    let context = SwiftAccessMethodSettingsContext { settings };
    SwiftAccessMethodSettingsWrapper::new(context)
}

#[repr(C)]
pub struct SwiftAccessMethodSettingsWrapper(*mut SwiftAccessMethodSettingsContext);

impl SwiftAccessMethodSettingsWrapper {
    pub fn new(context: SwiftAccessMethodSettingsContext) -> SwiftAccessMethodSettingsWrapper {
        SwiftAccessMethodSettingsWrapper(Box::into_raw(Box::new(context)))
    }

    pub unsafe fn into_rust_context(self) -> Box<SwiftAccessMethodSettingsContext> {
        Box::from_raw(self.0)
    }
}

#[derive(Debug)]
pub struct SwiftAccessMethodSettingsContext {
    pub settings: Settings,
}

impl SwiftAccessMethodSettingsContext {
    pub fn convert_access_method(&self) -> Option<Settings> {
        Some(self.settings.clone())
    }
}
