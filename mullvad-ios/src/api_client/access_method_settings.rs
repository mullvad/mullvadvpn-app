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

use super::helpers::convert_c_string;

/// Converts parameters into a `Box<AccessMethodSetting>` raw representation that
/// can be passed across the FFI boundary
///
/// # SAFETY:
/// `unique_identifier` and `name` must point to valid memory regions and contain NULL terminators.
/// `proxy_configuration` can be NULL, or must be a pointer gotten through
/// either the `convert_shadowsocks` or `convert_socks5` methods.
#[unsafe(no_mangle)]
unsafe extern "C" fn convert_builtin_access_method_setting(
    unique_identifier: *const c_char,
    name: *const c_char,
    is_enabled: bool,
    method_kind: SwiftAccessMethodKind,
    proxy_configuration: *const c_void,
) -> *mut c_void {
    match convert_builtin_access_method_setting_inner(
        unique_identifier,
        name,
        is_enabled,
        method_kind,
        proxy_configuration,
    ) {
        Some(access_method) => Box::into_raw(Box::new(access_method)) as *mut c_void,
        None => null_mut(),
    }
}

/// Converts parameters into an `AccessMethodSetting`
fn convert_builtin_access_method_setting_inner(
    unique_identifier: *const c_char,
    name: *const c_char,
    enabled: bool,
    method_kind: SwiftAccessMethodKind,
    proxy_configuration: *const c_void,
) -> Option<AccessMethodSetting> {
    // SAFETY: See `convert_builtin_access_method_setting`
    let id = Id::from_string(unsafe { convert_c_string(unique_identifier) })?;
    // SAFETY: See `convert_builtin_access_method_setting`
    let name = unsafe { convert_c_string(name) };
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
                // SAFETY: See `convert_builtin_access_method_setting`
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
                // SAFETY: See `convert_builtin_access_method_setting`
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

/// Used by Swift to instruct which access method kind it is trying to convert
#[allow(dead_code)]
#[repr(u8)]
pub enum SwiftAccessMethodKind {
    KindDirect = 0,
    KindBridge,
    KindEncryptedDnsProxy,
    KindShadowsocks,
    KindSocks5Local,
}

/// Creates a wrapper around a `Settings` object that can be safely sent across the FFI boundary.
///
/// # SAFETY
/// `direct_method_raw`, `bridges_method_raw` and `encrypted_dns_method_raw` must be raw pointers
/// resulting from a call to `convert_builtin_access_method_setting`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_access_method_settings_wrapper(
    direct_method_raw: *const c_void,
    bridges_method_raw: *const c_void,
    encrypted_dns_method_raw: *const c_void,
    custom_methods_raw: *const c_void,
    count: isize,
) -> SwiftAccessMethodSettingsWrapper {
    // SAFETY: See `init_access_method_settings_wrapper`
    let (direct, mullvad_bridges, encrypted_dns_proxy) = unsafe {
        (
            *Box::from_raw(direct_method_raw as *mut _),
            *Box::from_raw(bridges_method_raw as *mut _),
            *Box::from_raw(encrypted_dns_method_raw as *mut _),
        )
    };

    let custom = access_methods_from_raw_vector(custom_methods_raw, count);
    let settings = Settings::new(direct, mullvad_bridges, encrypted_dns_proxy, custom);
    let context = SwiftAccessMethodSettingsContext { settings };
    SwiftAccessMethodSettingsWrapper::new(context)
}

unsafe fn access_methods_from_raw_vector(
    vector_raw: *const c_void,
    count: isize,
) -> Vec<AccessMethodSetting> {
    let mut custom: Vec<AccessMethodSetting> = usize::try_from(count)
        .map(Vec::with_capacity)
        .unwrap_or_default();
    let vector_raw: *mut *mut AccessMethodSetting = vector_raw as _;
    for i in 0..count {
        let ptr: *mut AccessMethodSetting = *vector_raw.offset(i);
        let setting = Box::from_raw(ptr);
        custom.push(*setting);
    }
    custom
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
