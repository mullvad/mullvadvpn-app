use std::sync::Arc;

use mullvad_types::access_method::{
    AccessMethod, AccessMethodSetting,
    BuiltInAccessMethod::{Bridge, Direct, EncryptedDnsProxy},
    Id, Settings,
};
use talpid_types::net::proxy::{self, Shadowsocks, Socks5Remote};

/// Converts parameters into a `Box<AccessMethodSetting>` raw representation that
/// can be passed across the FFI boundary
///
/// # SAFETY:
/// `unique_identifier` and `name` must point to valid memory regions and contain NULL terminators.
/// They are only valid for the duration of this call.
///
/// `proxy_configuration` can be NULL, or must be a pointer gotten through
/// either the `convert_shadowsocks` or `convert_socks5` methods.
#[uniffi::export]
pub fn convert_builtin_access_method_setting(
    unique_identifier: String,
    name: String,
    is_enabled: bool,
    method_kind: SwiftAccessMethodKind,
    proxy_configuration: Option<u64>,
) -> Option<Arc<AccessMethodSettingWrapper>> {
    convert_builtin_access_method_setting_inner(
        unique_identifier,
        name,
        is_enabled,
        method_kind,
        proxy_configuration,
    )
    .map(|access_method| {
        Arc::new(AccessMethodSettingWrapper {
            inner: access_method,
        })
    })
}

/// Converts parameters into an `AccessMethodSetting`
///
/// This function copies the strings from the conversion of the variables
/// `unique_identifier`, `name`, and takes ownership of `proxy_configuration`
fn convert_builtin_access_method_setting_inner(
    unique_identifier: String,
    name: String,
    enabled: bool,
    method_kind: SwiftAccessMethodKind,
    proxy_configuration: Option<u64>,
) -> Option<AccessMethodSetting> {
    // SAFETY: See `convert_builtin_access_method_setting`
    let id = Id::from_string(unique_identifier)?;
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

        SwiftAccessMethodKind::KindShadowsocks => {
            proxy_configuration.map(|configuration| {
                // SAFETY: See `convert_builtin_access_method_setting`
                let configuration: Shadowsocks = unsafe { *Box::from_raw(configuration as *mut _) };
                AccessMethodSetting::with_id(
                    id,
                    name,
                    enabled,
                    AccessMethod::Custom(proxy::CustomProxy::Shadowsocks(configuration)),
                )
            })
        }
        SwiftAccessMethodKind::KindSocks5Local => {
            proxy_configuration.map(|configuration| {
                // SAFETY: See `convert_builtin_access_method_setting`
                let configuration: Socks5Remote =
                    unsafe { *Box::from_raw(configuration as *mut _) };
                AccessMethodSetting::with_id(
                    id,
                    name,
                    enabled,
                    AccessMethod::Custom(proxy::CustomProxy::Socks5Remote(configuration)),
                )
            })
        }
    }
}

/// Used by Swift to instruct which access method kind it is trying to convert

#[derive(uniffi::Enum)]
#[repr(u8)]
pub enum SwiftAccessMethodKind {
    KindDirect = 0,
    KindBridge,
    KindEncryptedDnsProxy,
    KindShadowsocks,
    KindSocks5Local,
}

#[derive(uniffi::Object)]
pub struct AccessMethodSettingWrapper {
    pub inner: AccessMethodSetting,
}

/// Creates a wrapper around a `Settings` object that can be safely sent across the FFI boundary.
///
/// # SAFETY
/// `direct_method_raw`, `bridges_method_raw` and `encrypted_dns_method_raw` must be raw pointers
/// resulting from a call to `convert_builtin_access_method_setting`.
/// `custom_methods_raw` is an array of pointers to instances of `AccessMethodSetting`.
#[uniffi::export]
pub fn init_access_method_settings_wrapper(
    direct: Arc<AccessMethodSettingWrapper>,
    bridges: Arc<AccessMethodSettingWrapper>,
    encrypted_dns: Arc<AccessMethodSettingWrapper>,
    custom: Vec<Arc<AccessMethodSettingWrapper>>,
) -> SwiftAccessMethodSettingsContext {
    // SAFETY: each of these pointers must be created by a call to
    // `convert_builtin_access_method_setting`, as per the function docs.

    // SAFETY: custom_methods_raw must be a valid pointer to an AccessMethodSetting.
    let settings = Settings::new(
        direct.inner.clone(),
        bridges.inner.clone(),
        encrypted_dns.inner.clone(),
        custom.into_iter().map(|a| a.inner.clone()).collect(),
    );
    SwiftAccessMethodSettingsContext { settings }
}

#[derive(Debug, uniffi::Object)]
pub struct SwiftAccessMethodSettingsContext {
    pub settings: Settings,
}
#[uniffi::export]
impl SwiftAccessMethodSettingsContext {
    #[uniffi::constructor]
    pub fn unsafe_from_raw(ptr: u64) -> Self {
        // SAFETY: todo
        unsafe { *Box::from_raw(ptr as *mut SwiftAccessMethodSettingsContext) }
    }
}

impl SwiftAccessMethodSettingsContext {
    pub fn convert_access_method(&self) -> Option<Settings> {
        Some(self.settings.clone())
    }
}
